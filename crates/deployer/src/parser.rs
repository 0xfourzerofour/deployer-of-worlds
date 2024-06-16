use std::collections::HashMap;

pub struct OutputCollector {
    output_data: HashMap<String, (String, String)>,
}

pub enum OutputDataType {
    Bytes,
    U256,
    U32,
    Address,
}

impl OutputCollector {
    pub fn new() -> Self {
        Self {
            output_data: HashMap::new(),
        }
    }

    pub fn save_output_data(&mut self, output: String) {}
}
