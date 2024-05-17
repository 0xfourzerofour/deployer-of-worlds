use alloy::{
    json_abi::JsonAbi,
    primitives::{Address, Bytes, B256},
};

#[derive(Debug)]
pub enum ActionType {
    Deployment,
    Write,
    Read,
}

pub struct DeploymentData {
    address: Address,
    constructor_args: Vec<String>,
    salt: B256,
    abi: JsonAbi,
    bytecode: Bytes,
}

pub struct WriteData {
    address: Address,
    args: Vec<String>,
    value: B256,
    condition: Option<WriteCondition>,
}

pub struct WriteCondition {
    action_id: String,
    cmp: CpmOption,
}

enum CpmOption {
    Neq,
    Eq,
    Gt,
    Gte,
    Lt,
    Lte,
}

pub struct ReadData {
    address: Address,
    constructor_args: Vec<String>,
    salt: B256,
    abi: JsonAbi,
    bytecode: Bytes,
}

#[derive(Debug)]
pub struct Action {
    action_type: ActionType,
    depends_on: Vec<String>,
    id: String,
    name: String,
    data: String,
    inputs: Vec<String>,
    outputs_schema: String,
}
