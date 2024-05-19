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
    address: String,
    constructor_args: Vec<String>,
    salt: U256,
    abi: AbiItem<'static>,
    bytecode: Bytes,
}

#[derive(Debug, Clone, Deserialize)]
pub struct WriteData {
    address: String,
    abi: AbiItem<'static>,
    args: Vec<String>,
    value: U256,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ReadData {
    address: String,
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
