use alloy::{
    dyn_abi::{DynSolType, DynSolValue},
    json_abi::Param,
};
use regex::Regex;
use std::collections::{HashMap, VecDeque};

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
        output_defintions: Vec<Param>,
        outputs: Vec<DynSolValue>,
    ) {
        if output_defintions.len() != outputs.len() {
            panic!("Something is wrong with the inputs and outputs");
        }

        let mut name_queue = VecDeque::new();
        recurse_abi_definition(id, output_defintions, &mut name_queue);
        recurse_abi_outputs(&mut self.output_data, outputs, &mut name_queue);
    }

    pub fn get_input_value(&self, input_str: String) -> anyhow::Result<String> {
        let _re = Regex::new(r"\$\{(\w+)\}")?;

        Ok(input_str)
    }
}

// TODO use only one recursive function so that we can parse dynamicly size
// arrays without needed to do any funky shit
fn recurse_abi_outputs(
    map: &mut HashMap<String, (DynSolType, DynSolValue)>,
    outputs: Vec<DynSolValue>,
    queue: &mut VecDeque<String>,
) {
    for output in outputs {
        if let Some(t) = output.as_type() {
            match t {
                DynSolType::Tuple(_) => {
                    recurse_abi_outputs(
                        map,
                        output.as_tuple().expect("Should be tuple").to_vec(),
                        queue,
                    );
                }
                DynSolType::Array(_) => {
                    recurse_abi_outputs(
                        map,
                        output.as_array().expect("Should be array").to_vec(),
                        queue,
                    );
                }
                DynSolType::FixedArray(_, _) => {
                    recurse_abi_outputs(
                        map,
                        output
                            .as_fixed_array()
                            .expect("Should be fixed array")
                            .to_vec(),
                        queue,
                    );
                }
                t => {
                    let name = queue.pop_back().unwrap();
                    map.insert(name, (t, output));
                }
            }
        }
    }
}

fn recurse_abi_definition(
    level: String,
    output_definition: Vec<Param>,
    queue: &mut VecDeque<String>,
) {
    for output in output_definition {
        let mut lev = level.clone();
        if !output.components.is_empty() {
            lev.push_str(".");
            recurse_abi_definition(lev, output.components, queue);
        } else {
            lev.push_str(&output.name);
            queue.push_front(lev.clone());
        }
    }
}
