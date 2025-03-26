use revm::primitives::{Address, U256};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccessType {
    Read,
    Write,
}

#[derive(Debug)]
pub enum ConflictType {
    MultipleMntTransfersFromSameSource,
    StorageSlotConflict,
}

#[derive(Debug)]
pub struct Conflict {
    pub conflict_type: ConflictType,
    pub address: Address,
    pub transactions: Vec<u64>,
    pub storage_slot: Option<U256>,
}

pub struct ConflictAnalyzer {
    mnt_transfers: HashMap<Address, Vec<u64>>,
    storage_accesses: HashMap<(Address, U256), Vec<(u64, AccessType)>>,
}

impl Default for ConflictAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

impl ConflictAnalyzer {
    pub fn new() -> Self {
        Self {
            mnt_transfers: HashMap::new(),
            storage_accesses: HashMap::new(),
        }
    }

    pub fn record_mnt_transfer(&mut self, from: Address, tx_number: u64) {
        self.mnt_transfers.entry(from).or_default().push(tx_number);
    }

    pub fn record_storage_access(
        &mut self,
        contract: Address,
        slot: U256,
        tx_number: u64,
        access_type: AccessType,
    ) {
        self.storage_accesses
            .entry((contract, slot))
            .or_default()
            .push((tx_number, access_type));
    }

    pub fn analyze_conflicts(&self) -> Vec<Conflict> {
        let mut conflicts = Vec::new();

        // 1. Multiple MNT transfers from the same source address
        for (address, txs) in &self.mnt_transfers {
            if txs.len() > 1 {
                conflicts.push(Conflict {
                    conflict_type: ConflictType::MultipleMntTransfersFromSameSource,
                    address: *address,
                    transactions: txs.clone(),
                    storage_slot: None,
                });
            }
        }

        // 2. Analyze storage slot conflicts
        for ((contract, slot), accesses) in &self.storage_accesses {
            let mut tx_accesses: HashMap<u64, AccessType> = HashMap::new();
            
            for (tx_num, access_type) in accesses {
                tx_accesses.entry(*tx_num)
                    .and_modify(|e| {
                        if *access_type == AccessType::Write {
                            *e = AccessType::Write
                        }
                    })
                    .or_insert(*access_type);
            }

            let has_write = tx_accesses.values().any(|at| *at == AccessType::Write);
            
            if has_write && tx_accesses.len() > 1 {
                let conflict_txs: Vec<u64> = tx_accesses.keys().cloned().collect();
                
                conflicts.push(Conflict {
                    conflict_type: ConflictType::StorageSlotConflict,
                    address: *contract,
                    transactions: conflict_txs,
                    storage_slot: Some(*slot),
                });
            }
        }

        conflicts
    }

    pub fn print_analysis(&self, total_txs: usize) -> usize {
        let conflicts = self.analyze_conflicts();
        let affected_txs: HashSet<u64> = conflicts
            .iter()
            .flat_map(|c| c.transactions.clone())
            .collect();

        let affected_count = affected_txs.len();
        let conflict_ratio = if total_txs > 0 {
            affected_count as f64 / total_txs as f64
        } else {
            0.0
        };

        println!(
            "Conflict rate: {:.6} ({} / {})",
            conflict_ratio, affected_count, total_txs
        );

        affected_count
    }
}
