use alloy::providers::ProviderBuilder;
use alloy::rpc::client::ClientBuilder;
use anyhow::Result;
use deployer::config::config::Config;
use deployer::executor::Executor;
use dotenv::dotenv;
use url::Url;

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();
    let rpc_url = std::env::var("RPC_URL")?;
    let client = ClientBuilder::default().http(Url::parse(&rpc_url)?);
    let provider = ProviderBuilder::new().on_client(client);
    let mut executor = Executor::new(provider.boxed());
    let config = Config::load_from_file("./examples/read-actions.yml")?;
    executor.register_config(config);
    executor.execute_actions().await?;

    Ok(())
}
