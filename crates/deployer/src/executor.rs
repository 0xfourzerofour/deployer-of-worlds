use crate::action::{ActionData, DeploymentData, ReadData, WriteData};
use anyhow::Result;
use std::{collections::HashMap, sync::Arc};

use alloy::{primitives::address, primitives::Address, providers::Provider};

use crate::action::Action;

#[derive(Debug)]
pub struct Executor<P> {
    provider: Arc<P>,
    actions: Vec<Action>,
    inputs: HashMap<String, Vec<(String, String)>>,
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
        let mut output_data: HashMap<String, (String, String)> = HashMap::new();
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
        output_data: &mut HashMap<String, (String, String)>,
    ) -> anyhow::Result<()> {
        Ok(())
    }

    async fn write(
        &self,
        _data: &WriteData,
        output_data: &mut HashMap<String, (String, String)>,
    ) -> anyhow::Result<()> {
        Ok(())
    }

    async fn deploy(
        &self,
        data: &DeploymentData,
        output_data: &mut HashMap<String, (String, String)>,
    ) -> anyhow::Result<()> {
        // add logic to gather abi/initcode/constructor_args to generate initcode
        // use this to recompute the create2 address and make sure it matches the expected address
        Ok(())
    }

    pub fn register_actions(&mut self, actions: Vec<Action>) {
        self.actions = actions;
    }
}
