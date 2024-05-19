use anyhow::Result;
use std::sync::Arc;

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
        for action in &self.actions {
            println!("{}", action.id);
        }
        Ok(())
    }

    pub fn register_actions(&mut self, actions: Vec<Action>) {
        self.actions = actions;
    }
}
