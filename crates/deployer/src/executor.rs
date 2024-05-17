use anyhow::Result;
use std::{
    collections::{HashMap, VecDeque},
    sync::Arc,
};

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
        Ok(())
    }

    pub async fn register_actions(&mut self, actions: Vec<Action>) -> Result<()> {
        // Build a dependency graph
        let mut graph = HashMap::new();
        let mut action_graph = HashMap::new();

        for action in &actions {
            graph.insert(action.id.clone(), action.depends_on.clone());
            action_graph.insert(action.id.clone(), action.clone());
        }

        // Perform topological sorting to find the execution order
        let order = topological_sort(&graph)?;

        // Validate for circular dependencies
        if order.len() != actions.len() {
            anyhow::bail!("Circular dependency found");
        }

        let mut ordered_list = vec![];

        for id in order {
            let action = action_graph
                .get(&id)
                .expect("Action should be present after topological sort");
            ordered_list.push((*action).clone());
        }

        self.actions = ordered_list;
        Ok(())
    }
}

// Helper function for topological sort
fn topological_sort(graph: &HashMap<String, Vec<String>>) -> Result<Vec<String>, anyhow::Error> {
    let mut in_degree = HashMap::new();
    let mut queue = VecDeque::new();

    // Initialize in-degree for each node
    for (node, dependencies) in graph {
        let count = dependencies.len();
        in_degree.insert(*node, count);
        if count == 0 {
            queue.push_back(*node);
        }
    }

    let mut ordered = Vec::new();
    while let Some(node) = queue.pop_front() {
        ordered.push(node);
        if let Some(dependencies) = graph.get(&node) {
            for dependent in dependencies {
                if let Some(count) = in_degree.get_mut(dependent) {
                    *count -= 1;
                    if *count == 0 {
                        queue.push_back(*dependent);
                    }
                }
            }
        }
    }

    if ordered.len() != graph.len() {
        Err(anyhow::Error::msg("Circular dependency found"))
    } else {
        Ok(ordered)
    }
}
