use std::{collections::HashMap, fs};

use alloy::{
    json_abi::JsonAbi,
    primitives::{Address, Bytes, B256},
};
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub enum ActionType {
    Deployment,
    Write,
    Read,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DeploymentData {
    address: Address,
    constructor_args: Vec<String>,
    salt: B256,
    abi: JsonAbi,
    bytecode: Bytes,
}

#[derive(Debug, Clone, Deserialize)]
pub struct WriteData {
    address: Address,
    function_sig: String,
    args: Vec<String>,
    value: B256,
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
    salt: B256,
    abi: JsonAbi,
    bytecode: Bytes,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Action {
    pub action_type: ActionType,
    pub depends_on: Vec<String>,
    pub id: String,
    pub name: String,
    pub data: HashMap<String, String>,
    pub inputs: Vec<String>,
    pub outputs_schema: OutputSchema,
}

#[derive(Debug, Clone, Deserialize)]
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
    Ok(actions)
}
