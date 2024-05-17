use alloy_core::{json_abi::JsonAbi, primitives::Address, primitives::Bytes};

#[derive(Debug)]
pub struct PreDeploy {
    address: Address,
    function_sig: String,
    args: Vec<String>,
}
