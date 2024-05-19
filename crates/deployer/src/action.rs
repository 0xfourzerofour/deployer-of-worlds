use std::{collections::HashMap, fs};

use alloy::{
    json_abi::{AbiItem, JsonAbi},
    primitives::{Address, Bytes, B256},
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
    salt: B256,
    abi: AbiItem<'static>,
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
    Ok(actions)
}
