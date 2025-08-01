use crate::{
    config::config::Config,
    execution::{DeploymentExecutor, ReadExecutor, WriteExecutor},
    indexer::Indexer,
    utils::topological_sort,
};
use alloy::providers::{network::Ethereum, Provider};
use deployer_core::{ActionData, DeploymentData, ReadData, WriteData};
use std::sync::Arc;

#[derive(Debug)]
pub struct Executor<P> {
    provider: Arc<P>,
    indexer: Indexer,
    config: Config,
}

impl<P> Executor<P>
where
    P: Provider<Ethereum>,
{
    pub fn new(provider: P) -> Self {
        Self {
            provider: Arc::new(provider),
            indexer: Indexer::new(),
            config: Config::new(),
        }
    }

    pub async fn execute_actions(&mut self) -> anyhow::Result<()> {
        let actions = self.config.actions.clone();
        let sorted = topological_sort(actions)?;
        for action in sorted {
            match &action.action_data {
                ActionData::Deploy(deploy_data) => {
                    self.deploy(action.id.clone(), deploy_data).await?
                }
                ActionData::Write(write_data) => self.write(write_data).await?,
                ActionData::Read(read_data) => self.read(action.id, read_data).await?,
            }
        }
        Ok(())
    }

    async fn read(&mut self, id: String, data: &ReadData) -> anyhow::Result<()> {
        let read_executor = ReadExecutor::new(self.provider.clone());
        let read_output = read_executor.read(data, &self.indexer).await?;

        let function: alloy::json_abi::Function = data.abi_item.parse()?;
        self.indexer
            .save_output_data(id, function.outputs.clone(), read_output)?;

        Ok(())
    }

    async fn write(&self, data: &WriteData) -> anyhow::Result<()> {
        let write_executor = WriteExecutor::new(self.provider.clone());
        write_executor.write(data, &self.indexer).await
    }

    async fn deploy(&mut self, action_id: String, data: &DeploymentData) -> anyhow::Result<()> {
        let deployment_executor = DeploymentExecutor::new(self.provider.clone());
        let (deployed_address, _tx_hash) = deployment_executor.deploy(data, &self.indexer).await?;

        // Save deployed address to indexer so it can be referenced by !output
        // For deployment outputs, we store directly with the action ID as the key
        // Create a synthetic output parameter with empty name so it gets stored as just the prefix
        let address_output = alloy::json_abi::Param {
            name: "".to_string(), // Empty name so it uses just the prefix (action_id)
            ty: "address".to_string(),
            internal_type: None,
            components: vec![],
        };
        self.indexer.save_output_data(
            action_id.clone(),
            vec![address_output],
            vec![alloy::dyn_abi::DynSolValue::Address(deployed_address)],
        )?;

        Ok(())
    }

    pub fn register_config(&mut self, config: Config) -> anyhow::Result<()> {
        for (k, v) in &config.variables {
            self.indexer.save_variable(k, &v.ty, &v.value)?;
        }
        self.config = config;
        Ok(())
    }
}
