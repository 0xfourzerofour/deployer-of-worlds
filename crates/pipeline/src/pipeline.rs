use alloy::{
    dyn_abi::DynSolValue,
    json_abi::{Constructor, Function},
    primitives::{Address, Bytes, U256},
};

pub struct Pipeline {
    actions: Vec<ActionData>,
}

impl Pipeline {
    pub fn new(actions: Vec<ActionData>) -> Self {
        Self { actions }
    }
}

#[derive(Debug, Clone)]
pub enum ActionData {
    Deploy(DeploymentData),
    Write(WriteData),
    Read(ReadData),
}

#[derive(Debug, Clone)]
pub struct DeploymentData {
    pub address: Address,
    pub constructor_args: Vec<DynSolValue>,
    pub salt: U256,
    pub constructor_abi_item: Constructor,
    pub bytecode: Bytes,
}

#[derive(Debug, Clone)]
pub struct WriteData {
    pub address: Address,
    pub abi_item: Function,
    pub args: Vec<DynSolValue>,
    pub value: Option<U256>,
}

#[derive(Debug, Clone)]
pub struct ReadData {
    pub address: Address,
    pub args: Vec<DynSolValue>,
    pub abi_item: Function,
}
