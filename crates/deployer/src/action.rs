use std::{
    collections::{HashMap, HashSet, VecDeque},
    fs,
};

use alloy::{
    json_abi::{AbiItem, JsonAbi},
    primitives::{Address, Bytes, U256},
};
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type", content = "content", rename_all = "snake_case")]
pub enum ActionData {
    Deploy(DeploymentData),
    Write(WriteData),
    Read(ReadData),
}

#[derive(Debug, Clone, Deserialize)]
pub struct DeploymentData {
    address: Address,
    constructor_args: Vec<String>,
    salt: U256,
    abi: AbiItem<'static>,
    bytecode: Bytes,
}

#[derive(Debug, Clone, Deserialize)]
pub struct WriteData {
    address: Address,
    abi: AbiItem<'static>,
    args: Vec<String>,
    value: U256,
    condition: Option<WriteCondition>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct WriteCondition {
    action_id: String,
    cmp: CpmOption,
}

#[derive(Debug, Clone, Deserialize)]
enum CpmOption {
    Neq,
    Eq,
    Gt,
    Gte,
    Lt,
    Lte,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ReadData {
    address: Address,
    constructor_args: Vec<String>,
    salt: U256,
    abi: JsonAbi,
    bytecode: Bytes,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Action {
    pub depends_on: Option<Vec<String>>,
    pub id: String,
    pub action_data: ActionData,
    pub inputs: Option<Vec<String>>,
    pub output_schema: Option<OutputSchema>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OutputSchemaType {
    String,
    Object,
    Bool,
    Int,
    Float,
}

#[derive(Debug, Clone, Deserialize)]
pub struct OutputSchema {
    pub output_type: OutputSchemaType,
    pub properties: Option<HashMap<String, OutputSchema>>,
}

pub fn load_actions(path: &str) -> anyhow::Result<Vec<Action>> {
    let contents = fs::read_to_string(path).expect("Should have been able to read the file");
    let actions: Vec<Action> = serde_json::from_str(&contents)?;
    let sorted = topological_sort(actions)?;

    Ok(sorted)
}

fn topological_sort(actions: Vec<Action>) -> anyhow::Result<Vec<Action>> {
    // Map each action id to its corresponding Action object
    let mut action_map: HashMap<String, Action> = HashMap::new();
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
