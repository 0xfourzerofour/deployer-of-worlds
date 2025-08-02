use alloy::{
    dyn_abi::{DynSolType, DynSolValue, JsonAbiExt},
    json_abi::Constructor,
    primitives::{keccak256, Address, Bytes, FixedBytes},
    providers::{network::TransactionBuilder, Provider},
    rpc::types::TransactionRequest,
};
use deployer_core::{DeploymentData, VariableResolver, VariableValue};
use hex;
use std::sync::Arc;

// Standard CREATE2 deployer address (deterministic deployment proxy)
const CREATE2_DEPLOYER: Address = Address::new([
    0x4e, 0x59, 0xb4, 0x48, 0x47, 0xb3, 0x79, 0x57, 0x85, 0x88,
    0x92, 0x0c, 0xa7, 0x8f, 0xbf, 0x26, 0xc0, 0xb4, 0x95, 0x6c
]);

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
        // Resolve the expected deployment address
        let expected_address = data
            .address
            .resolve(DynSolType::Address, resolver)
            .map_err(|e| anyhow::anyhow!("Failed to resolve expected address: {:?}", e))?
            .as_address()
            .ok_or_else(|| anyhow::anyhow!("Failed to resolve expected address"))?;

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
        let bytecode = match &data.bytecode {
            VariableValue::Value(v) => {
                // For direct string values, decode from hex
                Bytes::from(hex::decode(v.trim_start_matches("0x"))?)
            }
            VariableValue::Var(key) => {
                let bytecode_value = resolver
                    .get_variable(key)
                    .map_err(|e| anyhow::anyhow!("Failed to resolve bytecode variable: {:?}", e))?;

                // Check if it's already bytes
                if let Some(bytes) = bytecode_value.as_bytes() {
                    Bytes::from(bytes.to_vec())
                } else if let Some(str_val) = bytecode_value.as_str() {
                    // If it's a string, decode from hex
                    Bytes::from(hex::decode(str_val.trim_start_matches("0x"))?)
                } else {
                    anyhow::bail!("Bytecode variable must be bytes or hex string")
                }
            }
            VariableValue::Output(id) => {
                let bytecode_value = resolver
                    .get_output(id)
                    .map_err(|e| anyhow::anyhow!("Failed to resolve bytecode output: {:?}", e))?;

                if let Some(bytes) = bytecode_value.as_bytes() {
                    Bytes::from(bytes.to_vec())
                } else if let Some(str_val) = bytecode_value.as_str() {
                    Bytes::from(hex::decode(str_val.trim_start_matches("0x"))?)
                } else {
                    anyhow::bail!("Bytecode output must be bytes or hex string")
                }
            }
            VariableValue::Data(path) => {
                let bytecode_value = resolver
                    .get_data(path)
                    .map_err(|e| anyhow::anyhow!("Failed to resolve bytecode data: {:?}", e))?;

                if let Some(bytes) = bytecode_value.as_bytes() {
                    Bytes::from(bytes.to_vec())
                } else if let Some(str_val) = bytecode_value.as_str() {
                    Bytes::from(hex::decode(str_val.trim_start_matches("0x"))?)
                } else {
                    anyhow::bail!("Bytecode data must be bytes or hex string")
                }
            }
        };

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

        let encoded_args = constructor.abi_encode_input(&args)?;

        let mut initcode = bytecode.to_vec();
        initcode.extend_from_slice(&encoded_args);
        let initcode = Bytes::from(initcode);
        
        // Calculate the CREATE2 address using the standard deployer
        let create2_address = self.calculate_create2_address(CREATE2_DEPLOYER, salt, &initcode);
        
        // Verify that the computed address matches the expected address
        if create2_address != expected_address {
            anyhow::bail!(
                "CREATE2 address mismatch! Expected: 0x{:x}, Computed: 0x{:x}",
                expected_address,
                create2_address
            );
        }

        // Check if contract already exists at this address
        let code = self.provider.get_code_at(create2_address).await?;
        if !code.is_empty() {
            println!("Contract already deployed at: 0x{:x}", create2_address);
            // Return the existing address with a zero hash to indicate no new deployment
            return Ok((create2_address, FixedBytes::ZERO));
        }

        // Build CREATE2 deployment transaction
        let deployment_tx = self.build_create2_transaction(CREATE2_DEPLOYER, salt, initcode)?;

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
