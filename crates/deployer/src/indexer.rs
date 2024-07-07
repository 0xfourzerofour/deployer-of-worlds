use alloy::{
    dyn_abi::{DynSolType, DynSolValue},
    json_abi::Param,
};
use anyhow::bail;
use regex::Regex;
use std::collections::HashMap;

#[derive(Debug)]
pub struct Indexer {
    output_data: HashMap<String, (DynSolType, DynSolValue)>,
}

impl Indexer {
    pub fn new() -> Indexer {
        Indexer {
            output_data: HashMap::new(),
        }
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

        println!("{:?}", self.output_data);
        Ok(())
    }

    pub fn get_input_value(
        &self,
        input_str: &str,
        input_type: DynSolType,
    ) -> anyhow::Result<DynSolValue> {
        let re = Regex::new(r"\$\{([^\}]+)\}")?;
        if let Some(captures) = re.captures(&input_str) {
            let inner_string = captures.get(1).unwrap().as_str();
            if let Some((_t, v)) = self.output_data.get(inner_string) {
                return Ok(v.clone());
            } else {
                bail!("Unable to find value based on input key {:?}", captures);
            }
        }
        let coerced = input_type.coerce_str(input_str)?;
        Ok(coerced)
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
