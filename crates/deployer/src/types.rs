use serde::Deserialize;
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvmDataType {
    Address,
    Bool,
    Bytes,
    String,
    UInt8,
    UInt16,
    UInt32,
    UInt64,
    UInt128,
    UInt256,
    Int8,
    Int16,
    Int32,
    Int64,
    Int128,
    Int256,
}

// TODO allow for decoding of eth_Call data into internal types so that we can use them as inputs to other actions
