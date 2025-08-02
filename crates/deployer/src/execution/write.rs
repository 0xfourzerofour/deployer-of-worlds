use alloy::{
    dyn_abi::{DynSolType, DynSolValue, JsonAbiExt},
    json_abi::Function,
    primitives::Bytes,
    providers::{network::TransactionBuilder, Provider},
    rpc::types::TransactionRequest,
};
use deployer_core::{VariableResolver, WriteData};
use std::sync::Arc;

pub struct WriteExecutor<P> {
    provider: Arc<P>,
}

impl<P> WriteExecutor<P>
where
    P: Provider,
{
    pub fn new(provider: Arc<P>) -> Self {
        Self { provider }
    }

    pub async fn write<R: VariableResolver>(
        &self,
        data: &WriteData,
        resolver: &R,
    ) -> anyhow::Result<()> {
        // // Check condition if present
        // if let Some(condition) = &data.condition {
        //     let should_execute = condition.evaluate(&self.provider, resolver).await
        //         .map_err(|e| anyhow::anyhow!("Failed to evaluate condition: {:?}", e))?;

        //     if !should_execute {
        //         println!("Condition not met, skipping write action");
        //         return Ok(());
        //     }

        //     println!("Condition met, executing write action");
        // }
        let function: Function = data.abi_item.parse()?;
        let address = data
            .address
            .resolve(DynSolType::Address, resolver)
            .map_err(|e| anyhow::anyhow!("Failed to resolve address: {:?}", e))?
            .as_address()
            .ok_or_else(|| anyhow::anyhow!("Failed to resolve address"))?;

        let value = data
            .value
            .resolve(DynSolType::Uint(256), resolver)
            .map_err(|e| anyhow::anyhow!("Failed to resolve value: {:?}", e))?
            .as_uint()
            .ok_or_else(|| anyhow::anyhow!("Failed to resolve value as uint256"))?;

        let args: Vec<DynSolValue> = data
            .args
            .iter()
            .enumerate()
            .map(|(i, arg)| {
                let sol_type = DynSolType::parse(&function.inputs[i].ty)
                    .map_err(|e| anyhow::anyhow!("Invalid type: {}", e))?;
                arg.resolve(sol_type, resolver)
                    .map_err(|e| anyhow::anyhow!("Failed to resolve write arg: {:?}", e))
            })
            .collect::<anyhow::Result<Vec<_>>>()?;

        let input = function.abi_encode_input(&args)?;

        let tx = TransactionRequest::default()
            .with_to(address)
            .with_value(value.0)
            .with_input(Bytes::from(input));

        let pending_tx = self.provider.send_transaction(tx).await?;
        let receipt = pending_tx.get_receipt().await?;

        println!("{:?}", receipt);

        Ok(())
    }
}
