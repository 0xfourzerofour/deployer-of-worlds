use crate::{
    action::{ActionData, DeploymentData, ReadData, WriteData},
    parser::OutputCollector,
};
use std::sync::Arc;

use alloy::{
    contract::CallBuilder,
    dyn_abi::{DynSolType, JsonAbiExt},
    primitives::{Address, Bytes},
    providers::Provider,
};

use crate::action::Action;

#[derive(Debug)]
pub struct Executor<P> {
    provider: Arc<P>,
    actions: Vec<Action>,
}

impl<P> Executor<P>
where
    P: Provider,
{
    pub fn new(provider: P) -> Self {
        Self {
            provider: Arc::new(provider),
            actions: vec![],
        }
    }

    pub async fn execute_actions(&mut self) -> anyhow::Result<()> {
        let mut output_data = OutputCollector::new();
        for action in &self.actions {
            match &action.action_data {
                ActionData::Deploy(deploy_data) => {
                    self.deploy(deploy_data, &mut output_data).await?
                }
                ActionData::Write(write_data) => self.write(write_data, &mut output_data).await?,
                ActionData::Read(read_data) => self.read(read_data, &mut output_data).await?,
            }
        }
        Ok(())
    }

    async fn read(
        &self,
        data: &ReadData,
        _output_data: &mut OutputCollector,
    ) -> anyhow::Result<()> {
        let to: Address = data.address.parse()?;
        let mut dyn_args = vec![];

        for (i, arg) in data.args.iter().enumerate() {
            let sol_type = DynSolType::parse(&data.function.inputs[i].ty)?;
            let val = sol_type.coerce_str(arg)?;
            dyn_args.push(val);
        }

        let input = data.function.abi_encode_input(&dyn_args)?;

        let tx_req = CallBuilder::new_raw(self.provider.clone(), Bytes::from(input))
            .to(to)
            .call_raw()
            .with_decoder(&data.function)
            .await?;

        println!("{:?}", tx_req);

        Ok(())
    }

    async fn write(
        &self,
        _data: &WriteData,
        _output_data: &mut OutputCollector,
    ) -> anyhow::Result<()> {
        Ok(())
    }

    async fn deploy(
        &self,
        _data: &DeploymentData,
        _output_data: &mut OutputCollector,
    ) -> anyhow::Result<()> {
        // let address = output_data.get_input_value(data.address.clone())?;
        Ok(())
    }

    pub fn register_actions(&mut self, actions: Vec<Action>) {
        self.actions = actions;
    }
}
