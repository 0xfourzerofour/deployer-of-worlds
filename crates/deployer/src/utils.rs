use std::collections::{HashMap, HashSet, VecDeque};

use alloy::{
    json_abi::{JsonAbi, Param},
    primitives::Bytes,
};

use crate::config::config::ConfigAction;

pub fn generate_initcode(
    abi: JsonAbi,
    _bytecode: Bytes,
    constructor_args: Vec<Param>,
) -> anyhow::Result<Bytes> {
    if let Some(constructor) = abi.constructor {
        if constructor_args.len() != constructor.inputs.len() {
            anyhow::bail!("Input length an constructor args length are not the same");
        }
    }

    Ok(Bytes::new())
}

pub fn topological_sort(actions: Vec<ConfigAction>) -> anyhow::Result<Vec<ConfigAction>> {
    // Map each action id to its corresponding Action object
    let mut action_map: HashMap<String, ConfigAction> = HashMap::new();
    for action in actions.clone() {
        action_map.insert(action.id.clone(), action);
    }

    // Initialize in-degree map and adjacency list
    let mut in_degree: HashMap<String, usize> = HashMap::new();
    let mut adjacency_list: HashMap<String, HashSet<String>> = HashMap::new();

    for action in &actions {
        in_degree.entry(action.id.clone()).or_insert(0);

        if let Some(depends_on) = &action.depends_on {
            for dependency in depends_on {
                adjacency_list
                    .entry(dependency.clone())
                    .or_insert_with(HashSet::new)
                    .insert(action.id.clone());
                *in_degree.entry(action.id.clone()).or_insert(0) += 1;
            }
        }
    }

    // Initialize the queue with actions having no dependencies
    let mut queue: VecDeque<String> = VecDeque::new();
    for (id, &degree) in &in_degree {
        if degree == 0 {
            queue.push_back(id.clone());
        }
    }

    // Perform the topological sort
    let mut ordered_action_ids: Vec<String> = Vec::new();
    while let Some(action_id) = queue.pop_front() {
        ordered_action_ids.push(action_id.clone());

        if let Some(neighbors) = adjacency_list.get(&action_id) {
            for neighbor in neighbors {
                if let Some(degree) = in_degree.get_mut(neighbor) {
                    *degree -= 1;
                    if *degree == 0 {
                        queue.push_back(neighbor.clone());
                    }
                }
            }
        }
    }

    // Check for cycles
    if ordered_action_ids.len() != actions.len() {
        anyhow::bail!("There is a cycle in the dependencies");
    }

    // Map the ordered ids back to actions
    let ordered_actions = ordered_action_ids
        .into_iter()
        .map(|id| action_map.remove(&id).unwrap())
        .collect();

    Ok(ordered_actions)
}
