use deployer_core::{Action, Variable};
use serde::Deserialize;
use std::{collections::HashMap, fs};


#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub variables: HashMap<String, Variable>,
    pub actions: Vec<Action>,
}

impl Config {
    pub fn load_from_file(path: &str) -> anyhow::Result<Config> {
        let contents = fs::read_to_string(path)?;
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

