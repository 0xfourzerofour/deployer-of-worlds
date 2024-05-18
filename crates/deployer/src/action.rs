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
    function_sig: String,
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

#[derive(Debug, Clone)]
pub struct Action {
    pub action_type: ActionType,
    pub depends_on: Vec<String>,
    pub id: String,
    pub name: String,
    pub data: String,
    pub inputs: Vec<String>,
    pub outputs_schema: String,
}

pub fn load_actions(path: &str) -> Vec<Action> {
    return vec![];
}
