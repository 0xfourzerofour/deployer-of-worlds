use alloy::{
    json_abi::{JsonAbi, Param},
    primitives::Bytes,
};

pub fn generate_initcode(
    abi: JsonAbi,
    bytecode: Bytes,
    constructor_args: Vec<Param>,
) -> anyhow::Result<Bytes> {
    if let Some(constructor) = abi.constructor {
        if constructor_args.len() != constructor.inputs.len() {
            anyhow::bail!("Input length an constructor args length are not the same");
        }
    }

    Ok(Bytes::new())
}
