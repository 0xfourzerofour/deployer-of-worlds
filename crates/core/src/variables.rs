use alloy::dyn_abi::{DynSolType, DynSolValue};
use serde::{de::Error, Deserialize, Deserializer, Serialize};
use serde_yaml::Value as YamlValue;

use crate::errors::{DeployerError, Result};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Variable {
    pub ty: String,
    pub value: String,
}

#[derive(Debug, Clone, Serialize)]
pub enum VariableValue {
    Var(String),
    Output(String),
    Value(String),
    Data(String), // New: reference to data files like !data contract.bytecode
}

impl VariableValue {
    pub fn resolve<R: VariableResolver>(
        &self,
        expected_type: DynSolType,
        resolver: &R,
    ) -> Result<DynSolValue> {
        match self {
            VariableValue::Var(key) => resolver.get_variable(key),
            VariableValue::Output(id) => resolver.get_output(id),
            VariableValue::Value(value) => expected_type
                .coerce_str(value)
                .map_err(|_e| DeployerError::TypeConversion {
                    expected: format!("{:?}", expected_type),
                    actual: value.clone(),
                }),
            VariableValue::Data(path) => resolver.get_data(path),
        }
    }
}

pub trait VariableResolver {
    fn get_variable(&self, key: &str) -> Result<DynSolValue>;
    fn get_output(&self, id: &str) -> Result<DynSolValue>;
    fn get_data(&self, path: &str) -> Result<DynSolValue>;
}

impl<'de> Deserialize<'de> for VariableValue {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let yaml_value = YamlValue::deserialize(deserializer)?;
        match yaml_value {
            YamlValue::Tagged(tagged) => match tagged.tag.to_string().as_str() {
                "!var" => {
                    if let YamlValue::String(s) = tagged.value {
                        Ok(VariableValue::Var(s))
                    } else {
                        Err(Error::custom("Expected string value for !var"))
                    }
                }
                "!output" => {
                    if let YamlValue::String(s) = tagged.value {
                        Ok(VariableValue::Output(s))
                    } else {
                        Err(Error::custom("Expected string value for !output"))
                    }
                }
                "!data" => {
                    if let YamlValue::String(s) = tagged.value {
                        Ok(VariableValue::Data(s))
                    } else {
                        Err(Error::custom("Expected string value for !data"))
                    }
                }
                _ => Err(Error::custom(format!("Unknown tag: {}", tagged.tag))),
            },
            YamlValue::String(s) => Ok(VariableValue::Value(s)),
            _ => Err(Error::custom("Expected string or tagged value")),
        }
    }
}