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

    pub fn register_actions(&mut self, actions: Vec<Action>) -> Result<()> {
        let action_ids: HashMap<String, usize> = actions
            .iter()
            .enumerate()
            .map(|(i, a)| (a.id.clone(), i))
            .collect();

        let mut graph = HashMap::new();

        for action in &actions {
            let depends_on_ids: Vec<usize> = action
                .depends_on
                .iter()
                .map(|i| *action_ids.get(i).expect("should be here"))
                .collect();

            graph.insert(
                *action_ids.get(&action.id).expect("should be here"),
                depends_on_ids,
            );
        }

        // Perform topological sorting to find the execution order
        let order = topological_sort(&graph)?;

        // Validate for circular dependencies
        if order.len() != actions.len() {
            anyhow::bail!("Circular dependency found");
        }

        let mut ordered_list = vec![];

        for id in order {
            let action = actions
                .get(id)
                .expect("Action should be present after topological sort");
            ordered_list.push((*action).clone());
        }

        self.actions = ordered_list;
        Ok(())
    }
}

// Helper function for topological sort
fn topological_sort(graph: &HashMap<usize, Vec<usize>>) -> Result<Vec<usize>, anyhow::Error> {
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
