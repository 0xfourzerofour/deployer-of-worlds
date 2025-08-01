use alloy::primitives::{Address, Bytes, FixedBytes, U256};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentResult {
    pub address: Address,
    pub transaction_hash: FixedBytes<32>,
    pub gas_used: U256,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionResult {
    pub success: bool,
    pub transaction_hash: FixedBytes<32>,
    pub gas_used: U256,
    pub return_data: Option<Bytes>,
}

#[derive(Debug, Clone)]
pub struct ExecutionContext {
    pub chain_id: u64,
    pub gas_price: Option<U256>,
    pub gas_limit: Option<U256>,
    pub nonce: Option<u64>,
}