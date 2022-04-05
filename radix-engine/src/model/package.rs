use crate::engine::EnvModuleResolver;
use crate::errors::WasmValidationError;
use sbor::*;
use scrypto::buffer::scrypto_decode;
use scrypto::prelude::HashMap;
use scrypto::rust::string::String;
use scrypto::rust::vec;
use scrypto::rust::vec::Vec;
use wasmi::{
    ExternVal, ImportsBuilder, MemoryRef, Module, ModuleInstance, ModuleRef, NopExternals,
    RuntimeValue,
};

/// A collection of blueprints, compiled and published as a single unit.
#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct Package {
    code: Vec<u8>,
    blueprints: HashMap<String, Type>,
}

#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub enum PackageError {
    BlueprintNotFound,
}

impl Package {
    /// Validates and creates a package
    pub fn new(code: Vec<u8>) -> Result<Self, WasmValidationError> {
        // Parse
        let parsed = Self::parse_module(&code)?;

        // check floating point
        parsed
            .deny_floating_point()
            .map_err(|_| WasmValidationError::FloatingPointNotAllowed)?;

        // Instantiate
        let instance = ModuleInstance::new(
            &parsed,
            &ImportsBuilder::new().with_resolver("env", &EnvModuleResolver),
        )
        .map_err(|_| WasmValidationError::InvalidModule)?;

        // Check start function
        if instance.has_start() {
            return Err(WasmValidationError::StartFunctionNotAllowed);
        }
        let module = instance.assert_no_start();

        // Check memory export
        let memory = match module.export_by_name("memory") {
            Some(ExternVal::Memory(mem)) => mem,
            _ => return Err(WasmValidationError::NoValidMemoryExport),
        };

        let rtn = module
            .invoke_export("package_init", &[], &mut NopExternals)
            .map_err(|e| WasmValidationError::NoPackageInitExport(e.into()))?
            .ok_or(WasmValidationError::InvalidPackageInit)?;

        let blueprints = match rtn {
            RuntimeValue::I32(ptr) => {
                let len: u32 = memory
                    .get_value(ptr as u32)
                    .map_err(|_| WasmValidationError::InvalidPackageInit)?;

                // SECURITY: meter before allocating memory
                let mut data = vec![0u8; len as usize];
                memory
                    .get_into((ptr + 4) as u32, &mut data)
                    .map_err(|_| WasmValidationError::InvalidPackageInit)?;

                scrypto_decode(&data).map_err(|_| WasmValidationError::InvalidPackageInit)
            }
            _ => Err(WasmValidationError::InvalidPackageInit),
        }?;

        Ok(Self { blueprints, code })
    }

    pub fn code(&self) -> &[u8] {
        &self.code
    }

    pub fn contains_blueprint(&self, blueprint_name: &str) -> bool {
        self.blueprints.contains_key(blueprint_name)
    }

    pub fn load_blueprint_schema(&self, blueprint_name: &str) -> Result<&Type, PackageError> {
        self
            .blueprints
            .get(blueprint_name)
            .ok_or(PackageError::BlueprintNotFound)
    }

    pub fn load_module(
        &self,
    ) -> Result<(ModuleRef, MemoryRef), PackageError> {
        let module = Self::parse_module(&self.code).unwrap();
        let inst = Self::instantiate_module(&module).unwrap();
        Ok(inst)
    }

    fn parse_module(code: &[u8]) -> Result<Module, WasmValidationError> {
        Module::from_buffer(code).map_err(|_| WasmValidationError::InvalidModule)
    }

    fn instantiate_module(module: &Module) -> Result<(ModuleRef, MemoryRef), WasmValidationError> {
        // Instantiate
        let instance = ModuleInstance::new(
            module,
            &ImportsBuilder::new().with_resolver("env", &EnvModuleResolver),
        )
        .map_err(|_| WasmValidationError::InvalidModule)?
        .assert_no_start();

        // Find memory export
        if let Some(ExternVal::Memory(memory)) = instance.export_by_name("memory") {
            Ok((instance, memory))
        } else {
            Err(WasmValidationError::NoValidMemoryExport)
        }
    }
}
