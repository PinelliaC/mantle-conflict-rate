use crate::analyzer::Conflict;
use std::time::Instant;

pub struct GlobalStats {
    pub total_blocks: usize,
    pub invalid_blocks: usize,
    pub total_txs: usize,
    pub conflicted_txs: usize,
    pub same_source_conflicts: usize,
    pub storage_slot_conflicts: usize,
    pub start_time: Instant,
}

impl Default for GlobalStats {
    fn default() -> Self {
        Self::new()
    }
}

impl GlobalStats {
    pub fn new() -> Self {
        Self {
            total_blocks: 0,
            invalid_blocks: 0,
            total_txs: 0,
            conflicted_txs: 0,
            same_source_conflicts: 0,
            storage_slot_conflicts: 0,
            start_time: Instant::now(),
        }
    }

    pub fn add_block_stats(
        &mut self,
        total_block_txs: usize,
        conflicted_block_txs: usize,
        conflicts: &[Conflict],
    ) {
        self.total_blocks += 1;
        self.total_txs += total_block_txs;
        self.conflicted_txs += conflicted_block_txs;

        // Count different types of conflicts
        for conflict in conflicts {
            match conflict.conflict_type {
                crate::analyzer::ConflictType::MultipleMntTransfersFromSameSource => {
                    self.same_source_conflicts += conflict.transactions.len();
                }
                crate::analyzer::ConflictType::StorageSlotConflict => {
                    self.storage_slot_conflicts += conflict.transactions.len();
                }
            }
        }
    }

    pub fn record_invalid_block(&mut self) {
        self.invalid_blocks += 1;
    }

    pub fn print_final_stats(&self) {
        let elapsed = self.start_time.elapsed();
        let dependency_ratio = if self.total_txs > 0 {
            (self.conflicted_txs as f64 / self.total_txs as f64) * 100.0
        } else {
            0.0
        };

        println!("\nAnalysis Results:");
        println!("  Time taken: {:.2} seconds", elapsed.as_secs_f64());
        println!("  Total blocks: {}", self.total_blocks);
        println!("  Invalid blocks: {}", self.invalid_blocks);
        println!("  Total transactions: {}", self.total_txs);
        println!("  Dependent transactions: {}", self.conflicted_txs);
        println!("  Dependency ratio: {:.2}%", dependency_ratio);
        println!("  Conflict counts:");
        println!("    same-source: {}", self.same_source_conflicts);
        println!(
            "    contract-slot-conflict: {}",
            self.storage_slot_conflicts
        );
    }
}
