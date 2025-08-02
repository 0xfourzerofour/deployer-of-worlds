use alloy::{
    network::{EthereumWallet, IntoWallet},
    providers::ProviderBuilder,
    signers::{
        k256::ecdsa::SigningKey,
        local::{LocalSigner, PrivateKeySigner},
    },
};
use anyhow::Result;
use clap_derive::{Parser, Subcommand};
use deployer::{config::config::Config, executor::Executor};
use std::path::PathBuf;

#[derive(Debug, Parser)]
#[command(author, about = "Deployer Of Worlds", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
    #[arg(short, long, env = "DATA_DIR", default_value = "./configs")]
    pub data_dir: String,
    #[arg(long, env = "RPC_URL", default_value = "http://localhost:8545")]
    pub rpc_url: String,
    #[arg(long, env = "PRIVATE_KEY", default_value = "")]
    pub private_key: String,
}

impl Cli {
    pub async fn run(self) -> Result<()> {
        // Initialize tracing
        tracing_subscriber::fmt::init();

        match self.command {
            Commands::Start(ref args) => self.start_server(args).await,
            Commands::Execute(ref args) => self.execute_pipeline(args).await,
        }
    }

    async fn start_server(&self, args: &StartArgs) -> Result<()> {
        tracing::info!("Starting server on port {}", args.port);

        let server =
            crate::server::create_server(self.data_dir.clone(), self.rpc_url.clone()).await?;
        let listener = tokio::net::TcpListener::bind(&format!("0.0.0.0:{}", args.port)).await?;

        tracing::info!("Server listening on http://0.0.0.0:{}", args.port);
        axum::serve(listener, server).await?;

        Ok(())
    }

    async fn execute_pipeline(&self, args: &ExecuteArgs) -> Result<()> {
        tracing::info!("Executing pipeline: {}", args.config);

        // Load configuration from pipelines directory
        let config_path = PathBuf::from(&self.data_dir)
            .join("pipelines")
            .join(&args.config);
        let config = Config::load_from_file(config_path.to_str().unwrap())?;

        if args.dry_run {
            tracing::info!(
                "Dry run mode - would execute {} actions",
                config.actions.len()
            );
            for action in &config.actions {
                tracing::info!(
                    "Would execute action: {} (type: {:?})",
                    action.id,
                    action.action_data
                );
            }
        } else {
            let signer: PrivateKeySigner =
                self.private_key.parse().expect("should parse private key");
            let wallet = EthereumWallet::new(signer);
            let provider = ProviderBuilder::new()
                .wallet(wallet)
                .connect_http(self.rpc_url.parse()?);

            // Create executor with data directory
            let data_dir = PathBuf::from(&self.data_dir).join("data");
            let mut executor = Executor::with_data_dir(provider, data_dir);
            executor.register_config(config)?;
            executor.execute_actions().await?;
        }

        tracing::info!("Pipeline execution completed successfully");
        Ok(())
    }
}

#[derive(Debug, Parser)]
pub struct StartArgs {
    #[arg(short, long, env = "APP_PORT", default_value = "3000")]
    pub port: u16,
}

#[derive(Debug, Parser)]
pub struct ExecuteArgs {
    #[arg(help = "Configuration file name (e.g., deploy.yml)")]
    pub config: String,
    #[arg(short, long, help = "Dry run - don't execute transactions")]
    pub dry_run: bool,
}

/// Commands to be executed
#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Start the HTTP server
    #[command(name = "start")]
    Start(StartArgs),
    /// Execute a pipeline from config file
    #[command(name = "execute")]
    Execute(ExecuteArgs),
}
