#![cfg_attr(not(test), warn(unused_crate_dependencies))]
use alloy_eips::eip2718::Decodable2718;
use alloy_primitives::Bytes;
use anyhow::{Result, anyhow};
use dotenv::dotenv;
use ethers_core::types::H256;
use ethers_providers::{Http, Middleware, Provider};
use op_alloy_consensus::OpTxEnvelope;
use revm::{
    Evm,
    db::{CacheDB, EthersDB},
    primitives::{Address, ResultAndState, SpecId, U256},
};
use std::collections::HashSet;
use std::sync::Arc;
use std::time::Instant;

use mantle_conflict_rate::{
    AccessType, Conflict, ConflictAnalyzer, GlobalStats, StorageReadInspector, prepare_tx_env,
};

macro_rules! local_fill {
    ($left:expr, $right:expr, $fun:expr) => {
        if let Some(right) = $right {
            $left = $fun(right.0)
        }
    };
    ($left:expr, $right:expr) => {
        if let Some(right) = $right {
            $left = Address::from(right.as_fixed_bytes())
        }
    };
}

#[tokio::main]
async fn main() -> Result<()> {
    let start = 77075444;
    let mut global_stats = GlobalStats::new();

    for block_number in start..start + 2 {
        match range(block_number).await {
            Ok((block_txs, block_conflicts, conflicts)) => {
                global_stats.add_block_stats(block_txs, block_conflicts, &conflicts);
            }
            Err(_) => {
                global_stats.record_invalid_block();
            }
        }
    }

    global_stats.print_final_stats();
    Ok(())
}

async fn range(block_number: u64) -> Result<(usize, usize, Vec<Conflict>)> {
    dotenv().ok();
    let mantle_url = std::env::var("MANTLE_URL").unwrap();

    let client = Provider::<Http>::try_from(mantle_url)?;
    let client = Arc::new(client);

    let chain_id: u64 = client.get_chainid().await.unwrap().as_u64();

    let block = match client.get_block_with_txs(block_number).await {
        Ok(Some(block)) => block,
        Ok(None) => anyhow::bail!("Block not found"),
        Err(error) => anyhow::bail!("Error: {:?}", error),
    };
    println!("Fetched block number: {}", block.number.unwrap().0[0]);

    let previous_block_number = block_number - 1;
    let prev_id = previous_block_number.into();
    let state_db = EthersDB::new(client.clone(), Some(prev_id)).expect("panic");
    let mut cache_db = CacheDB::new(state_db);

    let mut evm = Evm::builder()
        .with_db(&mut cache_db)
        .with_external_context(StorageReadInspector::new(0))
        .modify_block_env(|b| {
            if let Some(number) = block.number {
                let nn = number.0[0];
                b.number = U256::from(nn);
            }
            local_fill!(b.coinbase, block.author);
            local_fill!(b.timestamp, Some(block.timestamp), U256::from_limbs);
            local_fill!(b.difficulty, Some(block.difficulty), U256::from_limbs);
            local_fill!(b.gas_limit, Some(block.gas_limit), U256::from_limbs);
            if let Some(base_fee) = block.base_fee_per_gas {
                local_fill!(b.basefee, Some(base_fee), U256::from_limbs);
            }
        })
        .with_spec_id(SpecId::SHANGHAI)
        .modify_cfg_env(|c| {
            c.chain_id = chain_id;
        })
        .optimism()
        .build();

    let txs = block.transactions.len();
    println!("Found {txs} transactions.");

    let start = Instant::now();
    let mut conflict_analyzer = ConflictAnalyzer::new();

    for tx in block.transactions {
        let tx_number = tx.transaction_index.unwrap().0[0];
        let tx_hash = tx.hash;
        let raw_tx = client
            .request::<&[H256; 1], Bytes>("debug_getRawTransaction", &[tx_hash.into()])
            .await
            .map_err(|e| anyhow!("Failed to fetch raw transaction: {e}"))?;
        let op_tx = OpTxEnvelope::decode_2718(&mut raw_tx.as_ref())
            .map_err(|e| anyhow!("Failed to decode EIP-2718 transaction: {e}"))?;
        let env = prepare_tx_env(&op_tx, raw_tx.as_ref())?;

        let from_address = env.caller;
        if !env.value.is_zero() {
            conflict_analyzer.record_mnt_transfer(from_address, tx_number);
        }

        let inspector = StorageReadInspector::new(tx_number);
        evm = evm
            .modify()
            .with_tx_env(env)
            .reset_handler_with_external_context(inspector.clone())
            .build();

        let ResultAndState { result, state } = evm
            .transact()
            .map_err(|e| anyhow!("Failed to transact: {e}"))?;

        if result.is_success() {
            for (address, account) in &state {
                if account.is_touched() {
                    for (slot_key, slot) in &account.storage {
                        if slot.is_changed() {
                            conflict_analyzer.record_storage_access(
                                *address,
                                *slot_key,
                                tx_number,
                                AccessType::Write,
                            );
                        }
                    }
                }
            }
        }

        let read_slots = inspector.get_read_slots();
        for (address, slot) in read_slots {
            conflict_analyzer.record_storage_access(address, slot, tx_number, AccessType::Read);
        }
    }

    let conflicts = conflict_analyzer.analyze_conflicts();
    let affected_txs = conflicts
        .iter()
        .flat_map(|c| c.transactions.clone())
        .collect::<HashSet<_>>()
        .len();

    let elapsed = start.elapsed();
    println!(
        "Finished execution. Total CPU time: {:.6}s",
        elapsed.as_secs_f64()
    );
    drop(evm);

    Ok((txs, affected_txs, conflicts))
}
