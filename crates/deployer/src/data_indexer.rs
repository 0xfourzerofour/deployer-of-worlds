use crate::indexer::Indexer;
use alloy::dyn_abi::DynSolValue;
use deployer_core::{DataReference, FileDataResolver, HierarchicalVariableResolver, Result, Variable, VariableResolver};
use std::collections::HashMap;
use std::path::PathBuf;

/// Enhanced indexer that supports data references in addition to variables and outputs
#[derive(Debug)]
pub struct DataIndexer {
    /// Base indexer for variables and outputs
    base_indexer: Indexer,
    /// Data references from the config
    data_refs: HashMap<String, DataReference>,
    /// Local variables from the config
    local_variables: HashMap<String, Variable>,
    /// Data resolver for loading external files
    data_resolver: FileDataResolver,
}

impl DataIndexer {
    pub fn new(
        data_refs: HashMap<String, DataReference>,
        local_variables: HashMap<String, Variable>,
        data_dir: PathBuf,
    ) -> Self {
        Self {
            base_indexer: Indexer::new(),
            data_refs,
            local_variables,
            data_resolver: FileDataResolver::new(data_dir),
        }
    }

    /// Get the base indexer for direct access
    pub fn base_indexer(&self) -> &Indexer {
        &self.base_indexer
    }

    /// Get mutable reference to base indexer
    pub fn base_indexer_mut(&mut self) -> &mut Indexer {
        &mut self.base_indexer
    }

    /// Save variable to the base indexer
    pub fn save_variable(&mut self, key: &str, ty: &str, value: &str) -> anyhow::Result<()> {
        self.base_indexer.save_variable(key, ty, value)
    }

    /// Save output data to the base indexer
    pub fn save_output_data(
        &mut self,
        id: String,
        output_definitions: Vec<alloy::json_abi::Param>,
        outputs: Vec<DynSolValue>,
    ) -> anyhow::Result<()> {
        self.base_indexer.save_output_data(id, output_definitions, outputs)
    }
}

impl VariableResolver for DataIndexer {
    fn get_variable(&self, key: &str) -> Result<DynSolValue> {
        // Create a hierarchical resolver that checks local variables first,
        // then shared variables from data references, then base indexer
        let hierarchical_resolver = HierarchicalVariableResolver::new(
            &self.local_variables,
            &self.data_refs,
            &self.data_resolver,
        );

        // Try hierarchical resolver first (local and shared variables)
        match hierarchical_resolver.get_variable(key) {
            Ok(value) => Ok(value),
            Err(_) => {
                // Fall back to base indexer (for dynamically saved variables)
                self.base_indexer.get_variable(key)
            }
        }
    }

    fn get_output(&self, id: &str) -> Result<DynSolValue> {
        // Outputs are only in the base indexer
        self.base_indexer.get_output(id)
    }

    fn get_data(&self, path: &str) -> Result<DynSolValue> {
        // Use hierarchical resolver for data references
        let hierarchical_resolver = HierarchicalVariableResolver::new(
            &self.local_variables,
            &self.data_refs,
            &self.data_resolver,
        );
        
        hierarchical_resolver.get_data(path)
    }
}