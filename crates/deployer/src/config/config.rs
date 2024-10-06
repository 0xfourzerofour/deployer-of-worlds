use alloy::dyn_abi::{DynSolType, DynSolValue};
use serde::{de::Error, Deserialize, Deserializer};
use serde_yaml::Value as YamlValue;
use std::{collections::HashMap, fs};

use crate::indexer::Indexer;

#[derive(Debug, Clone, Deserialize)]
pub struct ConfigVariable {
    pub ty: String,
    pub value: String,
}

#[derive(Debug, Clone)]
pub enum VariableValue {
    Var(String),
    Output(String),
    Value(String),
}

impl VariableValue {
    pub fn resolve(
        &self,
        expected_type: DynSolType,
        indexer: &Indexer,
    ) -> anyhow::Result<DynSolValue> {
        match self {
            VariableValue::Var(key) => indexer.get_variable_value(key),
            VariableValue::Output(id) => indexer.get_output_value(id),
            VariableValue::Value(value) => expected_type
                .coerce_str(value)
                .map_err(|e| anyhow::Error::new(e)),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type", content = "content", rename_all = "snake_case")]
pub enum ActionData {
    Deploy(DeploymentData),
    Write(WriteData),
    Read(ReadData),
}

#[derive(Debug, Clone, Deserialize)]
pub struct DeploymentData {
    pub address: VariableValue,
    pub constructor_args: Vec<VariableValue>,
    pub salt: VariableValue,
    pub constructor_abi_item: String,
    pub bytecode: VariableValue,
}

#[derive(Debug, Clone, Deserialize)]
pub struct WriteData {
    pub address: VariableValue,
    pub abi_item: String,
    pub args: Vec<VariableValue>,
    pub value: VariableValue,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ReadData {
    pub address: VariableValue,
    pub args: Vec<VariableValue>,
    pub abi_item: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ConfigAction {
    pub depends_on: Option<Vec<String>>,
    pub id: String,
    pub action_data: ActionData,
}

#[derive(Debug, Deserialize)]
pub struct Config {
    pub variables: HashMap<String, ConfigVariable>,
    pub actions: Vec<ConfigAction>,
}

impl Config {
    pub fn load_from_file(path: &str) -> anyhow::Result<Config> {
        let contents = fs::read_to_string(path).expect("Should have been able to read the file");
        let config: Config = serde_yaml::from_str(&contents)?;

        Ok(config)
    }

    pub fn new() -> Config {
        Config {
            variables: HashMap::new(),
            actions: Vec::new(),
        }
    }
}

impl<'de> Deserialize<'de> for VariableValue {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
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
                _ => Err(Error::custom(format!("Unknown tag: {}", tagged.tag))),
            },
            YamlValue::String(s) => Ok(VariableValue::Value(s)),
            _ => Err(Error::custom("Expected string or tagged value")),
        }
    }
}
