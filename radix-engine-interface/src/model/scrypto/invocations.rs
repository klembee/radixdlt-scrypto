use radix_engine_interface::model::ComponentAddress;
use crate::api::api::Invocation;
use crate::api::types::{ScryptoFunctionIdent, ScryptoMethodIdent, ScryptoReceiver};
use crate::data::IndexedScryptoValue;
use crate::model::SerializedInvocation;
use crate::scrypto;
use crate::wasm::SerializableInvocation;
use sbor::rust::vec::Vec;
use sbor::*;

/// Scrypto function/method invocation.
#[derive(Debug)]
#[scrypto(Categorize, Encode, Decode)]
pub enum ScryptoInvocation {
    Function(ScryptoFunctionIdent, Vec<u8>),
}

impl Invocation for ScryptoInvocation {
    type Output = Vec<u8>;
}

impl SerializableInvocation for ScryptoInvocation {
    type ScryptoOutput = Vec<u8>;
}

impl Into<SerializedInvocation> for ScryptoInvocation {
    fn into(self) -> SerializedInvocation {
        SerializedInvocation::Scrypto(self)
    }
}

impl ScryptoInvocation {
    pub fn args(&self) -> &[u8] {
        match self {
            ScryptoInvocation::Function(_, args) => &args,
        }
    }
}

/// Scrypto function/method invocation.
#[derive(Debug)]
#[scrypto(Categorize, Encode, Decode)]
pub struct ScryptoMethodInvocation {
    pub receiver: ScryptoReceiver,
    pub method_name: String,
    pub args: Vec<u8>,
}

impl Invocation for ScryptoMethodInvocation {
    type Output = Vec<u8>;
}

impl SerializableInvocation for ScryptoMethodInvocation {
    type ScryptoOutput = Vec<u8>;
}

impl Into<SerializedInvocation> for ScryptoMethodInvocation {
    fn into(self) -> SerializedInvocation {
        SerializedInvocation::Component(self)
    }
}

#[derive(Debug)]
pub enum ParsedScryptoInvocation {
    Function(ScryptoFunctionIdent, IndexedScryptoValue),
    Method(ScryptoMethodIdent, IndexedScryptoValue),
}

impl Invocation for ParsedScryptoInvocation {
    type Output = IndexedScryptoValue;
}

impl ParsedScryptoInvocation {
    pub fn args(&self) -> &IndexedScryptoValue {
        match self {
            ParsedScryptoInvocation::Function(_, args) => &args,
            ParsedScryptoInvocation::Method(_, args) => &args,
        }
    }
}
