use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::errors::{DeployerError, Result};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractData {
    pub name: String,
    pub bytecode: String,
    pub abi: serde_json::Value,
    pub constructor: Option<String>,
    #[serde(default)]
    pub metadata: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VariableData {
    #[serde(flatten)]
    pub variables: HashMap<String, crate::Variable>,
}

/// Reference to external data files
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum DataReference {
    /// Reference to a contract file: contracts/MyContract.json
    Contract { path: String },
    /// Reference to shared variables: variables/mainnet.yml
    Variables { path: String },
    /// Reference to raw data: data/config.json
    Raw { path: String },
}

/// Enhanced pipeline configuration with data references and local variables
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineConfig {
    /// Local variables specific to this pipeline
    #[serde(default)]
    pub variables: HashMap<String, crate::Variable>,
    
    /// References to external data sources (shared variables, contracts, etc.)
    #[serde(default)]
    pub data: HashMap<String, DataReference>,
    
    /// Actions to execute
    pub actions: Vec<crate::Action>,
    
    /// Optional metadata
    #[serde(default)]
    pub metadata: HashMap<String, serde_json::Value>,
}

pub trait DataResolver {
    fn get_contract_data(&self, path: &str) -> Result<ContractData>;
    fn get_variable_data(&self, path: &str) -> Result<VariableData>;
    fn get_raw_data(&self, path: &str) -> Result<serde_json::Value>;
    fn get_raw_contract_data(&self, path: &str) -> Result<serde_json::Value>;
}

#[derive(Debug)]
pub struct FileDataResolver {
    data_dir: std::path::PathBuf,
}

impl FileDataResolver {
    pub fn new(data_dir: impl Into<std::path::PathBuf>) -> Self {
        Self {
            data_dir: data_dir.into(),
        }
    }

    pub fn resolve_path(&self, path: &str) -> Result<std::path::PathBuf> {
        let full_path = if path.ends_with(".json") || path.ends_with(".yml") || path.ends_with(".yaml") {
            self.data_dir.join(path)
        } else {
            // Try different extensions
            let json_path = self.data_dir.join(format!("{}.json", path));
            let yml_path = self.data_dir.join(format!("{}.yml", path));
            let yaml_path = self.data_dir.join(format!("{}.yaml", path));
            
            if json_path.exists() {
                json_path
            } else if yml_path.exists() {
                yml_path
            } else if yaml_path.exists() {
                yaml_path
            } else {
                return Err(DeployerError::Config(format!("Data file not found: {}", path)));
            }
        };

        if !full_path.exists() {
            return Err(DeployerError::Config(format!("Data file not found: {}", full_path.display())));
        }

        Ok(full_path)
    }

    pub fn load_file(&self, path: &std::path::Path) -> Result<serde_json::Value> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| DeployerError::Config(format!("Failed to read file {}: {}", path.display(), e)))?;

        if path.extension().and_then(|s| s.to_str()) == Some("json") {
            serde_json::from_str(&content)
                .map_err(|e| DeployerError::Config(format!("Invalid JSON in {}: {}", path.display(), e)))
        } else {
            // YAML file
            let yaml_value: serde_yaml::Value = serde_yaml::from_str(&content)
                .map_err(|e| DeployerError::Config(format!("Invalid YAML in {}: {}", path.display(), e)))?;
            
            // Convert YAML to JSON Value
            serde_json::to_value(yaml_value)
                .map_err(|e| DeployerError::Config(format!("Failed to convert YAML to JSON: {}", e)))
        }
    }
}

impl DataResolver for FileDataResolver {
    fn get_contract_data(&self, path: &str) -> Result<ContractData> {
        let file_path = self.resolve_path(path)?;
        let json_value = self.load_file(&file_path)?;
        
        // Handle standard contract JSON format where bytecode is nested
        let bytecode = if let Some(bytecode_obj) = json_value.get("bytecode") {
            if let Some(bytecode_str) = bytecode_obj.get("object") {
                bytecode_str.as_str()
                    .ok_or_else(|| DeployerError::Config("bytecode.object must be a string".to_string()))?
                    .to_string()
            } else if let Some(bytecode_str) = bytecode_obj.as_str() {
                // Handle case where bytecode is directly a string
                bytecode_str.to_string()
            } else {
                return Err(DeployerError::Config("Invalid bytecode format in contract JSON".to_string()));
            }
        } else {
            return Err(DeployerError::Config("No bytecode field in contract JSON".to_string()));
        };
        
        let abi = json_value.get("abi")
            .ok_or_else(|| DeployerError::Config("No abi field in contract JSON".to_string()))?
            .clone();
            
        let name = file_path.file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("Unknown")
            .to_string();
            
        Ok(ContractData {
            name,
            bytecode,
            abi,
            constructor: None,
            metadata: HashMap::new(),
        })
    }

    fn get_variable_data(&self, path: &str) -> Result<VariableData> {
        let file_path = self.resolve_path(path)?;
        let json_value = self.load_file(&file_path)?;
        
        serde_json::from_value(json_value)
            .map_err(|e| DeployerError::Config(format!("Invalid variable data format in {}: {}", path, e)))
    }

