use alloy::{
    dyn_abi::{DynSolType, DynSolValue, JsonAbiExt},
    json_abi::Constructor,
    primitives::{Address, Bytes, FixedBytes, keccak256},
    providers::{Provider, network::TransactionBuilder},
    rpc::types::TransactionRequest,
};
use hex;
use deployer_core::{DeploymentData, VariableValue, VariableResolver};
use std::sync::Arc;

pub struct DeploymentExecutor<P> {
    provider: Arc<P>,
}

impl<P> DeploymentExecutor<P>
where
    P: Provider,
{
    pub fn new(provider: Arc<P>) -> Self {
        Self { provider }
    }

    pub async fn deploy<R: VariableResolver>(
        &self,
        data: &DeploymentData,
        resolver: &R,
    ) -> anyhow::Result<(Address, FixedBytes<32>)> {
        // Resolve the deployer address
        let deployer_address = data
            .address
            .resolve(DynSolType::Address, resolver)
            .map_err(|e| anyhow::anyhow!("Failed to resolve deployer address: {:?}", e))?
            .as_address()
            .ok_or_else(|| anyhow::anyhow!("Failed to resolve deployer address"))?;

        // Resolve salt
        let salt_value = data
            .salt
            .resolve(DynSolType::FixedBytes(32), resolver)
            .map_err(|e| anyhow::anyhow!("Failed to resolve salt: {:?}", e))?;
        let salt_bytes = salt_value
            .as_fixed_bytes()
            .ok_or_else(|| anyhow::anyhow!("Failed to resolve salt as bytes32"))?;
        let salt = FixedBytes::<32>::try_from(salt_bytes.0)
            .map_err(|_| anyhow::anyhow!("Invalid salt length"))?;

        // Resolve bytecode
        let bytecode_str = match &data.bytecode {
            VariableValue::Value(v) => v.clone(),
            VariableValue::Var(key) => {
                resolver.get_variable(key)
                    .map_err(|e| anyhow::anyhow!("Failed to resolve bytecode variable: {:?}", e))?
                    .as_str()
                    .ok_or_else(|| anyhow::anyhow!("Bytecode variable is not a string"))?
                    .to_string()
            }
            VariableValue::Output(id) => {
                resolver.get_output(id)
                    .map_err(|e| anyhow::anyhow!("Failed to resolve bytecode output: {:?}", e))?
                    .as_str()
                    .ok_or_else(|| anyhow::anyhow!("Bytecode output is not a string"))?
                    .to_string()
            }
            VariableValue::Data(path) => {
                resolver.get_data(path)
                    .map_err(|e| anyhow::anyhow!("Failed to resolve bytecode data: {:?}", e))?
                    .as_str()
                    .ok_or_else(|| anyhow::anyhow!("Bytecode data is not a string"))?
                    .to_string()
            }
        };
        
        let bytecode = Bytes::from(hex::decode(bytecode_str.trim_start_matches("0x"))?);

        // Parse constructor ABI and encode args
        let constructor: Constructor = data.constructor_abi_item.parse()?;
        let args: Vec<DynSolValue> = data
            .constructor_args
            .iter()
            .enumerate()
            .map(|(i, arg)| {
                let sol_type = DynSolType::parse(&constructor.inputs[i].ty)
                    .map_err(|e| anyhow::anyhow!("Invalid constructor arg type: {}", e))?;
                arg.resolve(sol_type, resolver)
                    .map_err(|e| anyhow::anyhow!("Failed to resolve constructor arg: {:?}", e))
            })
            .collect::<anyhow::Result<Vec<_>>>()?;

        // Encode constructor arguments
        let encoded_args = constructor.abi_encode_input(&args)?;
        
        // Combine bytecode with constructor args to create initcode
        let mut initcode = bytecode.to_vec();
        initcode.extend_from_slice(&encoded_args);
        let initcode = Bytes::from(initcode);

        // Calculate CREATE2 address
        let create2_address = self.calculate_create2_address(
            deployer_address,
            salt,
            &initcode
        );

        // Check if contract already exists at this address
        let code = self.provider.get_code_at(create2_address).await?;
        if !code.is_empty() {
            println!("Contract already deployed at: 0x{:x}", create2_address);
            // Return the existing address with a zero hash to indicate no new deployment
            return Ok((create2_address, FixedBytes::ZERO));
        }

        // Build CREATE2 deployment transaction
        let deployment_tx = self.build_create2_transaction(
            deployer_address,
            salt,
            initcode
        )?;

        // Send transaction
        let pending_tx = self.provider.send_transaction(deployment_tx).await?;
        let receipt = pending_tx.get_receipt().await?;
        
        if !receipt.status() {
            anyhow::bail!("Deployment transaction failed");
        }

        let tx_hash = receipt.transaction_hash;
        println!("Contract deployed at: 0x{:x}", create2_address);
        
        Ok((create2_address, tx_hash))
    }

    fn calculate_create2_address(
        &self,
        deployer: Address,
        salt: FixedBytes<32>,
        initcode: &Bytes,
    ) -> Address {
        let initcode_hash = keccak256(initcode);
        
        let mut data = Vec::with_capacity(1 + 20 + 32 + 32);
        data.push(0xff);
        data.extend_from_slice(deployer.as_slice());
        data.extend_from_slice(salt.as_slice());
        data.extend_from_slice(initcode_hash.as_slice());
        
        let hash = keccak256(&data);
        Address::from_slice(&hash[12..])
    }

    fn build_create2_transaction(
        &self,
        deployer: Address,
        salt: FixedBytes<32>,
        initcode: Bytes,
    ) -> anyhow::Result<TransactionRequest> {
        // For the deterministic deployment proxy, the calldata format is: salt + initcode
        let mut calldata = Vec::with_capacity(32 + initcode.len());
        calldata.extend_from_slice(salt.as_slice());
        calldata.extend_from_slice(&initcode);
        
        let tx = TransactionRequest::default()
            .with_to(deployer)
            .with_input(calldata);

        Ok(tx)
    }
}