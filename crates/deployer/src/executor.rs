use crate::{
    config::config::Config,
    data_indexer::DataIndexer,
    execution::{DeploymentExecutor, ReadExecutor, WriteExecutor},
    utils::topological_sort,
};
use alloy::providers::{network::Ethereum, Provider};
use deployer_core::{ActionData, DeploymentData, ReadData, WriteData};
use std::{path::PathBuf, sync::Arc};

#[derive(Debug)]
pub struct Executor<P> {
    provider: Arc<P>,
    indexer: Option<DataIndexer>,
    config: Config,
    data_dir: PathBuf,
}

impl<P> Executor<P>
where
    P: Provider<Ethereum>,
{
    pub fn new(provider: P) -> Self {
        Self::with_data_dir(provider, PathBuf::from("configs/data"))
    }

    pub fn with_data_dir(provider: P, data_dir: PathBuf) -> Self {
        Self {
            provider: Arc::new(provider),
            indexer: None,
            config: Config::new(),
            data_dir,
        }
    }

    pub async fn execute_actions(&mut self) -> anyhow::Result<()> {
        // Ensure indexer is initialized
        if self.indexer.is_none() {
            return Err(anyhow::anyhow!("Config must be registered before executing actions"));
        }
        
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
        let indexer = self.indexer.as_ref()
            .ok_or_else(|| anyhow::anyhow!("Indexer not initialized"))?;
        
        let read_executor = ReadExecutor::new(self.provider.clone());
        let read_output = read_executor.read(data, indexer).await?;

        let function: alloy::json_abi::Function = data.abi_item.parse()?;
        self.indexer.as_mut().unwrap()
            .save_output_data(id, function.outputs.clone(), read_output)?;

        Ok(())
    }

    async fn write(&self, data: &WriteData) -> anyhow::Result<()> {
        let indexer = self.indexer.as_ref()
            .ok_or_else(|| anyhow::anyhow!("Indexer not initialized"))?;
        
        let write_executor = WriteExecutor::new(self.provider.clone());
        write_executor.write(data, indexer).await
    }

    async fn deploy(&mut self, action_id: String, data: &DeploymentData) -> anyhow::Result<()> {
        let indexer = self.indexer.as_ref()
            .ok_or_else(|| anyhow::anyhow!("Indexer not initialized"))?;
        
        let deployment_executor = DeploymentExecutor::new(self.provider.clone());
        let (deployed_address, _tx_hash) = deployment_executor.deploy(data, indexer).await?;

        // Save deployed address to indexer so it can be referenced by !output
        // For deployment outputs, we store directly with the action ID as the key
        // Create a synthetic output parameter with empty name so it gets stored as just the prefix
        let address_output = alloy::json_abi::Param {
            name: "".to_string(), // Empty name so it uses just the prefix (action_id)
            ty: "address".to_string(),
            internal_type: None,
            components: vec![],
        };
        self.indexer.as_mut().unwrap().save_output_data(
            action_id.clone(),
            vec![address_output],
            vec![alloy::dyn_abi::DynSolValue::Address(deployed_address)],
        )?;

        Ok(())
    }

    pub fn register_config(&mut self, config: Config) -> anyhow::Result<()> {
        // Create a new DataIndexer with the config's data references and variables
        let mut data_indexer = DataIndexer::new(
            config.data.clone(),
            config.variables.clone(),
            self.data_dir.clone(),
        );
        
        // Also save variables to the base indexer for backward compatibility
        for (k, v) in &config.variables {
            data_indexer.save_variable(k, &v.ty, &v.value)?;
        }
        
        self.indexer = Some(data_indexer);
        self.config = config;
        Ok(())
    }
}
