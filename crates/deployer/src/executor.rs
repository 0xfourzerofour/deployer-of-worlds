use anyhow::Result;
use jq_rs::run;
use std::{collections::HashMap, sync::Arc};

use alloy::providers::Provider;

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

    pub async fn execute_actions(&self) -> Result<()> {
        let mut output_data = HashMap::new();
        for action in &self.actions {
            if let Some(inputs) = action.inputs {
                for input in inputs {
                    let input_val = jq_rs::run();
                }
            }

            println!("{}", action.id);
        }
        Ok(())
    }

    pub fn register_actions(&mut self, actions: Vec<Action>) {
        self.actions = actions;
    }
}