    fn get_raw_data(&self, path: &str) -> Result<serde_json::Value> {
        let file_path = self.resolve_path(path)?;
        self.load_file(&file_path)
    }
    
    fn get_raw_contract_data(&self, path: &str) -> Result<serde_json::Value> {
        // Same as get_raw_data for file-based resolver
        self.get_raw_data(path)
    }
}

/// Variable resolver that handles both local and shared variables with proper precedence
pub struct HierarchicalVariableResolver<'a, R: DataResolver> {
    /// Local variables (highest precedence)
    local_variables: &'a HashMap<String, crate::Variable>,
    /// Data references for shared variables
    data_refs: &'a HashMap<String, DataReference>,
    /// Data resolver for loading external data
    data_resolver: &'a R,
    /// Cache for loaded shared variables
    shared_variables_cache: std::cell::RefCell<HashMap<String, VariableData>>,
}

impl<'a, R: DataResolver> HierarchicalVariableResolver<'a, R> {
    pub fn new(
        local_variables: &'a HashMap<String, crate::Variable>,
        data_refs: &'a HashMap<String, DataReference>,
        data_resolver: &'a R,
    ) -> Self {
        Self {
            local_variables,
            data_refs,
            data_resolver,
            shared_variables_cache: std::cell::RefCell::new(HashMap::new()),
        }
    }

    fn load_shared_variables(&self, data_ref_key: &str) -> Result<VariableData> {
        let mut cache = self.shared_variables_cache.borrow_mut();
        
        if let Some(cached) = cache.get(data_ref_key) {
            return Ok(cached.clone());
        }

        let data_ref = self.data_refs.get(data_ref_key)
            .ok_or_else(|| DeployerError::Config(format!("Data reference not found: {}", data_ref_key)))?;

        let variable_data = match data_ref {
            DataReference::Variables { path } => {
                self.data_resolver.get_variable_data(path)?
            }
            _ => {
                return Err(DeployerError::Config(format!("Data reference '{}' is not a variables type", data_ref_key)));
            }
        };

        cache.insert(data_ref_key.to_string(), variable_data.clone());
        Ok(variable_data)
    }
}

impl<'a, R: DataResolver> crate::VariableResolver for HierarchicalVariableResolver<'a, R> {
    fn get_variable(&self, key: &str) -> Result<alloy::dyn_abi::DynSolValue> {
        // 1. Check local variables first (highest precedence)
        if let Some(var) = self.local_variables.get(key) {
            let sol_type = alloy::dyn_abi::DynSolType::parse(&var.ty)
                .map_err(|_e| DeployerError::TypeConversion {
                    expected: var.ty.clone(),
                    actual: format!("parse error"),
                })?;
            return sol_type.coerce_str(&var.value)
                .map_err(|_e| DeployerError::TypeConversion {
                    expected: var.ty.clone(),
                    actual: var.value.clone(),
                });
        }

        // 2. Check shared variables from data references
        for (data_ref_key, _) in self.data_refs.iter() {
            if let Ok(shared_vars) = self.load_shared_variables(data_ref_key) {
                if let Some(var) = shared_vars.variables.get(key) {
                    let sol_type = alloy::dyn_abi::DynSolType::parse(&var.ty)
                        .map_err(|_e| DeployerError::TypeConversion {
                            expected: var.ty.clone(),
                            actual: format!("parse error"),
                        })?;
                    return sol_type.coerce_str(&var.value)
                        .map_err(|_e| DeployerError::TypeConversion {
                            expected: var.ty.clone(),
                            actual: var.value.clone(),
                        });
                }
            }
        }

        Err(DeployerError::VariableNotFound(key.to_string()))
    }

    fn get_output(&self, id: &str) -> Result<alloy::dyn_abi::DynSolValue> {
        // Output resolution would be handled by the indexer
        // This is just a placeholder implementation
        Err(DeployerError::OutputNotFound(id.to_string()))
    }

    fn get_data(&self, path: &str) -> Result<alloy::dyn_abi::DynSolValue> {
        // Parse the path like "contract.bytecode" or "contract.abi[0].name" etc
        let (data_ref_key, field_path) = path.split_once('.')
            .ok_or_else(|| DeployerError::Config(format!("Invalid data path format: {}", path)))?;

        let data_ref = self.data_refs.get(data_ref_key)
            .ok_or_else(|| DeployerError::Config(format!("Data reference not found: {}", data_ref_key)))?;

        match data_ref {
            DataReference::Contract { path: contract_path } => {
                // Get the raw contract JSON
                let contract_json = self.data_resolver.get_raw_contract_data(contract_path)?;
                
                // Navigate through the JSON using the field path
                let value = navigate_json_path(&contract_json, field_path)?;
                
                // Convert JSON value to DynSolValue
                json_to_dyn_sol_value(value)
            }
            DataReference::Variables { path: vars_path } => {
                let var_data = self.data_resolver.get_variable_data(vars_path)?;
                if let Some(var) = var_data.variables.get(field_path) {
                    let sol_type = alloy::dyn_abi::DynSolType::parse(&var.ty)
                        .map_err(|_e| DeployerError::TypeConversion {
                            expected: var.ty.clone(),
                            actual: format!("parse error"),
                        })?;
                    sol_type.coerce_str(&var.value)
                        .map_err(|_e| DeployerError::TypeConversion {
                            expected: var.ty.clone(),
                            actual: var.value.clone(),
                        })
                } else {
                    Err(DeployerError::VariableNotFound(field_path.to_string()))
                }
            }
            DataReference::Raw { path: raw_path } => {
                let raw_data = self.data_resolver.get_raw_data(raw_path)?;
                let value = navigate_json_path(&raw_data, field_path)?;
                json_to_dyn_sol_value(value)
            }
        }
    }
}

