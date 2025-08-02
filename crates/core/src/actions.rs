use crate::{variables::VariableValue, Result, VariableResolver};
use alloy::{
    dyn_abi::{DynSolValue, FunctionExt, JsonAbiExt},
    providers::network::TransactionBuilder,
};
use serde::{Deserialize, Serialize};

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
    // /// Optional condition that must be met for this write to execute
    // #[serde(default)]
    // pub condition: Option<Condition>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadData {
    pub address: VariableValue,
    pub args: Vec<VariableValue>,
    pub abi_item: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Condition {
    /// Compare a value from a contract call
    ContractCall {
        address: VariableValue,
        abi_item: String,
        args: Vec<VariableValue>,
        comparison: Comparison,
    },
    /// Compare an output from a previous action
    OutputComparison {
        output_ref: String,
        comparison: Comparison,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "operator", content = "value", rename_all = "snake_case")]
pub enum Comparison {
    /// Less than the specified value
    Lt(VariableValue),
    /// Less than or equal to the specified value
    Lte(VariableValue),
    /// Greater than the specified value
    Gt(VariableValue),
    /// Greater than or equal to the specified value
    Gte(VariableValue),
    /// Equal to the specified value
    Eq(VariableValue),
    /// Not equal to the specified value
    Ne(VariableValue),
}

impl Condition {
    /// Evaluate the condition using the given provider and resolver
    pub async fn evaluate<P, R>(&self, provider: &P, resolver: &R) -> Result<bool>
    where
        P: alloy::providers::Provider,
        R: VariableResolver,
    {
        match self {
            Condition::ContractCall {
                address,
                abi_item,
                args,
                comparison,
            } => {
                // Make a contract call to get the current value
                let contract_address = address
                    .resolve(alloy::dyn_abi::DynSolType::Address, resolver)?
                    .as_address()
                    .ok_or_else(|| crate::DeployerError::Config("Invalid address".to_string()))?;

                let function: alloy::json_abi::Function = abi_item
                    .parse()
                    .map_err(|e| crate::DeployerError::Config(format!("Invalid ABI: {}", e)))?;

                let call_args: Vec<DynSolValue> = args
                    .iter()
                    .enumerate()
                    .map(|(i, arg)| {
                        let sol_type = alloy::dyn_abi::DynSolType::parse(&function.inputs[i].ty)
                            .map_err(|e| {
                                crate::DeployerError::Config(format!("Invalid type: {}", e))
                            })?;
                        arg.resolve(sol_type, resolver)
                    })
                    .collect::<Result<Vec<_>>>()?;

                let call_data = function.abi_encode_input(&call_args).map_err(|e| {
                    crate::DeployerError::Config(format!("Failed to encode call: {}", e))
                })?;

                let result = provider
                    .call(
                        alloy::rpc::types::TransactionRequest::default()
                            .with_to(contract_address)
                            .with_input(call_data),
                    )
                    .await
                    .map_err(|e| {
                        crate::DeployerError::Config(format!("Contract call failed: {}", e))
                    })?;

                let decoded = function.abi_decode_output(&result).map_err(|e| {
                    crate::DeployerError::Config(format!("Failed to decode result: {}", e))
                })?;

                // For simplicity, assume single return value for now
                let current_value = decoded
                    .first()
                    .ok_or_else(|| crate::DeployerError::Config("No return value".to_string()))?;

                comparison.evaluate(current_value, resolver)
            }
            Condition::OutputComparison {
                output_ref,
                comparison,
            } => {
                let current_value = resolver.get_output(output_ref)?;
                comparison.evaluate(&current_value, resolver)
            }
        }
    }
}

impl Comparison {
    /// Evaluate the comparison between current_value and the target value
    pub fn evaluate<R: VariableResolver>(
        &self,
        current_value: &DynSolValue,
        resolver: &R,
    ) -> Result<bool> {
        match self {
            Comparison::Lt(target) => {
                let target_val = self.resolve_target_value(target, current_value, resolver)?;
                Ok(self.compare_values(current_value, &target_val)? < 0)
            }
            Comparison::Lte(target) => {
                let target_val = self.resolve_target_value(target, current_value, resolver)?;
                Ok(self.compare_values(current_value, &target_val)? <= 0)
            }
            Comparison::Gt(target) => {
                let target_val = self.resolve_target_value(target, current_value, resolver)?;
                Ok(self.compare_values(current_value, &target_val)? > 0)
            }
            Comparison::Gte(target) => {
                let target_val = self.resolve_target_value(target, current_value, resolver)?;
                Ok(self.compare_values(current_value, &target_val)? >= 0)
            }
            Comparison::Eq(target) => {
                let target_val = self.resolve_target_value(target, current_value, resolver)?;
                Ok(self.compare_values(current_value, &target_val)? == 0)
            }
            Comparison::Ne(target) => {
                let target_val = self.resolve_target_value(target, current_value, resolver)?;
                Ok(self.compare_values(current_value, &target_val)? != 0)
            }
        }
    }

    fn resolve_target_value<R: VariableResolver>(
        &self,
        target: &VariableValue,
        current_value: &DynSolValue,
        resolver: &R,
    ) -> Result<DynSolValue> {
        // Try to infer the type from the current value
        let target_type = current_value.as_type().ok_or_else(|| {
            crate::DeployerError::Config("Cannot determine value type".to_string())
        })?;

        target.resolve(target_type, resolver)
    }

    fn compare_values(&self, a: &DynSolValue, b: &DynSolValue) -> Result<i32> {
        match (a, b) {
            (DynSolValue::Uint(a_val, _), DynSolValue::Uint(b_val, _)) => {
                Ok(a_val.cmp(b_val) as i32)
            }
            (DynSolValue::Int(a_val, _), DynSolValue::Int(b_val, _)) => Ok(a_val.cmp(b_val) as i32),
            (DynSolValue::Address(a_val), DynSolValue::Address(b_val)) => {
                Ok(a_val.cmp(b_val) as i32)
            }
            (DynSolValue::Bool(a_val), DynSolValue::Bool(b_val)) => Ok(a_val.cmp(b_val) as i32),
            _ => Err(crate::DeployerError::Config(
                "Cannot compare values of different types".to_string(),
            )),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Action {
    pub depends_on: Option<Vec<String>>,
    pub id: String,
    pub action_data: ActionData,
}
