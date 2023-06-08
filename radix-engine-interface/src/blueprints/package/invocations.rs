use crate::blueprints::resource::*;
use crate::types::*;
use crate::*;
use radix_engine_common::data::manifest::model::ManifestBlobRef;
use radix_engine_common::data::manifest::model::ManifestOwn;
use radix_engine_common::prelude::ScryptoSchema;
use radix_engine_interface::api::node_modules::metadata::MetadataValue;
use sbor::rust::collections::BTreeMap;
use sbor::rust::string::String;
use sbor::rust::vec::Vec;
use scrypto_schema::{BlueprintSchema, SchemaMethodKey, SchemaMethodPermission};

pub const PACKAGE_BLUEPRINT: &str = "Package";

pub const PACKAGE_PUBLISH_WASM_IDENT: &str = "publish_wasm";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor, ManifestSbor)]
pub struct PackagePublishWasmInput {
    pub code: Vec<u8>,
    pub setup: PackageSetup,
    pub metadata: BTreeMap<String, MetadataValue>,
}

#[derive(Debug, Clone, Eq, PartialEq, ManifestSbor)]
pub struct PackagePublishWasmManifestInput {
    pub code: ManifestBlobRef,
    pub setup: PackageSetup,
    pub metadata: BTreeMap<String, MetadataValue>,
}

pub type PackagePublishWasmOutput = (PackageAddress, Bucket);

pub const PACKAGE_PUBLISH_WASM_ADVANCED_IDENT: &str = "publish_wasm_advanced";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor)]
pub struct PackagePublishWasmAdvancedInput {
    pub package_address: Option<GlobalAddressReservation>,
    pub code: Vec<u8>,
    pub setup: PackageSetup,
    pub metadata: BTreeMap<String, MetadataValue>,
    pub owner_rule: OwnerRole,
}

#[derive(Debug, Clone, Eq, PartialEq, ManifestSbor)]
pub struct PackagePublishWasmAdvancedManifestInput {
    pub package_address: Option<ManifestOwn>,
    pub code: ManifestBlobRef,
    pub setup: PackageSetup,
    pub metadata: BTreeMap<String, MetadataValue>,
    pub owner_rule: OwnerRole,
}

pub type PackagePublishWasmAdvancedOutput = PackageAddress;

pub const PACKAGE_PUBLISH_NATIVE_IDENT: &str = "publish_native";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor)]
pub struct PackagePublishNativeInput {
    pub package_address: Option<GlobalAddressReservation>,
    pub native_package_code_id: u8,
    pub setup: PackageSetup,
    pub metadata: BTreeMap<String, MetadataValue>,
}

#[derive(Debug, Clone, Eq, PartialEq, ManifestSbor)]
pub struct PackagePublishNativeManifestInput {
    pub package_address: Option<ManifestOwn>,
    pub native_package_code_id: u8,
    pub setup: PackageSetup,
    pub metadata: BTreeMap<String, MetadataValue>,
}

pub type PackagePublishNativeOutput = PackageAddress;

pub const PACKAGE_SET_ROYALTY_IDENT: &str = "PackageRoyalty_set_royalty";

#[derive(
    Debug, Clone, Eq, PartialEq, ScryptoSbor, ManifestCategorize, ManifestEncode, ManifestDecode,
)]
pub struct PackageSetRoyaltyInput {
    pub blueprint: String,
    pub fn_name: String,
    pub royalty: RoyaltyAmount,
}

pub type PackageSetRoyaltyOutput = ();

pub const PACKAGE_CLAIM_ROYALTIES_IDENT: &str = "PackageRoyalty_claim_royalties";

#[derive(
    Debug, Clone, Eq, PartialEq, ScryptoSbor, ManifestCategorize, ManifestEncode, ManifestDecode,
)]
pub struct PackageClaimRoyaltiesInput {}

pub type PackageClaimRoyaltiesOutput = Bucket;

#[derive(Debug, Clone, Eq, PartialEq, Default, ScryptoSbor, ManifestSbor)]
pub struct PackageSetup {
    pub blueprints: BTreeMap<String, BlueprintSetup>,
}

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor, ManifestSbor)]
pub struct BlueprintSetup {
    pub blueprint: BlueprintSchema,

    pub function_auth: BTreeMap<String, AccessRule>,
    pub royalty_config: RoyaltyConfig,

    pub schema: ScryptoSchema,
    pub template: BlueprintTemplate,
}

impl Default for BlueprintSetup {
    fn default() -> Self {
        Self {
            blueprint: BlueprintSchema::default(),
            function_auth: BTreeMap::default(),
            royalty_config: RoyaltyConfig::default(),
            schema: ScryptoSchema {
                type_kinds: vec![],
                type_metadata: vec![],
                type_validations: vec![],
            },
            template: BlueprintTemplate::default(),
        }
    }
}


#[derive(Debug, Clone, Eq, PartialEq, Default, ScryptoSbor, ManifestSbor)]
pub struct BlueprintTemplate {
    pub method_auth_template: BTreeMap<SchemaMethodKey, SchemaMethodPermission>,
    pub outer_method_auth_template: BTreeMap<SchemaMethodKey, SchemaMethodPermission>,
}
