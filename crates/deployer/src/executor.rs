use crate::{
    action::{ActionData, DeploymentData, ReadData, WriteData},
    indexer::Indexer,
};
use std::sync::Arc;

use alloy::{
    contract::CallBuilder,
    dyn_abi::{DynSolType, DynSolValue, JsonAbiExt},
    json_abi::{AbiItem, Function},
    primitives::{Address, Bytes},
    providers::Provider,
};

use crate::action::Action;

#[derive(Debug)]
pub struct Executor<P> {
    provider: Arc<P>,
    indexer: Indexer,
    actions: Vec<Action>,
}

impl<P> Executor<P>
where
    P: Provider,
{
    pub fn new(provider: P) -> Self {
        Self {
            provider: Arc::new(provider),
            indexer: Indexer::new(),
            actions: vec![],
        }
    }

    pub async fn execute_actions(&mut self) -> anyhow::Result<()> {
        for action in self.actions.clone() {
            match &action.action_data {
                ActionData::Deploy(deploy_data) => self.deploy(deploy_data).await?,
                ActionData::Write(write_data) => self.write(write_data).await?,
                ActionData::Read(read_data) => self.read(action.id, read_data).await?,
            }
        }
        Ok(())
    }

    async fn read(&mut self, id: String, data: &ReadData) -> anyhow::Result<()> {
        let to: Address = data.address.parse()?;
        let item: AbiItem = data.function_signature.parse()?;
        println!("item {:?}", item);
        let function: Function = data.function_signature.parse()?;
        println!("function {:?}", function);
        let dyn_args: Vec<DynSolValue> = data
            .args
            .iter()
            .enumerate()
            .map(|(i, arg)| {
                let sol_type = DynSolType::parse(&function.inputs[i].ty).expect("Invalid type");

                let val = self
                    .indexer
                    .get_input_value(arg, sol_type)
                    .expect("Should be dynamic value or valid static type");

                val
            })
            .collect();

        let input = function.abi_encode_input(&dyn_args)?;
        let read_output = CallBuilder::new_raw(self.provider.clone(), Bytes::from(input))
            .to(to)
            .call_raw()
            .with_decoder(&function)
            .await?;

        self.indexer
            .save_output_data(id, function.outputs.clone(), read_output)?;

        Ok(())
    }

    async fn write(&self, _data: &WriteData) -> anyhow::Result<()> {
        Ok(())
    }

    async fn deploy(&self, _data: &DeploymentData) -> anyhow::Result<()> {
        // let address = output_data.get_input_value(data.address.clone())?;
        Ok(())
    }

    pub fn register_actions(&mut self, actions: Vec<Action>) {
        self.actions = actions;
    }
}
