use anyhow::Result;
use std::sync::Arc;

use alloy::providers::Provider;

use crate::contract::Contract;

#[derive(Debug)]
pub struct Deployer<P> {
    provider: Arc<P>,
    execution_steps: Vec<Actions>,
}

impl<P> Deployer<P>
where
    P: Provider,
{
    pub fn new(provider: P) -> Self {
        Self {
            provider: Arc::new(provider),
            contract_deployments: vec![],
        }
    }
    pub async fn execute_deployment(&self) -> Result<()> {}
}
