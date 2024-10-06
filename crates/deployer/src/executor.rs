use crate::{
    config::config::{ActionData, Config, DeploymentData, ReadData, WriteData},
    indexer::Indexer,
    utils::topological_sort,
};
use std::sync::Arc;

use alloy::{
    contract::CallBuilder,
    dyn_abi::{DynSolType, DynSolValue, JsonAbiExt},
    json_abi::Function,
    primitives::Bytes,
    providers::{network::TransactionBuilder, Provider},
    rpc::types::TransactionRequest,
};

#[derive(Debug)]
pub struct Executor<P> {
    provider: Arc<P>,
    indexer: Indexer,
    config: Config,
}

impl<P> Executor<P>
where
    P: Provider,
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
                ActionData::Deploy(deploy_data) => self.deploy(deploy_data).await?,
                ActionData::Write(write_data) => self.write(write_data).await?,
                ActionData::Read(read_data) => self.read(action.id, read_data).await?,
            }
        }
        Ok(())
    }

    async fn read(&mut self, id: String, data: &ReadData) -> anyhow::Result<()> {
        let function: Function = data.abi_item.parse()?;
        let address = data
            .address
            .resolve(DynSolType::Address, &self.indexer)?
            .as_address()
            .expect("Should have resolved into address");
        let args: Vec<DynSolValue> = data
            .args
            .iter()
            .enumerate()
            .map(|(i, arg)| {
                let sol_type = DynSolType::parse(&function.inputs[i].ty).expect("Invalid type");
                arg.resolve(sol_type, &self.indexer)
                    .expect("Should have resolved")
            })
            .collect();

        let input = function.abi_encode_input(&args)?;

        let read_output = CallBuilder::new_raw(self.provider.clone(), Bytes::from(input))
            .to(address)
            .call_raw()
            .with_decoder(&function)
            .await?;

        self.indexer
            .save_output_data(id, function.outputs.clone(), read_output)?;

        Ok(())
    }

    async fn write(&self, data: &WriteData) -> anyhow::Result<()> {
        let function: Function = data.abi_item.parse()?;
        let address = data
            .address
            .resolve(DynSolType::Address, &self.indexer)?
            .as_address()
            .expect("Should have resolved into address");

        let value = data
            .value
            .resolve(DynSolType::Uint(256), &self.indexer)?
            .as_uint()
            .expect("Should have resolved into address");

        let args: Vec<DynSolValue> = data
            .args
            .iter()
            .enumerate()
            .map(|(i, arg)| {
                let sol_type = DynSolType::parse(&function.inputs[i].ty).expect("Invalid type");
                arg.resolve(sol_type, &self.indexer)
                    .expect("Should have resolved")
            })
            .collect();

        let input = function.abi_encode_input(&args)?;

        let tx = TransactionRequest::default()
            .with_to(address)
            .with_value(value.0)
            .with_input(Bytes::from(input));

        println!("TX to send {:?}", tx);

        Ok(())
    }

    async fn deploy(&self, _data: &DeploymentData) -> anyhow::Result<()> {
        // let address = output_data.get_input_value(data.address.clone())?;
        Ok(())
    }

    pub fn register_config(&mut self, config: Config) {
        config.variables.iter().for_each(|(k, v)| {
            self.indexer
                .save_variable(k, &v.ty, &v.value)
                .expect("Expected to save value")
        });

        self.config = config;
    }
}
