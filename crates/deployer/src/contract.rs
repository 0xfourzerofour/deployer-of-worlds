use alloy_core::{json_abi::JsonAbi, primitives::Address, primitives::Bytes};

#[derive(Debug)]
pub struct Contract {
    name: String,
    address: Address,
    abi: JsonAbi,
    bytecode: Bytes,
    constructor_args: Vec<String>,
    salt: String,
}

impl Contract {
    pub fn new(
        name: String,
        address: Address,
        abi: JsonAbi,
        bytecode: Bytes,
        constructor_args: Vec<String>,
        salt: String,
    ) -> Self {
        Self {
            name,
            address,
            abi,
            bytecode,
            constructor_args,
            salt,
        }
    }
}
