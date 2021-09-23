use radix_engine::execution::*;
use radix_engine::model::Level;
use sbor::*;
use scrypto::rust::fmt;
use scrypto::types::*;

use crate::utils::*;

/// A transaction consists a sequence of instructions.
#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct Transaction {
    pub instructions: Vec<Instruction>,
}

/// Represents an instruction
#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub enum Instruction {
    /// Reserve `n` buckets upfront.
    ReserveBuckets {
        n: u8,
    },

    /// Create a bucket to be used for function call.
    MoveToBucket {
        amount: Amount,
        resource: Address,
        index: u8,
    },

    /// Call a function.
    CallFunction {
        package: Address,
        blueprint: String,
        function: String,
        args: Args,
    },

    /// Call a method.
    CallMethod {
        component: Address,
        method: String,
        args: Args,
    },

    /// Pass all remaining resources to a component.
    DepositAll {
        component: Address,
        method: String,
    },

    Finalize,
}

#[derive(Clone, TypeId, Encode, Decode)]
pub struct Args(pub Vec<Vec<u8>>);

impl fmt::Debug for Args {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "[{}]",
            self.0
                .iter()
                .map(|v| format_sbor(v).unwrap())
                .collect::<Vec<String>>()
                .join(", ")
        )
    }
}

#[derive(Debug)]
pub struct TransactionReceipt {
    pub transaction: Transaction,
    pub success: bool,
    pub execution_time: u128,
    pub results: Vec<Result<Option<Vec<u8>>, RuntimeError>>,
    pub logs: Vec<(Level, String)>,
    pub new_addresses: Vec<Address>,
}