/// Navigate through a JSON value using a dot-separated path
/// Supports nested objects and array indexing with brackets
fn navigate_json_path<'a>(value: &'a serde_json::Value, path: &str) -> Result<&'a serde_json::Value> {
    let mut current = value;
    let parts = path.split('.');
    
    for part in parts {
        if part.is_empty() {
            continue;
        }
        
        // Check if this part contains array indexing
        if let Some(bracket_pos) = part.find('[') {
            let key = &part[..bracket_pos];
            
            // Navigate to the object field first
            if !key.is_empty() {
                current = current.get(key)
                    .ok_or_else(|| DeployerError::Config(format!("Field '{}' not found", key)))?;
            }
            
            // Handle array indexing
            let remaining = &part[bracket_pos..];
            for array_part in remaining.split('[').filter(|s| !s.is_empty()) {
                let index_str = array_part.trim_end_matches(']');
                let index: usize = index_str.parse()
                    .map_err(|_| DeployerError::Config(format!("Invalid array index: {}", index_str)))?;
                
                current = current.get(index)
                    .ok_or_else(|| DeployerError::Config(format!("Array index {} out of bounds", index)))?;
            }
        } else {
            // Simple field access
            current = current.get(part)
                .ok_or_else(|| DeployerError::Config(format!("Field '{}' not found", part)))?;
        }
    }
    
    Ok(current)
}

/// Convert a JSON value to DynSolValue
fn json_to_dyn_sol_value(value: &serde_json::Value) -> Result<alloy::dyn_abi::DynSolValue> {
    match value {
        serde_json::Value::String(s) => {
            // Try to detect the appropriate type based on the string content
            // First check if it's a hex string that could be bytes or address
            if s.starts_with("0x") {
                let hex_data = &s[2..];
                // Check if it's a valid address (40 hex chars)
                if hex_data.len() == 40 {
                    match alloy::dyn_abi::DynSolType::Address.coerce_str(s) {
                        Ok(val) => return Ok(val),
                        Err(_) => {} // Fall through to try as bytes
                    }
                }
                // Try as bytes
                match alloy::dyn_abi::DynSolType::Bytes.coerce_str(s) {
                    Ok(val) => return Ok(val),
                    Err(_) => {} // Fall through to string
                }
            }
            // Default to string
            alloy::dyn_abi::DynSolType::String.coerce_str(s)
                .map_err(|_| DeployerError::Config("Failed to convert string value".to_string()))
        }
        serde_json::Value::Number(n) => {
            // Convert number to string and parse as uint256
            let num_str = n.to_string();
            alloy::dyn_abi::DynSolType::Uint(256).coerce_str(&num_str)
                .map_err(|_| DeployerError::Config(format!("Failed to convert number: {}", num_str)))
        }
        serde_json::Value::Bool(b) => {
            Ok(alloy::dyn_abi::DynSolValue::Bool(*b))
        }
        serde_json::Value::Array(arr) => {
            // For now, convert array to JSON string
            // In the future, we could try to detect the array element type
            let json_str = serde_json::to_string(arr)
                .map_err(|e| DeployerError::Config(format!("Failed to serialize array: {}", e)))?;
            alloy::dyn_abi::DynSolType::String.coerce_str(&json_str)
                .map_err(|_| DeployerError::Config("Failed to convert array to string".to_string()))
        }
        serde_json::Value::Object(obj) => {
            // For objects, check if it has special structure (like bytecode object)
            if let Some(bytecode_obj) = obj.get("object") {
                if let Some(bytecode_str) = bytecode_obj.as_str() {
                    return alloy::dyn_abi::DynSolType::Bytes.coerce_str(bytecode_str)
                        .map_err(|_| DeployerError::Config("Failed to convert bytecode object".to_string()));
                }
            }
            
            // Otherwise, convert to JSON string
            let json_str = serde_json::to_string(obj)
                .map_err(|e| DeployerError::Config(format!("Failed to serialize object: {}", e)))?;
            alloy::dyn_abi::DynSolType::String.coerce_str(&json_str)
                .map_err(|_| DeployerError::Config("Failed to convert object to string".to_string()))
        }
        serde_json::Value::Null => {
            Err(DeployerError::Config("Cannot convert null value to DynSolValue".to_string()))
        }
    }
}