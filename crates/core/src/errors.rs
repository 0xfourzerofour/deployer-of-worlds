use thiserror::Error;

#[derive(Debug, Error)]
pub enum DeployerError {
    #[error("Configuration error: {0}")]
    Config(String),
    
    #[error("Execution error: {0}")]
    Execution(String),
    
    #[error("Variable not found: {0}")]
    VariableNotFound(String),
    
    #[error("Output not found: {0}")]
    OutputNotFound(String),
    
    #[error("Invalid type conversion: expected {expected}, got {actual}")]
    TypeConversion { expected: String, actual: String },
    
    #[error("Deployment failed: {0}")]
    DeploymentFailed(String),
    
    #[error("Transaction failed: {0}")]
    TransactionFailed(String),
    
    #[error("ABI parsing error: {0}")]
    AbiParsing(String),
    
    #[error("Hex decoding error: {0}")]
    HexDecoding(#[from] hex::FromHexError),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Serialization error: {0}")]
    Serialization(String),
    
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

pub type Result<T> = std::result::Result<T, DeployerError>;