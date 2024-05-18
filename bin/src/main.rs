use alloy::providers::ProviderBuilder;
use alloy::rpc::client::ClientBuilder;
use anyhow::Result;
use deployer::action::load_actions;
use deployer::executor::Executor;
use url::Url;

#[tokio::main]
async fn main() -> Result<()> {
    let rpc_url = std::env::var("RPC_URL")?;
    let client = ClientBuilder::default().http(Url::parse(&rpc_url)?);
    let provider = ProviderBuilder::new().on_client(client);
    let mut executor = Executor::new(provider.boxed());
    let actions = load_actions("./examples/actions.json")?;
    executor.register_actions(actions)?;
    executor.execute_actions().await?;

    Ok(())
}
