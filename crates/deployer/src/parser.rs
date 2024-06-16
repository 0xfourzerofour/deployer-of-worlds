use regex::Regex;
use std::collections::HashMap;

pub struct OutputCollector {
    output_data: HashMap<String, (OutputDataType, String)>,
}

pub enum OutputDataType {
    Address,
    Bool,
    Bytes,
    String,
    UInt8,
    UInt16,
    UInt32,
    UInt64,
    UInt128,
    UInt256,
    Int8,
    Int16,
    Int32,
    Int64,
    Int128,
    Int256,
}

impl OutputCollector {
    pub fn new() -> Self {
        Self {
            output_data: HashMap::new(),
        }
    }

    pub fn save_output_data(&mut self, output: String) {}

    pub fn get_input_value(&self, input_str: String) -> anyhow::Result<String> {
        let re = Regex::new(r"\$\{(\w+)\}")?;

        if let Some(value) = re.find(&input_str) {
            let query = value.as_str();

            let (_output_type, str_encoded_val) = self
                .output_data
                .get(query)
                .expect("Data should be available at query index");

            return Ok(str_encoded_val.to_owned());
        }

        Ok(input_str)
    }
}
