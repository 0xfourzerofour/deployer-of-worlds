use crate::{
    action::{ActionData, DeploymentData, ReadData, WriteData},
    parser::OutputCollector,
};
use std::{collections::HashMap, sync::Arc};

use alloy::{primitives::Address, providers::Provider, rpc::types::eth::{TransactionRequest, TransactionInput}};

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
            inputs: HashMap::new(),
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
        _data: &ReadData,
        output_data: &mut OutputCollector,
    ) -> anyhow::Result<()> {
        Ok(())
    }

    async fn write(
        &self,
        _data: &WriteData,
        output_data: &mut OutputCollector,
    ) -> anyhow::Result<()> {
        Ok(())
    }

    async fn deploy(
        &self,
        data: &DeploymentData,
        output_data: &mut OutputCollector,
    ) -> anyhow::Result<()> {
        let address = output_data.get_input_value(data.address.clone())?;
        let tx_input = TransactionInput::new(data.constructor_args);
        let tx_req = TransactionRequest::default().to(Address::ZERO).input()

        self.provider.send_transaction().await?;
        Ok(())
    }

    pub fn register_actions(&mut self, actions: Vec<Action>) {
        self.actions = actions;
    }
}
