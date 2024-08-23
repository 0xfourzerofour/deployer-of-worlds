use std::collections::HashMap;

use alloy::dyn_abi::DynSolValue;
use serde::{Deserialize, Deserializer};
use serde_yaml::Value;

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum ParseValue {
    Var(Var),
    Output(Output),
    Raw(Value), // Handle all other YAML types
}

#[derive(Debug)]
struct Var {
    value: String,
}

#[derive(Debug)]
struct Output {
    value: String,
}

impl<'de> Deserialize<'de> for Var {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s: String = Deserialize::deserialize(deserializer)?;
        Ok(Var { value: s })
    }
}

impl<'de> Deserialize<'de> for Output {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s: String = Deserialize::deserialize(deserializer)?;
        Ok(Output { value: s })
    }
}

pub fn config_deserializer(
    value: Value,
    external_data: &HashMap<String, String>,
) -> Result<Value, String> {
    match value {
        Value::Tagged(tagged) => match tagged.tag.to_string().as_str() {
            "!var" => {
                let var = tagged
                    .value
                    .as_str()
                    .ok_or("Expected string for !var tag")?;
                let replacement = external_data
                    .get(var)
                    .ok_or(format!("Variable {} not found", var))?;
                Ok(Value::String(replacement.clone()))
            }
            "!output" => {
                let output = tagged
                    .value
                    .as_str()
                    .ok_or("Expected string for !output tag")?;
                Ok(Value::String(result))
            }
            _ => Err(format!("Unknown tag: {}", tagged.tag)),
        },
        _ => Ok(value),
    }
}
