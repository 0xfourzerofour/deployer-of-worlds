use deployer_core::{Action, Variable, DataReference, PipelineConfig};
use serde::Deserialize;
use std::{collections::HashMap, fs};


#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub variables: HashMap<String, Variable>,
    #[serde(default)]
    pub data: HashMap<String, DataReference>,
    pub actions: Vec<Action>,
}

impl Config {
    pub fn load_from_file(path: &str) -> anyhow::Result<Config> {
        let contents = fs::read_to_string(path)?;
        
        // First try to parse as PipelineConfig (which includes data references)
        if let Ok(pipeline_config) = serde_yaml::from_str::<PipelineConfig>(&contents) {
            return Ok(Config {
                variables: pipeline_config.variables,
                data: pipeline_config.data,
                actions: pipeline_config.actions,
            });
        }
        
        // Fall back to direct Config parsing (for backwards compatibility)
        let config: Config = serde_yaml::from_str(&contents)?;
        Ok(config)
    }

    pub fn new() -> Config {
        Config {
            variables: HashMap::new(),
            data: HashMap::new(),
            actions: Vec::new(),
        }
    }
}

