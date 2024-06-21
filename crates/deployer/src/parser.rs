use alloy::{
    dyn_abi::{DynSolType, DynSolValue},
    json_abi::Param,
};
use anyhow::bail;
use regex::Regex;
use std::collections::HashMap;

// TODO make sure this works with dynamically sized arrays
#[derive(Debug)]
pub struct OutputCollector {
    output_data: HashMap<String, (DynSolType, DynSolValue)>,
}

impl OutputCollector {
    pub fn new() -> Self {
        Self {
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

        let mut name_stack = vec![];
        self.recurse_abi(id, output_definitions, outputs, &mut name_stack)?;
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

    fn recurse_abi(
        &mut self,
        level: String,
        output_definitions: Vec<Param>,
        outputs: Vec<DynSolValue>,
        name_stack: &mut Vec<String>,
    ) -> anyhow::Result<()> {
        for (i, output_def) in output_definitions.iter().enumerate() {
            let mut current_level = level.clone();
            if !output_def.components.is_empty() {
                current_level.push_str(".");
                self.recurse_abi(
                    current_level,
                    output_def.components.clone(),
                    outputs.clone(),
                    name_stack,
                )?;
            } else {
                current_level.push_str(&output_def.name);
                name_stack.push(current_level.clone());
            }

            if let Some(output) = outputs.get(i) {
                self.save_to_map(output, name_stack)?;
            }
        }
        Ok(())
    }

    fn save_to_map(
        &mut self,
        output: &DynSolValue,
        name_stack: &mut Vec<String>,
    ) -> anyhow::Result<()> {
        if let Some(t) = output.as_type() {
            match t {
                DynSolType::Tuple(_) => {
                    self.recurse_abi_outputs(
                        output.as_tuple().expect("should be tuple").to_vec(),
                        name_stack,
                    )?;
                }
                DynSolType::Array(_) => {
                    self.recurse_abi_outputs(
                        output.as_array().expect("should be array").to_vec(),
                        name_stack,
                    )?;
                }
                DynSolType::FixedArray(_, _) => {
                    self.recurse_abi_outputs(
                        output
                            .as_fixed_array()
                            .expect("Should be fixed array")
                            .to_vec(),
                        name_stack,
                    )?;
                }
                t => {
                    if let Some(name) = name_stack.pop() {
                        self.output_data.insert(name, (t, output.clone()));
                    }
                }
            }
        }
        Ok(())
    }

    fn recurse_abi_outputs(
        &mut self,
        outputs: Vec<DynSolValue>,
        name_stack: &mut Vec<String>,
    ) -> anyhow::Result<()> {
        for output in outputs {
            self.save_to_map(&output, name_stack)?;
        }
        Ok(())
    }
}
