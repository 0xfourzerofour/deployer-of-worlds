use crate::action::{DeploymentData, ReadData, WriteData};
use anyhow::Result;
use jq_rs;
use std::{collections::HashMap, sync::Arc};

use alloy::{primitives::address, primitives::Address, providers::Provider};

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

    pub async fn execute_actions(&self) -> anyhow::Result<()> {
        let mut _output_data: HashMap<String, String> = HashMap::new();
        for action in &self.actions {
            if let Some(inputs) = &action.inputs {
                for input in inputs {
                    let (prefix, jq_query) = input.split_once(".").expect("Invalid jq format");

                    let output = output_data
                        .get(prefix)
                        .expect("Cound not find output data based on prefix");

                    let input_val = jq_rs::run(jq_query, output).unwrap();
                }
            }

            println!("{}", action.id);
        }
        Ok(())
    }

    async fn read(&self, _data: ReadData) -> anyhow::Result<()> {
        Ok(())
    }

    async fn write(&self, _data: WriteData) -> anyhow::Result<()> {
        Ok(())
    }

    async fn deploy(&self, data: DeploymentData) -> anyhow::Result<()> {
        Ok(())
    }

    pub fn register_actions(&mut self, actions: Vec<Action>) {
        self.actions = actions;
    }
}
