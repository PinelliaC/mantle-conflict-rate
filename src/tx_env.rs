use anyhow::{Result, anyhow};
use op_alloy_consensus::OpTxEnvelope;
use revm::primitives::{OptimismFields, TransactTo, TxEnv, TxKind, U256};

pub fn prepare_tx_env(transaction: &OpTxEnvelope, encoded_transaction: &[u8]) -> Result<TxEnv> {
    let mut env = TxEnv::default();
    match transaction {
        OpTxEnvelope::Legacy(signed_tx) => {
            let tx = signed_tx.tx();
            env.caller = signed_tx
                .recover_signer()
                .map_err(|e| anyhow!("Failed to recover signer: {e}"))?;
            env.gas_limit = tx.gas_limit;
            env.gas_price = U256::from(tx.gas_price);
            env.gas_priority_fee = None;
            env.transact_to = match tx.to {
                TxKind::Call(to) => TransactTo::Call(to),
                TxKind::Create => TransactTo::Create,
            };
            env.value = tx.value;
            env.data = tx.input.clone();
            env.chain_id = tx.chain_id;
            env.nonce = Some(tx.nonce);
            env.access_list.clear();
            env.blob_hashes.clear();
            env.max_fee_per_blob_gas.take();
            env.optimism = OptimismFields {
                source_hash: None,
                mint: None,
                is_system_transaction: Some(false),
                enveloped_tx: Some(encoded_transaction.to_vec().into()),
                eth_value: None,
                eth_tx_value: None,
            };
            Ok(env)
        }
        OpTxEnvelope::Eip2930(signed_tx) => {
            let tx = signed_tx.tx();
            env.caller = signed_tx
                .recover_signer()
                .map_err(|e| anyhow!("Failed to recover signer: {e}"))?;
            env.gas_limit = tx.gas_limit;
            env.gas_price = U256::from(tx.gas_price);
            env.gas_priority_fee = None;
            env.transact_to = match tx.to {
                TxKind::Call(to) => TransactTo::Call(to),
                TxKind::Create => TransactTo::Create,
            };
            env.value = tx.value;
            env.data = tx.input.clone();
            env.chain_id = Some(tx.chain_id);
            env.nonce = Some(tx.nonce);
            env.access_list = tx.access_list.to_vec();
            env.blob_hashes.clear();
            env.max_fee_per_blob_gas.take();
            env.optimism = OptimismFields {
                source_hash: None,
                mint: None,
                is_system_transaction: Some(false),
                enveloped_tx: Some(encoded_transaction.to_vec().into()),
                eth_value: None,
                eth_tx_value: None,
            };
            Ok(env)
        }
        OpTxEnvelope::Eip1559(signed_tx) => {
            let tx = signed_tx.tx();
            env.caller = signed_tx
                .recover_signer()
                .map_err(|e| anyhow!("Failed to recover signer: {e}"))?;
            env.gas_limit = tx.gas_limit;
            env.gas_price = U256::from(tx.max_fee_per_gas);
            env.gas_priority_fee = Some(U256::from(tx.max_priority_fee_per_gas));
            env.transact_to = match tx.to {
                TxKind::Call(to) => TransactTo::Call(to),
                TxKind::Create => TransactTo::Create,
            };
            env.value = tx.value;
            env.data = tx.input.clone();
            env.chain_id = Some(tx.chain_id);
            env.nonce = Some(tx.nonce);
            env.access_list = tx.access_list.to_vec();
            env.blob_hashes.clear();
            env.max_fee_per_blob_gas.take();
            env.optimism = OptimismFields {
                source_hash: None,
                mint: None,
                is_system_transaction: Some(false),
                enveloped_tx: Some(encoded_transaction.to_vec().into()),
                eth_value: None,
                eth_tx_value: None,
            };
            Ok(env)
        }
        OpTxEnvelope::Deposit(tx) => {
            env.caller = tx.from;
            env.access_list.clear();
            env.gas_limit = tx.gas_limit;
            env.gas_price = U256::ZERO;
            env.gas_priority_fee = None;
            match tx.to {
                TxKind::Call(to) => env.transact_to = TransactTo::Call(to),
                TxKind::Create => env.transact_to = TransactTo::Create,
            }
            env.value = tx.value;
            env.data = tx.input.clone();
            env.chain_id = None;
            env.nonce = None;
            env.optimism = OptimismFields {
                source_hash: Some(tx.source_hash),
                mint: tx.mint,
                is_system_transaction: Some(tx.is_system_transaction),
                enveloped_tx: Some(encoded_transaction.to_vec().into()),
                eth_value: tx.eth_value,
                eth_tx_value: tx.eth_tx_value,
            };
            Ok(env)
        }
        _ => Err(anyhow!(
            "Unsupported transaction type: {:?}",
            transaction.tx_type() as u8
        )),
    }
}
