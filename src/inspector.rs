use revm::inspectors::GasInspector;
use revm::{
    EvmContext, Inspector,
    interpreter::{CallInputs, CallOutcome, CreateInputs, CreateOutcome, Interpreter, opcode},
    primitives::{Address, U256, db::Database},
};
use std::cell::RefCell;
use std::collections::HashMap;

// Storage read tracker
#[derive(Default, Clone)]
pub struct StorageReadInspector {
    pub tx_number: u64,
    reads: RefCell<HashMap<(Address, U256), ()>>, // Use HashMap to record existence only, avoiding duplicates
    gas_inspector: GasInspector,
}

impl StorageReadInspector {
    pub fn new(tx_number: u64) -> Self {
        Self {
            tx_number,
            reads: RefCell::new(HashMap::new()),
            gas_inspector: GasInspector::default(),
        }
    }

    // Get read storage slots
    pub fn get_read_slots(&self) -> Vec<(Address, U256)> {
        self.reads.borrow().keys().cloned().collect()
    }
}

impl<DB: Database> Inspector<DB> for StorageReadInspector {
    // Initialize interpreter
    fn initialize_interp(&mut self, interp: &mut Interpreter, context: &mut EvmContext<DB>) {
        self.gas_inspector.initialize_interp(interp, context);
    }

    // Capture sload operation
    fn step(&mut self, interp: &mut Interpreter, context: &mut EvmContext<DB>) {
        self.gas_inspector.step(interp, context);

        // Check if the current opcode is SLOAD
        if interp.current_opcode() == opcode::SLOAD {
            // SLOAD will pop the storage slot index from the stack
            if let Ok(slot) = interp.stack.peek(0) {
                let address = interp.contract.target_address;
                self.reads.borrow_mut().insert((address, slot), ());
            }
        }
    }

    fn step_end(&mut self, interp: &mut Interpreter, context: &mut EvmContext<DB>) {
        self.gas_inspector.step_end(interp, context);
    }

    fn call_end(
        &mut self,
        context: &mut EvmContext<DB>,
        inputs: &CallInputs,
        outcome: CallOutcome,
    ) -> CallOutcome {
        self.gas_inspector.call_end(context, inputs, outcome)
    }

    fn create_end(
        &mut self,
        context: &mut EvmContext<DB>,
        inputs: &CreateInputs,
        outcome: CreateOutcome,
    ) -> CreateOutcome {
        self.gas_inspector.create_end(context, inputs, outcome)
    }
}
