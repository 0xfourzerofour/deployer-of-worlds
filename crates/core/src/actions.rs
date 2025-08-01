use serde::{Deserialize, Serialize};
use crate::variables::VariableValue;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "content", rename_all = "snake_case")]
pub enum ActionData {
    Deploy(DeploymentData),
    Write(WriteData),
    Read(ReadData),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentData {
    pub address: VariableValue,
    pub constructor_args: Vec<VariableValue>,
    pub salt: VariableValue,
    pub constructor_abi_item: String,
    pub bytecode: VariableValue,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WriteData {
    pub address: VariableValue,
    pub abi_item: String,
    pub args: Vec<VariableValue>,
    pub value: VariableValue,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadData {
    pub address: VariableValue,
    pub args: Vec<VariableValue>,
    pub abi_item: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Action {
    pub depends_on: Option<Vec<String>>,
    pub id: String,
    pub action_data: ActionData,
}