use alloy::{
    dyn_abi::{DynSolType, DynSolValue},
    json_abi::Param,
};
use anyhow::bail;
use std::{collections::HashMap, str::FromStr};

#[derive(Debug)]
pub struct Indexer {
    output_data: HashMap<String, (DynSolType, DynSolValue)>,
    variables: HashMap<String, (DynSolType, DynSolValue)>,
}

impl Indexer {
    pub fn new() -> Indexer {
        Indexer {
            output_data: HashMap::new(),
            variables: HashMap::new(),
        }
    }

    pub fn get_variable_value(&self, key: &str) -> anyhow::Result<DynSolValue> {
        let (_data, val) = self
            .variables
            .get(key)
            .expect(&format!("No variable been indexed for {}", key));
        Ok(val.clone())
    }

    pub fn save_variable(&mut self, key: &str, ty: &str, value: &str) -> anyhow::Result<()> {
        let dyn_type = DynSolType::from_str(ty)?;
        let val = dyn_type.coerce_str(value)?;
        self.variables.insert(key.to_string(), (dyn_type, val));
        Ok(())
    }

    pub fn save_output_data(
        &mut self,
        id: String,
        output_definitions: Vec<Param>,
        outputs: Vec<DynSolValue>,
    ) -> anyhow::Result<()> {
        if output_definitions.len() != outputs.len() {
            return Err(anyhow::anyhow!("Mismatched input and output lengths"));
        }

        for (output_def, output) in output_definitions.into_iter().zip(outputs.into_iter()) {
            self.index_output(id.clone(), output_def, output)?;
        }

        Ok(())
    }

    pub fn get_output_value(&self, input_str: &str) -> anyhow::Result<DynSolValue> {
        let (_data, val) = self.output_data.get(input_str).expect(&format!(
            "No output data has been indexed for {}",
            input_str
        ));
        Ok(val.clone())
    }

    fn index_output(
        &mut self,
        prefix: String,
        output_def: Param,
        output: DynSolValue,
    ) -> anyhow::Result<()> {
        let current_level = if prefix.is_empty() {
            output_def.name.clone()
        } else {
            format!("{}.{}", prefix, output_def.name)
        };

        if !output_def.components.is_empty() {
            for (i, component_def) in output_def.components.into_iter().enumerate() {
                let component_value = match &output {
                    DynSolValue::Tuple(values) => values.get(i).unwrap().clone(),
                    _ => bail!("Expected tuple for components"),
                };
                self.index_output(current_level.clone(), component_def, component_value)?;
            }
        } else if let Some(t) = output.as_type() {
            match t {
                DynSolType::Array(_) | DynSolType::FixedArray(_, _) => {
                    let elements = match t {
                        DynSolType::Array(_) => output.as_array().unwrap().to_vec(),
                        DynSolType::FixedArray(_, _) => output.as_fixed_array().unwrap().to_vec(),
                        _ => unreachable!(),
                    };
                    for (index, element) in elements.into_iter().enumerate() {
                        self.index_output(
                            format!("{}[{}]", current_level, index),
                            output_def.clone(),
                            element,
                        )?;
                    }
                }
                _ => {
                    self.output_data.insert(current_level, (t, output));
                }
            }
        }
        Ok(())
    }
}
