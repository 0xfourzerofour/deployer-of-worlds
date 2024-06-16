use alloy::dyn_abi::{DynSolType, DynSolValue};
use regex::Regex;
use std::collections::HashMap;

#[derive(Debug)]
pub struct OutputCollector {
    output_data: HashMap<String, DynSolValue>,
}

impl OutputCollector {
    pub fn new() -> Self {
        Self {
            output_data: HashMap::new(),
        }
    }

    pub fn save_output_data(&mut self, outputs: Vec<DynSolValue>) {
        for output in outputs {
            if let Some(t) = output.as_type() {
                match t {
                    DynSolType::FixedBytes(size) => todo!(),
                    DynSolType::Bool => todo!(),
                    DynSolType::Int(size) => todo!(),
                    DynSolType::Uint(size) => println!("UINTTTT {:?}", size),
                    DynSolType::Tuple(types) => println!("TUPPLLLEE {:?}", types),
                    DynSolType::String => todo!(),
                    DynSolType::Bytes => todo!(),
                    DynSolType::Array(array_type) => todo!(),
                    DynSolType::FixedArray(array_type, size) => todo!(),
                    DynSolType::Address => todo!(),
                    DynSolType::Function => todo!(),
                }
            }
        }
    }

    pub fn get_input_value(&self, input_str: String) -> anyhow::Result<String> {
        let _re = Regex::new(r"\$\{(\w+)\}")?;

        Ok(input_str)
    }
}
