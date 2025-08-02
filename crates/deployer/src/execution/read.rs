use alloy::{
    contract::CallBuilder,
    dyn_abi::{DynSolType, DynSolValue, JsonAbiExt},
    json_abi::Function,
    primitives::Bytes,
    providers::Provider,
};
use deployer_core::{ReadData, VariableResolver};
use std::sync::Arc;

pub struct ReadExecutor<P> {
    provider: Arc<P>,
}

impl<P> ReadExecutor<P>
where
    P: Provider,
{
    pub fn new(provider: Arc<P>) -> Self {
        Self { provider }
    }

    pub async fn read<R: VariableResolver>(
        &self,
        data: &ReadData,
        resolver: &R,
    ) -> anyhow::Result<Vec<DynSolValue>> {
        let function: Function = data.abi_item.parse()?;
        let address = data
            .address
            .resolve(DynSolType::Address, resolver)
            .map_err(|e| anyhow::anyhow!("Failed to resolve address: {:?}", e))?
            .as_address()
            .ok_or_else(|| anyhow::anyhow!("Failed to resolve address"))?;

        let args: Vec<DynSolValue> = data
            .args
            .iter()
            .enumerate()
            .map(|(i, arg)| {
                let sol_type = DynSolType::parse(&function.inputs[i].ty)
                    .map_err(|e| anyhow::anyhow!("Invalid type: {}", e))?;
                arg.resolve(sol_type, resolver)
                    .map_err(|e| anyhow::anyhow!("Failed to resolve read arg: {:?}", e))
            })
            .collect::<anyhow::Result<Vec<_>>>()?;

        let input = function.abi_encode_input(&args)?;

        // Check if contract has bytecode
        let code = self.provider.get_code_at(address).await?;
        if code.is_empty() {
            anyhow::bail!("No contract deployed at address 0x{:x}", address);
        }

        let read_output = CallBuilder::new_raw(self.provider.clone(), Bytes::from(input))
            .to(address)
            .call_raw()
            .with_decoder(&function)
            .await?;

        println!("output {:?}", read_output);

        Ok(read_output)
    }
}
