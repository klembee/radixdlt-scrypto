use crate::api::types::*;
use crate::blueprints::resource::*;
use crate::*;
use sbor::rust::collections::BTreeMap;
use sbor::rust::string::String;
use sbor::rust::vec::Vec;
use scrypto_abi::BlueprintAbi;

pub struct PackageAbi;

impl PackageAbi {
    pub fn blueprint_abis() -> BTreeMap<String, BlueprintAbi> {
        BTreeMap::new()
    }
}
pub const PACKAGE_BLUEPRINT: &str = "Package";

pub const PACKAGE_PUBLISH_WASM_IDENT: &str = "publish_wasm";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct PackagePublishWasmInput {
    pub package_address: Option<[u8; 26]>, // TODO: Clean this up
    pub code: Vec<u8>,
    pub abi: Vec<u8>,
    pub royalty_config: BTreeMap<String, RoyaltyConfig>,
    pub metadata: BTreeMap<String, String>,
    pub access_rules: AccessRules,
}

pub const PACKAGE_PUBLISH_PRECOMPILED_IDENT: &str = "publish_precompiled";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct PackagePublishPrecompiledInput {
    pub package_address: Option<[u8; 26]>, // TODO: Clean this up
    pub native_package_code_id: u8,
    pub abi: Vec<u8>,
    pub dependent_resources: Vec<ResourceAddress>,
    pub dependent_components: Vec<ComponentAddress>,
    pub metadata: BTreeMap<String, String>,
    pub access_rules: AccessRules,
}
