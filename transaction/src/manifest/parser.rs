use crate::manifest::ast::{Instruction, Value, ValueKind, ValueKindWithSpan, ValueWithSpan};
use crate::manifest::diagnostic_snippets::create_snippet;
use crate::manifest::manifest_enums::KNOWN_ENUM_DISCRIMINATORS;
use crate::manifest::token::{Position, Span, Token, TokenKind};
use radix_engine_common::data::manifest::MANIFEST_SBOR_V1_MAX_DEPTH;
use sbor::rust::fmt;

use super::ast::InstructionWithSpan;

// For values greater than below it is not possible to encode compiled manifest due to
//   EncodeError::MaxDepthExceeded(MANIFEST_SBOR_V1_MAX_DEPTH)
pub const PARSER_MAX_DEPTH: usize = MANIFEST_SBOR_V1_MAX_DEPTH - 4;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParserError {
    UnexpectedEof,
    UnexpectedToken {
        expected: TokenType,
        actual: TokenKind,
        span: Span,
    },
    UnexpectedTokenOrMissingSemicolon {
        expected: TokenType,
        actual: TokenKind,
        span: Span,
    },
    InvalidNumberOfValues {
        expected: usize,
        actual: usize,
        span: Span,
    },
    InvalidNumberOfTypes {
        expected: usize,
        actual: usize,
        span: Span,
    },
    UnknownEnumDiscriminator {
        actual: String,
        span: Span,
    },
    MaxDepthExceeded {
        actual: usize,
        max: usize,
        span: Span,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TokenType {
    Instruction,
    Value,
    ValueKind,
    EnumDiscriminator,
    Exact(TokenKind),
}

impl fmt::Display for TokenType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            TokenType::Instruction => write!(f, "an Instruction"),
            TokenType::Value => write!(f, "a Value"),
            TokenType::ValueKind => write!(f, "a Value kind"),
            TokenType::EnumDiscriminator => write!(f, "a valid enum discriminator"),
            TokenType::Exact(token_kind) => write!(f, "{}", token_kind),
        }
    }
}

pub enum InstructionIdent {
    // ==============
    // Standard instructions
    // ==============
    TakeFromWorktop,
    TakeNonFungiblesFromWorktop,
    TakeAllFromWorktop,
    ReturnToWorktop,
    AssertWorktopContains,
    AssertWorktopContainsNonFungibles,
    AssertWorktopContainsAny,

    PopFromAuthZone,
    PushToAuthZone,
    CreateProofFromAuthZoneOfAmount,
    CreateProofFromAuthZoneOfNonFungibles,
    CreateProofFromAuthZoneOfAll,
    DropAuthZoneProofs,
    DropAuthZoneRegularProofs,
    DropAuthZoneSignatureProofs,
    CreateProofFromBucketOfAmount,
    CreateProofFromBucketOfNonFungibles,
    CreateProofFromBucketOfAll,
    BurnResource,
    CloneProof,
    DropProof,
    CallFunction,
    CallMethod,
    CallRoyaltyMethod,
    CallMetadataMethod,
    CallRoleAssignmentMethod,
    CallDirectVaultMethod,
    DropNamedProofs,
    DropAllProofs,
    AllocateGlobalAddress,

    // ==============
    // Call direct vault method aliases
    // ==============
    RecallFromVault,
    FreezeVault,
    UnfreezeVault,
    RecallNonFungiblesFromVault,

    // ==============
    // Call function aliases
    // ==============
    PublishPackage,
    PublishPackageAdvanced,
    CreateFungibleResource,
    CreateFungibleResourceWithInitialSupply,
    CreateNonFungibleResource,
    CreateNonFungibleResourceWithInitialSupply,
    CreateAccessController,
    CreateIdentity,
    CreateIdentityAdvanced,
    CreateAccount,
    CreateAccountAdvanced,

    // ==============
    // Call non-main-method aliases
    // ==============
    SetMetadata,
    RemoveMetadata,
    LockMetadata,
    SetComponentRoyalty,
    LockComponentRoyalty,
    ClaimComponentRoyalties,
    SetOwnerRole,
    LockOwnerRole,
    SetRole,

    // ==============
    // Call main-method aliases
    // ==============
    ClaimPackageRoyalties,
    MintFungible,
    MintNonFungible,
    MintRuidNonFungible,
    CreateValidator,
}

impl InstructionIdent {
    pub fn from_ident(ident: &str) -> Option<Self> {
        let value = match ident {
            // ==============
            // Standard instructions
            // ==============
            "TAKE_FROM_WORKTOP" => InstructionIdent::TakeFromWorktop,
            "TAKE_NON_FUNGIBLES_FROM_WORKTOP" => InstructionIdent::TakeNonFungiblesFromWorktop,
            "TAKE_ALL_FROM_WORKTOP" => InstructionIdent::TakeAllFromWorktop,
            "RETURN_TO_WORKTOP" => InstructionIdent::ReturnToWorktop,
            "ASSERT_WORKTOP_CONTAINS" => InstructionIdent::AssertWorktopContains,
            "ASSERT_WORKTOP_CONTAINS_NON_FUNGIBLES" => {
                InstructionIdent::AssertWorktopContainsNonFungibles
            }
            "ASSERT_WORKTOP_CONTAINS_ANY" => InstructionIdent::AssertWorktopContainsAny,

            "POP_FROM_AUTH_ZONE" => InstructionIdent::PopFromAuthZone,
            "PUSH_TO_AUTH_ZONE" => InstructionIdent::PushToAuthZone,
            "CREATE_PROOF_FROM_AUTH_ZONE_OF_AMOUNT" => {
                InstructionIdent::CreateProofFromAuthZoneOfAmount
            }
            "CREATE_PROOF_FROM_AUTH_ZONE_OF_NON_FUNGIBLES" => {
                InstructionIdent::CreateProofFromAuthZoneOfNonFungibles
            }
            "CREATE_PROOF_FROM_AUTH_ZONE_OF_ALL" => InstructionIdent::CreateProofFromAuthZoneOfAll,
            "DROP_AUTH_ZONE_PROOFS" => InstructionIdent::DropAuthZoneProofs,
            "DROP_AUTH_ZONE_SIGNATURE_PROOFS" => InstructionIdent::DropAuthZoneSignatureProofs,
            "DROP_AUTH_ZONE_REGULAR_PROOFS" => InstructionIdent::DropAuthZoneRegularProofs,

            "CREATE_PROOF_FROM_BUCKET_OF_AMOUNT" => InstructionIdent::CreateProofFromBucketOfAmount,
            "CREATE_PROOF_FROM_BUCKET_OF_NON_FUNGIBLES" => {
                InstructionIdent::CreateProofFromBucketOfNonFungibles
            }
            "CREATE_PROOF_FROM_BUCKET_OF_ALL" => InstructionIdent::CreateProofFromBucketOfAll,
            "BURN_RESOURCE" => InstructionIdent::BurnResource,

            "CLONE_PROOF" => InstructionIdent::CloneProof,
            "DROP_PROOF" => InstructionIdent::DropProof,

            "CALL_FUNCTION" => InstructionIdent::CallFunction,
            "CALL_METHOD" => InstructionIdent::CallMethod,
            "CALL_ROYALTY_METHOD" => InstructionIdent::CallRoyaltyMethod,
            "CALL_METADATA_METHOD" => InstructionIdent::CallMetadataMethod,
            "CALL_ROLE_ASSIGNMENT_METHOD" => InstructionIdent::CallRoleAssignmentMethod,
            "CALL_DIRECT_VAULT_METHOD" => InstructionIdent::CallDirectVaultMethod,

            "DROP_NAMED_PROOFS" => InstructionIdent::DropNamedProofs,
            "DROP_ALL_PROOFS" => InstructionIdent::DropAllProofs,
            "ALLOCATE_GLOBAL_ADDRESS" => InstructionIdent::AllocateGlobalAddress,

            // ==============
            // Call direct vault method aliases
            // ==============
            "RECALL_FROM_VAULT" => InstructionIdent::RecallFromVault,
            "FREEZE_VAULT" => InstructionIdent::FreezeVault,
            "UNFREEZE_VAULT" => InstructionIdent::UnfreezeVault,
            "RECALL_NON_FUNGIBLES_FROM_VAULT" => InstructionIdent::RecallNonFungiblesFromVault,

            // ==============
            // Call function aliases
            // ==============
            "PUBLISH_PACKAGE" => InstructionIdent::PublishPackage,
            "PUBLISH_PACKAGE_ADVANCED" => InstructionIdent::PublishPackageAdvanced,
            "CREATE_FUNGIBLE_RESOURCE" => InstructionIdent::CreateFungibleResource,
            "CREATE_FUNGIBLE_RESOURCE_WITH_INITIAL_SUPPLY" => {
                InstructionIdent::CreateFungibleResourceWithInitialSupply
            }
            "CREATE_NON_FUNGIBLE_RESOURCE" => InstructionIdent::CreateNonFungibleResource,
            "CREATE_NON_FUNGIBLE_RESOURCE_WITH_INITIAL_SUPPLY" => {
                InstructionIdent::CreateNonFungibleResourceWithInitialSupply
            }
            "CREATE_IDENTITY" => InstructionIdent::CreateIdentity,
            "CREATE_IDENTITY_ADVANCED" => InstructionIdent::CreateIdentityAdvanced,
            "CREATE_ACCOUNT" => InstructionIdent::CreateAccount,
            "CREATE_ACCOUNT_ADVANCED" => InstructionIdent::CreateAccountAdvanced,
            "CREATE_ACCESS_CONTROLLER" => InstructionIdent::CreateAccessController,

            // ==============
            // Call non-main-method aliases
            // ==============
            "SET_METADATA" => InstructionIdent::SetMetadata,
            "REMOVE_METADATA" => InstructionIdent::RemoveMetadata,
            "LOCK_METADATA" => InstructionIdent::LockMetadata,
            "SET_COMPONENT_ROYALTY" => InstructionIdent::SetComponentRoyalty,
            "LOCK_COMPONENT_ROYALTY" => InstructionIdent::LockComponentRoyalty,
            "CLAIM_COMPONENT_ROYALTIES" => InstructionIdent::ClaimComponentRoyalties,
            "SET_OWNER_ROLE" => InstructionIdent::SetOwnerRole,
            "LOCK_OWNER_ROLE" => InstructionIdent::LockOwnerRole,
            "SET_ROLE" => InstructionIdent::SetRole,

            // ==============
            // Call main-method aliases
            // ==============
            "MINT_FUNGIBLE" => InstructionIdent::MintFungible,
            "MINT_NON_FUNGIBLE" => InstructionIdent::MintNonFungible,
            "MINT_RUID_NON_FUNGIBLE" => InstructionIdent::MintRuidNonFungible,
            "CLAIM_PACKAGE_ROYALTIES" => InstructionIdent::ClaimPackageRoyalties,
            "CREATE_VALIDATOR" => InstructionIdent::CreateValidator,
            _ => {
                return None;
            }
        };
        Some(value)
    }
}

pub enum SborValueIdent {
    // ==============
    // SBOR composite value types
    // ==============
    Enum,
    Array,
    Tuple,
    Map,
    // ==============
    // SBOR aliases
    // ==============
    Some,
    None,
    Ok,
    Err,
    Bytes,
    NonFungibleGlobalId,
    // ==============
    // SBOR custom types
    // ==============
    Address,
    Bucket,
    Proof,
    Expression,
    Blob,
    Decimal,
    PreciseDecimal,
    NonFungibleLocalId,
    AddressReservation,
    NamedAddress,
}

impl SborValueIdent {
    pub fn from_ident(ident: &str) -> Option<Self> {
        let value = match ident {
            // ==============
            // SBOR composite value types
            // ==============
            "Enum" => SborValueIdent::Enum,
            "Array" => SborValueIdent::Array,
            "Tuple" => SborValueIdent::Tuple,
            "Map" => SborValueIdent::Map,
            // ==============
            // SBOR aliases
            // ==============
            "Some" => SborValueIdent::Some,
            "None" => SborValueIdent::None,
            "Ok" => SborValueIdent::Ok,
            "Err" => SborValueIdent::Err,
            "Bytes" => SborValueIdent::Bytes,
            "NonFungibleGlobalId" => SborValueIdent::NonFungibleGlobalId,
            // ==============
            // Custom types
            // ==============
            "Address" => SborValueIdent::Address,
            "Bucket" => SborValueIdent::Bucket,
            "Proof" => SborValueIdent::Proof,
            "Expression" => SborValueIdent::Expression,
            "Blob" => SborValueIdent::Blob,
            "Decimal" => SborValueIdent::Decimal,
            "PreciseDecimal" => SborValueIdent::PreciseDecimal,
            "NonFungibleLocalId" => SborValueIdent::NonFungibleLocalId,
            "AddressReservation" => SborValueIdent::AddressReservation,
            "NamedAddress" => SborValueIdent::NamedAddress,
            _ => {
                return None;
            }
        };
        Some(value)
    }
}

pub enum SborValueKindIdent {
    // ==============
    // Simple basic value kinds
    // ==============
    Bool,
    I8,
    I16,
    I32,
    I64,
    I128,
    U8,
    U16,
    U32,
    U64,
    U128,
    String,
    // ==============
    // Composite basic value kinds
    // ==============
    Enum,
    Array,
    Tuple,
    Map,
    // ==============
    // Value kind aliases
    // ==============
    Bytes,
    NonFungibleGlobalId,
    // ==============
    // Custom value kinds
    // ==============
    Address,
    Bucket,
    Proof,
    Expression,
    Blob,
    Decimal,
    PreciseDecimal,
    NonFungibleLocalId,
    AddressReservation,
    NamedAddress,
}

impl SborValueKindIdent {
    pub fn from_ident(ident: &str) -> Option<Self> {
        let value = match ident {
            // ==============
            // Basic simple types
            // ==============
            "Bool" => SborValueKindIdent::Bool,
            "I8" => SborValueKindIdent::I8,
            "I16" => SborValueKindIdent::I16,
            "I32" => SborValueKindIdent::I32,
            "I64" => SborValueKindIdent::I64,
            "I128" => SborValueKindIdent::I128,
            "U8" => SborValueKindIdent::U8,
            "U16" => SborValueKindIdent::U16,
            "U32" => SborValueKindIdent::U32,
            "U64" => SborValueKindIdent::U64,
            "U128" => SborValueKindIdent::U128,
            "String" => SborValueKindIdent::String,
            // ==============
            // Basic composite types
            // ==============
            "Enum" => SborValueKindIdent::Enum,
            "Array" => SborValueKindIdent::Array,
            "Tuple" => SborValueKindIdent::Tuple,
            "Map" => SborValueKindIdent::Map,
            // ==============
            // Value kind aliases
            // ==============
            "Bytes" => SborValueKindIdent::Bytes,
            "NonFungibleGlobalId" => SborValueKindIdent::NonFungibleGlobalId,
            // ==============
            // Custom types
            // ==============
            "Address" => SborValueKindIdent::Address,
            "Bucket" => SborValueKindIdent::Bucket,
            "Proof" => SborValueKindIdent::Proof,
            "Expression" => SborValueKindIdent::Expression,
            "Blob" => SborValueKindIdent::Blob,
            "Decimal" => SborValueKindIdent::Decimal,
            "PreciseDecimal" => SborValueKindIdent::PreciseDecimal,
            "NonFungibleLocalId" => SborValueKindIdent::NonFungibleLocalId,
            "AddressReservation" => SborValueKindIdent::AddressReservation,
            "NamedAddress" => SborValueKindIdent::NamedAddress,
            _ => {
                return None;
            }
        };
        Some(value)
    }
}

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
    max_depth: usize,
    stack_depth: usize,
}

#[macro_export]
macro_rules! advance_ok {
    ( $self:expr, $v:expr ) => {{
        $self.advance()?;
        Ok($v)
    }};
}

#[macro_export]
macro_rules! advance_match {
    ( $self:expr, $expected:expr ) => {{
        let token = $self.advance()?;
        if token.kind != $expected {
            return Err(ParserError::UnexpectedToken {
                expected: TokenType::Exact($expected),
                actual: token.kind,
                span: token.span,
            });
        }
    }};
}

impl Parser {
    pub fn new(tokens: Vec<Token>, max_depth: usize) -> Self {
        Self {
            tokens,
            current: 0,
            max_depth,
            stack_depth: 0,
        }
    }

    #[inline]
    fn track_stack_depth_increase(&mut self) -> Result<(), ParserError> {
        self.stack_depth += 1;
        if self.stack_depth > self.max_depth {
            let token = self.peek()?;

            return Err(ParserError::MaxDepthExceeded {
                actual: self.stack_depth,
                max: self.max_depth,
                span: token.span,
            });
        }
        Ok(())
    }

    #[inline]
    fn track_stack_depth_decrease(&mut self) -> Result<(), ParserError> {
        self.stack_depth -= 1;
        Ok(())
    }

    pub fn is_eof(&self) -> bool {
        self.current == self.tokens.len()
    }

    pub fn peek(&mut self) -> Result<Token, ParserError> {
        self.tokens
            .get(self.current)
            .cloned()
            .ok_or(ParserError::UnexpectedEof)
    }

    pub fn advance(&mut self) -> Result<Token, ParserError> {
        let token = self.peek()?;
        self.current += 1;
        Ok(token)
    }

    pub fn parse_manifest(&mut self) -> Result<Vec<InstructionWithSpan>, ParserError> {
        let mut instructions = Vec::<InstructionWithSpan>::new();

        while !self.is_eof() {
            instructions.push(self.parse_instruction()?);
        }

        Ok(instructions)
    }

    fn parse_values_till_semicolon(&mut self) -> Result<Vec<ValueWithSpan>, ParserError> {
        let mut values = Vec::new();
        while self.peek()?.kind != TokenKind::Semicolon {
            let stack_depth = self.stack_depth;
            let result = self.parse_value();
            match result {
                Ok(value) => values.push(value),
                Err(err) => match err {
                    // parse_value() is recursive so we need to check the stack depth to determine
                    // if semicolon might be missing
                    ParserError::UnexpectedToken {
                        expected,
                        actual,
                        span,
                    } if expected == TokenType::Value && (stack_depth + 1 == self.stack_depth) => {
                        return Err(ParserError::UnexpectedTokenOrMissingSemicolon {
                            expected,
                            actual,
                            span,
                        })
                    }
                    err => return Err(err),
                },
            }
        }
        Ok(values)
    }

    pub fn parse_instruction(&mut self) -> Result<InstructionWithSpan, ParserError> {
        let token = self.advance()?;
        let instruction_ident = match &token.kind {
            TokenKind::Ident(ident_str) => {
                InstructionIdent::from_ident(ident_str).ok_or(ParserError::UnexpectedToken {
                    expected: TokenType::Instruction,
                    actual: token.kind,
                    span: token.span,
                })?
            }
            _ => {
                return Err(ParserError::UnexpectedToken {
                    expected: TokenType::Instruction,
                    actual: token.kind,
                    span: token.span,
                });
            }
        };
        let instruction_start = token.span.start;

        let instruction = match instruction_ident {
            InstructionIdent::TakeFromWorktop => Instruction::TakeFromWorktop {
                resource_address: self.parse_value()?,
                amount: self.parse_value()?,
                new_bucket: self.parse_value()?,
            },
            InstructionIdent::TakeNonFungiblesFromWorktop => {
                Instruction::TakeNonFungiblesFromWorktop {
                    resource_address: self.parse_value()?,
                    ids: self.parse_value()?,
                    new_bucket: self.parse_value()?,
                }
            }
            InstructionIdent::TakeAllFromWorktop => Instruction::TakeAllFromWorktop {
                resource_address: self.parse_value()?,
                new_bucket: self.parse_value()?,
            },
            InstructionIdent::ReturnToWorktop => Instruction::ReturnToWorktop {
                bucket: self.parse_value()?,
            },
            InstructionIdent::AssertWorktopContains => Instruction::AssertWorktopContains {
                resource_address: self.parse_value()?,
                amount: self.parse_value()?,
            },
            InstructionIdent::AssertWorktopContainsNonFungibles => {
                Instruction::AssertWorktopContainsNonFungibles {
                    resource_address: self.parse_value()?,
                    ids: self.parse_value()?,
                }
            }
            InstructionIdent::AssertWorktopContainsAny => Instruction::AssertWorktopContainsAny {
                resource_address: self.parse_value()?,
            },
            InstructionIdent::PopFromAuthZone => Instruction::PopFromAuthZone {
                new_proof: self.parse_value()?,
            },
            InstructionIdent::PushToAuthZone => Instruction::PushToAuthZone {
                proof: self.parse_value()?,
            },
            InstructionIdent::DropAuthZoneProofs => Instruction::DropAuthZoneProofs,
            InstructionIdent::DropAuthZoneRegularProofs => Instruction::DropAuthZoneRegularProofs,
            InstructionIdent::DropAuthZoneSignatureProofs => {
                Instruction::DropAuthZoneSignatureProofs
            }
            InstructionIdent::CreateProofFromAuthZoneOfAmount => {
                Instruction::CreateProofFromAuthZoneOfAmount {
                    resource_address: self.parse_value()?,
                    amount: self.parse_value()?,
                    new_proof: self.parse_value()?,
                }
            }
            InstructionIdent::CreateProofFromAuthZoneOfNonFungibles => {
                Instruction::CreateProofFromAuthZoneOfNonFungibles {
                    resource_address: self.parse_value()?,
                    ids: self.parse_value()?,
                    new_proof: self.parse_value()?,
                }
            }
            InstructionIdent::CreateProofFromAuthZoneOfAll => {
                Instruction::CreateProofFromAuthZoneOfAll {
                    resource_address: self.parse_value()?,
                    new_proof: self.parse_value()?,
                }
            }

            InstructionIdent::CreateProofFromBucketOfAmount => {
                Instruction::CreateProofFromBucketOfAmount {
                    bucket: self.parse_value()?,
                    amount: self.parse_value()?,
                    new_proof: self.parse_value()?,
                }
            }
            InstructionIdent::CreateProofFromBucketOfNonFungibles => {
                Instruction::CreateProofFromBucketOfNonFungibles {
                    bucket: self.parse_value()?,
                    ids: self.parse_value()?,
                    new_proof: self.parse_value()?,
                }
            }
            InstructionIdent::CreateProofFromBucketOfAll => {
                Instruction::CreateProofFromBucketOfAll {
                    bucket: self.parse_value()?,
                    new_proof: self.parse_value()?,
                }
            }
            InstructionIdent::BurnResource => Instruction::BurnResource {
                bucket: self.parse_value()?,
            },

            InstructionIdent::CloneProof => Instruction::CloneProof {
                proof: self.parse_value()?,
                new_proof: self.parse_value()?,
            },
            InstructionIdent::DropProof => Instruction::DropProof {
                proof: self.parse_value()?,
            },
            InstructionIdent::CallFunction => Instruction::CallFunction {
                package_address: self.parse_value()?,
                blueprint_name: self.parse_value()?,
                function_name: self.parse_value()?,
                args: self.parse_values_till_semicolon()?,
            },
            InstructionIdent::CallMethod => Instruction::CallMethod {
                address: self.parse_value()?,
                method_name: self.parse_value()?,
                args: self.parse_values_till_semicolon()?,
            },
            InstructionIdent::CallRoyaltyMethod => Instruction::CallRoyaltyMethod {
                address: self.parse_value()?,
                method_name: self.parse_value()?,
                args: self.parse_values_till_semicolon()?,
            },
            InstructionIdent::CallMetadataMethod => Instruction::CallMetadataMethod {
                address: self.parse_value()?,
                method_name: self.parse_value()?,
                args: self.parse_values_till_semicolon()?,
            },
            InstructionIdent::CallRoleAssignmentMethod => Instruction::CallRoleAssignmentMethod {
                address: self.parse_value()?,
                method_name: self.parse_value()?,
                args: self.parse_values_till_semicolon()?,
            },
            InstructionIdent::CallDirectVaultMethod => Instruction::CallDirectVaultMethod {
                address: self.parse_value()?,
                method_name: self.parse_value()?,
                args: self.parse_values_till_semicolon()?,
            },
            InstructionIdent::DropNamedProofs => Instruction::DropNamedProofs,
            InstructionIdent::DropAllProofs => Instruction::DropAllProofs,
            InstructionIdent::AllocateGlobalAddress => Instruction::AllocateGlobalAddress {
                package_address: self.parse_value()?,
                blueprint_name: self.parse_value()?,
                address_reservation: self.parse_value()?,
                named_address: self.parse_value()?,
            },

            /* Call direct vault method aliases */
            InstructionIdent::RecallFromVault => Instruction::RecallFromVault {
                vault_id: self.parse_value()?,
                args: self.parse_values_till_semicolon()?,
            },
            InstructionIdent::FreezeVault => Instruction::FreezeVault {
                vault_id: self.parse_value()?,
                args: self.parse_values_till_semicolon()?,
            },
            InstructionIdent::UnfreezeVault => Instruction::UnfreezeVault {
                vault_id: self.parse_value()?,
                args: self.parse_values_till_semicolon()?,
            },
            InstructionIdent::RecallNonFungiblesFromVault => {
                Instruction::RecallNonFungiblesFromVault {
                    vault_id: self.parse_value()?,
                    args: self.parse_values_till_semicolon()?,
                }
            }

            /* Call function aliases */
            InstructionIdent::PublishPackage => Instruction::PublishPackage {
                args: self.parse_values_till_semicolon()?,
            },
            InstructionIdent::PublishPackageAdvanced => Instruction::PublishPackageAdvanced {
                args: self.parse_values_till_semicolon()?,
            },
            InstructionIdent::CreateFungibleResource => Instruction::CreateFungibleResource {
                args: self.parse_values_till_semicolon()?,
            },
            InstructionIdent::CreateFungibleResourceWithInitialSupply => {
                Instruction::CreateFungibleResourceWithInitialSupply {
                    args: self.parse_values_till_semicolon()?,
                }
            }
            InstructionIdent::CreateNonFungibleResource => Instruction::CreateNonFungibleResource {
                args: self.parse_values_till_semicolon()?,
            },
            InstructionIdent::CreateNonFungibleResourceWithInitialSupply => {
                Instruction::CreateNonFungibleResourceWithInitialSupply {
                    args: self.parse_values_till_semicolon()?,
                }
            }
            InstructionIdent::CreateAccessController => Instruction::CreateAccessController {
                args: self.parse_values_till_semicolon()?,
            },
            InstructionIdent::CreateIdentity => Instruction::CreateIdentity {
                args: self.parse_values_till_semicolon()?,
            },
            InstructionIdent::CreateIdentityAdvanced => Instruction::CreateIdentityAdvanced {
                args: self.parse_values_till_semicolon()?,
            },
            InstructionIdent::CreateAccount => Instruction::CreateAccount {
                args: self.parse_values_till_semicolon()?,
            },
            InstructionIdent::CreateAccountAdvanced => Instruction::CreateAccountAdvanced {
                args: self.parse_values_till_semicolon()?,
            },

            /* Call non-main method aliases */
            InstructionIdent::SetMetadata => Instruction::SetMetadata {
                address: self.parse_value()?,
                args: self.parse_values_till_semicolon()?,
            },
            InstructionIdent::RemoveMetadata => Instruction::RemoveMetadata {
                address: self.parse_value()?,
                args: self.parse_values_till_semicolon()?,
            },
            InstructionIdent::LockMetadata => Instruction::LockMetadata {
                address: self.parse_value()?,
                args: self.parse_values_till_semicolon()?,
            },
            InstructionIdent::SetComponentRoyalty => Instruction::SetComponentRoyalty {
                address: self.parse_value()?,
                args: self.parse_values_till_semicolon()?,
            },
            InstructionIdent::LockComponentRoyalty => Instruction::LockComponentRoyalty {
                address: self.parse_value()?,
                args: self.parse_values_till_semicolon()?,
            },
            InstructionIdent::ClaimComponentRoyalties => Instruction::ClaimComponentRoyalties {
                address: self.parse_value()?,
                args: self.parse_values_till_semicolon()?,
            },
            InstructionIdent::SetOwnerRole => Instruction::SetOwnerRole {
                address: self.parse_value()?,
                args: self.parse_values_till_semicolon()?,
            },
            InstructionIdent::LockOwnerRole => Instruction::LockOwnerRole {
                address: self.parse_value()?,
                args: self.parse_values_till_semicolon()?,
            },
            InstructionIdent::SetRole => Instruction::SetRole {
                address: self.parse_value()?,
                args: self.parse_values_till_semicolon()?,
            },

            /* Call main method aliases */
            InstructionIdent::MintFungible => Instruction::MintFungible {
                address: self.parse_value()?,
                args: self.parse_values_till_semicolon()?,
            },
            InstructionIdent::MintNonFungible => Instruction::MintNonFungible {
                address: self.parse_value()?,
                args: self.parse_values_till_semicolon()?,
            },
            InstructionIdent::MintRuidNonFungible => Instruction::MintRuidNonFungible {
                address: self.parse_value()?,
                args: self.parse_values_till_semicolon()?,
            },
            InstructionIdent::ClaimPackageRoyalties => Instruction::ClaimPackageRoyalties {
                address: self.parse_value()?,
                args: self.parse_values_till_semicolon()?,
            },
            InstructionIdent::CreateValidator => Instruction::CreateValidator {
                args: self.parse_values_till_semicolon()?,
            },
        };
        let instruction_end = self.peek()?.span.end;

        advance_match!(self, TokenKind::Semicolon);

        Ok(InstructionWithSpan {
            instruction,
            span: Span {
                start: instruction_start,
                end: instruction_end,
            },
        })
    }

    pub fn parse_value(&mut self) -> Result<ValueWithSpan, ParserError> {
        self.track_stack_depth_increase()?;
        let token = self.advance()?;
        let value = match &token.kind {
            // ==============
            // Basic Types
            // ==============
            TokenKind::BoolLiteral(value) => Value::Bool(*value),
            TokenKind::U8Literal(value) => Value::U8(*value),
            TokenKind::U16Literal(value) => Value::U16(*value),
            TokenKind::U32Literal(value) => Value::U32(*value),
            TokenKind::U64Literal(value) => Value::U64(*value),
            TokenKind::U128Literal(value) => Value::U128(*value),
            TokenKind::I8Literal(value) => Value::I8(*value),
            TokenKind::I16Literal(value) => Value::I16(*value),
            TokenKind::I32Literal(value) => Value::I32(*value),
            TokenKind::I64Literal(value) => Value::I64(*value),
            TokenKind::I128Literal(value) => Value::I128(*value),
            TokenKind::StringLiteral(value) => Value::String(value.clone()),
            TokenKind::Ident(ident_str) => {
                let value_ident =
                    SborValueIdent::from_ident(ident_str).ok_or(ParserError::UnexpectedToken {
                        expected: TokenType::Value,
                        actual: token.kind.clone(),
                        span: token.span,
                    })?;
                match value_ident {
                    SborValueIdent::Enum => self.parse_enum_content()?,
                    SborValueIdent::Array => self.parse_array_content()?,
                    SborValueIdent::Tuple => self.parse_tuple_content()?,
                    SborValueIdent::Map => self.parse_map_content()?,

                    // ==============
                    // Aliases
                    // ==============
                    SborValueIdent::Some => Value::Some(Box::new(self.parse_values_one()?)),
                    SborValueIdent::None => Value::None,
                    SborValueIdent::Ok => Value::Ok(Box::new(self.parse_values_one()?)),
                    SborValueIdent::Err => Value::Err(Box::new(self.parse_values_one()?)),
                    SborValueIdent::Bytes => Value::Bytes(Box::new(self.parse_values_one()?)),
                    SborValueIdent::NonFungibleGlobalId => {
                        Value::NonFungibleGlobalId(Box::new(self.parse_values_one()?))
                    }

                    // ==============
                    // Custom Types
                    // ==============
                    SborValueIdent::Address => Value::Address(self.parse_values_one()?.into()),
                    SborValueIdent::Bucket => Value::Bucket(self.parse_values_one()?.into()),
                    SborValueIdent::Proof => Value::Proof(self.parse_values_one()?.into()),
                    SborValueIdent::Expression => {
                        Value::Expression(self.parse_values_one()?.into())
                    }
                    SborValueIdent::Blob => Value::Blob(self.parse_values_one()?.into()),
                    SborValueIdent::Decimal => Value::Decimal(self.parse_values_one()?.into()),
                    SborValueIdent::PreciseDecimal => {
                        Value::PreciseDecimal(self.parse_values_one()?.into())
                    }
                    SborValueIdent::NonFungibleLocalId => {
                        Value::NonFungibleLocalId(self.parse_values_one()?.into())
                    }
                    SborValueIdent::AddressReservation => {
                        Value::AddressReservation(self.parse_values_one()?.into())
                    }
                    SborValueIdent::NamedAddress => {
                        Value::NamedAddress(self.parse_values_one()?.into())
                    }
                }
            }
            _ => {
                return Err(ParserError::UnexpectedToken {
                    expected: TokenType::Value,
                    actual: token.kind,
                    span: token.span,
                });
            }
        };
        self.track_stack_depth_decrease()?;
        Ok(ValueWithSpan {
            value,
            span: token.span,
        })
    }

    pub fn parse_enum_content(&mut self) -> Result<Value, ParserError> {
        advance_match!(self, TokenKind::LessThan);
        let discriminator_token = self.advance()?;
        let discriminator = match discriminator_token.kind {
            TokenKind::U8Literal(discriminator) => discriminator,
            TokenKind::Ident(discriminator) => KNOWN_ENUM_DISCRIMINATORS
                .get(discriminator.as_str())
                .cloned()
                .ok_or(ParserError::UnknownEnumDiscriminator {
                    actual: discriminator.clone(),
                    span: discriminator_token.span,
                })?,
            _ => {
                return Err(ParserError::UnexpectedToken {
                    expected: TokenType::EnumDiscriminator,
                    actual: discriminator_token.kind,
                    span: discriminator_token.span,
                })
            }
        };
        advance_match!(self, TokenKind::GreaterThan);

        let fields =
            self.parse_values_any(TokenKind::OpenParenthesis, TokenKind::CloseParenthesis)?;

        Ok(Value::Enum(discriminator, fields))
    }

    pub fn parse_array_content(&mut self) -> Result<Value, ParserError> {
        let generics = self.parse_generics(1)?;
        Ok(Value::Array(
            generics[0].clone(),
            self.parse_values_any(TokenKind::OpenParenthesis, TokenKind::CloseParenthesis)?,
        ))
    }

    pub fn parse_tuple_content(&mut self) -> Result<Value, ParserError> {
        Ok(Value::Tuple(self.parse_values_any(
            TokenKind::OpenParenthesis,
            TokenKind::CloseParenthesis,
        )?))
    }

    pub fn parse_map_content(&mut self) -> Result<Value, ParserError> {
        let generics = self.parse_generics(2)?;
        advance_match!(self, TokenKind::OpenParenthesis);
        let mut entries = Vec::new();
        while self.peek()?.kind != TokenKind::CloseParenthesis {
            let key = self.parse_value()?;
            advance_match!(self, TokenKind::FatArrow);
            let value = self.parse_value()?;
            entries.push((key, value));
            if self.peek()?.kind != TokenKind::CloseParenthesis {
                advance_match!(self, TokenKind::Comma);
            }
        }
        advance_match!(self, TokenKind::CloseParenthesis);
        Ok(Value::Map(
            generics[0].clone(),
            generics[1].clone(),
            entries,
        ))
    }

    /// Parse a comma-separated value list, enclosed by a pair of marks.
    fn parse_values_any(
        &mut self,
        open: TokenKind,
        close: TokenKind,
    ) -> Result<Vec<ValueWithSpan>, ParserError> {
        advance_match!(self, open);
        let mut values = Vec::new();
        while self.peek()?.kind != close {
            values.push(self.parse_value()?);
            if self.peek()?.kind != close {
                advance_match!(self, TokenKind::Comma);
            }
        }
        advance_match!(self, close);
        Ok(values)
    }

    fn parse_values_one(&mut self) -> Result<ValueWithSpan, ParserError> {
        let values =
            self.parse_values_any(TokenKind::OpenParenthesis, TokenKind::CloseParenthesis)?;
        if values.len() != 1 {
            Err(ParserError::InvalidNumberOfValues {
                actual: values.len(),
                expected: 1,
                span: Span {
                    start: values[0].span.start,
                    end: values[values.len() - 1].span.end,
                },
            })
        } else {
            Ok(values[0].clone())
        }
    }

    fn parse_generics(&mut self, n: usize) -> Result<Vec<ValueKindWithSpan>, ParserError> {
        advance_match!(self, TokenKind::LessThan);
        let mut types = Vec::new();
        while self.peek()?.kind != TokenKind::GreaterThan {
            let token_value_kind = self.parse_type()?;
            types.push(token_value_kind);
            if self.peek()?.kind != TokenKind::GreaterThan {
                advance_match!(self, TokenKind::Comma);
            }
        }
        advance_match!(self, TokenKind::GreaterThan);

        if types.len() != n {
            Err(ParserError::InvalidNumberOfTypes {
                expected: n,
                actual: types.len(),
                span: Span {
                    start: types[0].span.start,
                    end: types[types.len() - 1].span.end,
                },
            })
        } else {
            Ok(types)
        }
    }

    fn parse_type(&mut self) -> Result<ValueKindWithSpan, ParserError> {
        let token = self.advance()?;
        let value_kind = match &token.kind {
            TokenKind::Ident(ident_str) => {
                let value_kind_ident = SborValueKindIdent::from_ident(&ident_str).ok_or(
                    ParserError::UnexpectedToken {
                        expected: TokenType::ValueKind,
                        actual: token.kind.clone(),
                        span: token.span,
                    },
                )?;
                match value_kind_ident {
                    // ==============
                    // Simple basic value kinds
                    // ==============
                    SborValueKindIdent::Bool => ValueKind::Bool,
                    SborValueKindIdent::I8 => ValueKind::I8,
                    SborValueKindIdent::I16 => ValueKind::I16,
                    SborValueKindIdent::I32 => ValueKind::I32,
                    SborValueKindIdent::I64 => ValueKind::I64,
                    SborValueKindIdent::I128 => ValueKind::I128,
                    SborValueKindIdent::U8 => ValueKind::U8,
                    SborValueKindIdent::U16 => ValueKind::U16,
                    SborValueKindIdent::U32 => ValueKind::U32,
                    SborValueKindIdent::U64 => ValueKind::U64,
                    SborValueKindIdent::U128 => ValueKind::U128,
                    SborValueKindIdent::String => ValueKind::String,

                    // ==============
                    // Composite basic value kinds
                    // ==============
                    SborValueKindIdent::Enum => ValueKind::Enum,
                    SborValueKindIdent::Array => ValueKind::Array,
                    SborValueKindIdent::Tuple => ValueKind::Tuple,
                    SborValueKindIdent::Map => ValueKind::Map,

                    // ==============
                    // Value kind aliases
                    // ==============
                    SborValueKindIdent::Bytes => ValueKind::Bytes,
                    SborValueKindIdent::NonFungibleGlobalId => ValueKind::NonFungibleGlobalId,

                    // ==============
                    // Custom value kinds
                    // ==============
                    SborValueKindIdent::Address => ValueKind::Address,
                    SborValueKindIdent::Bucket => ValueKind::Bucket,
                    SborValueKindIdent::Proof => ValueKind::Proof,
                    SborValueKindIdent::Expression => ValueKind::Expression,
                    SborValueKindIdent::Blob => ValueKind::Blob,
                    SborValueKindIdent::Decimal => ValueKind::Decimal,
                    SborValueKindIdent::PreciseDecimal => ValueKind::PreciseDecimal,
                    SborValueKindIdent::NonFungibleLocalId => ValueKind::NonFungibleLocalId,
                    SborValueKindIdent::AddressReservation => ValueKind::AddressReservation,
                    SborValueKindIdent::NamedAddress => ValueKind::NamedAddress,
                }
            }
            _ => {
                return Err(ParserError::UnexpectedToken {
                    expected: TokenType::ValueKind,
                    actual: token.kind,
                    span: token.span,
                });
            }
        };
        Ok(ValueKindWithSpan {
            value_kind,
            span: token.span,
        })
    }
}

pub fn parser_error_diagnostics(s: &str, err: ParserError) -> String {
    let lines_cnt = s.lines().count();
    // println!("err = {:?}", err);
    let (span, title, label) = match err {
        ParserError::UnexpectedEof => (
            Span {
                start: Position {
                    full_index: s.len() - 1,
                    line_number: lines_cnt,
                    line_char_index: 0,
                },
                end: Position {
                    full_index: s.len(),
                    line_number: lines_cnt,
                    line_char_index: 0,
                },
            },
            "unexpected end of file".to_string(),
            "end of file".to_string(),
        ),
        ParserError::UnexpectedToken {
            expected,
            actual,
            span,
        } => {
            let title = format!("expected {}, found {}", expected, actual);
            (span, title, "unexpected token".to_string())
        }
        ParserError::UnexpectedTokenOrMissingSemicolon {
            expected,
            actual,
            span,
        } => {
            let title = format!("expected `;` or {}, found {}", expected, actual);
            (span, title, "unexpected token".to_string())
        }
        ParserError::InvalidNumberOfValues {
            expected,
            actual,
            span,
        } => {
            let title = format!("expected {} number of values, found {}", expected, actual);
            (span, title, "invalid number of values".to_string())
        }
        ParserError::InvalidNumberOfTypes {
            expected,
            actual,
            span,
        } => {
            let title = format!("expected {} number of types, found {}", expected, actual);
            (span, title, "invalid number of types".to_string())
        }
        ParserError::MaxDepthExceeded { span, actual, max } => {
            let title = format!("manifest actual depth {} exceeded max {}", actual, max);
            (span, title, "max depth exceeded".to_string())
        }
        ParserError::UnknownEnumDiscriminator { actual, span } => {
            let title = format!("unknown enum discriminator found `{}`", actual);
            (span, title, "unknown enum discriminator".to_string())
        }
    };

    create_snippet(s, &span, &title, &label)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::manifest::lexer::tokenize;
    use crate::manifest::token::{Position, Span};

    #[macro_export]
    macro_rules! parse_instruction_ok {
        ( $s:expr, $expected:expr ) => {{
            let mut parser = Parser::new(tokenize($s).unwrap()), PARSER_MAX_DEPTH;
            assert_eq!(parser.parse_instruction(), Ok($expected));
            assert!(parser.is_eof());
        }};
    }

    #[macro_export]
    macro_rules! parse_value_ok {
        ( $s:expr, $expected:expr ) => {{
            let mut parser = Parser::new(tokenize($s).unwrap(), PARSER_MAX_DEPTH);
            assert_eq!(parser.parse_value().map(|tv| tv.value), Ok($expected));
            assert!(parser.is_eof());
        }};
    }

    #[macro_export]
    macro_rules! parse_value_error {
        ( $s:expr, $expected:expr ) => {{
            let mut parser = Parser::new(tokenize($s).unwrap(), PARSER_MAX_DEPTH);
            match parser.parse_value() {
                Ok(_) => {
                    panic!("Expected {:?} but no error is thrown", $expected);
                }
                Err(e) => {
                    assert_eq!(e, $expected);
                }
            }
        }};
    }

    #[test]
    fn test_literals() {
        parse_value_ok!(r#"true"#, Value::Bool(true));
        parse_value_ok!(r#"false"#, Value::Bool(false));
        parse_value_ok!(r#"1i8"#, Value::I8(1));
        parse_value_ok!(r#"1i16"#, Value::I16(1));
        parse_value_ok!(r#"1i32"#, Value::I32(1));
        parse_value_ok!(r#"1i64"#, Value::I64(1));
        parse_value_ok!(r#"1i128"#, Value::I128(1));
        parse_value_ok!(r#"1u8"#, Value::U8(1));
        parse_value_ok!(r#"1u16"#, Value::U16(1));
        parse_value_ok!(r#"1u32"#, Value::U32(1));
        parse_value_ok!(r#"1u64"#, Value::U64(1));
        parse_value_ok!(r#"1u128"#, Value::U128(1));
        parse_value_ok!(r#""test""#, Value::String("test".into()));
    }

    macro_rules! position {
        ($full_index:expr, $line_number:expr, $line_char_index:expr) => {
            Position {
                full_index: $full_index,
                line_number: $line_number,
                line_char_index: $line_char_index,
            }
        };
    }
    macro_rules! span {
        (start = ($st_full_index:expr, $st_line_number:expr, $st_line_char_index:expr),
         end = ($end_full_index:expr, $end_line_number:expr, $end_line_char_index:expr)) => {
            Span {
                start: position!($st_full_index, $st_line_number, $st_line_char_index),
                end: position!($end_full_index, $end_line_number, $end_line_char_index),
            }
        };
    }
    #[test]
    fn test_enum() {
        parse_value_ok!(
            r#"Enum<0u8>("Hello", 123u8)"#,
            Value::Enum(
                0,
                vec![
                    ValueWithSpan {
                        value: Value::String("Hello".into()),
                        span: span!(start = (10, 1, 10), end = (17, 1, 17)),
                    },
                    ValueWithSpan {
                        value: Value::U8(123),
                        span: span!(start = (19, 1, 19), end = (24, 1, 24)),
                    },
                ],
            )
        );
        parse_value_ok!(r#"Enum<0u8>()"#, Value::Enum(0, Vec::new()));
        parse_value_ok!(
            r#"Enum<PublicKey::Secp256k1>()"#,
            Value::Enum(0, Vec::new())
        );
        // Check we allow trailing commas
        parse_value_ok!(
            r#"Enum<0u8>("Hello", 123u8,)"#,
            Value::Enum(
                0,
                vec![
                    ValueWithSpan {
                        value: Value::String("Hello".into()),
                        span: span!(start = (10, 1, 10), end = (17, 1, 17)),
                    },
                    ValueWithSpan {
                        value: Value::U8(123),
                        span: span!(start = (19, 1, 19), end = (24, 1, 24)),
                    },
                ],
            )
        );
    }

    #[test]
    fn test_array() {
        parse_value_ok!(
            r#"Array<U8>(1u8, 2u8)"#,
            Value::Array(
                ValueKindWithSpan {
                    value_kind: ValueKind::U8,
                    span: span!(start = (6, 1, 6), end = (8, 1, 8)),
                },
                vec![
                    ValueWithSpan {
                        value: Value::U8(1),
                        span: span!(start = (10, 1, 10), end = (13, 1, 13)),
                    },
                    ValueWithSpan {
                        value: Value::U8(2),
                        span: span!(start = (15, 1, 15), end = (18, 1, 18)),
                    }
                ],
            )
        );
        parse_value_ok!(
            r#"Array<U8>()"#,
            Value::Array(
                ValueKindWithSpan {
                    value_kind: ValueKind::U8,
                    span: span!(start = (6, 1, 6), end = (8, 1, 8)),
                },
                vec![]
            )
        );
        // Check we allow trailing commas
        parse_value_ok!(
            r#"Array<U8>(1u8, 2u8,)"#,
            Value::Array(
                ValueKindWithSpan {
                    value_kind: ValueKind::U8,
                    span: span!(start = (6, 1, 6), end = (8, 1, 8)),
                },
                vec![
                    ValueWithSpan {
                        value: Value::U8(1),
                        span: span!(start = (10, 1, 10), end = (13, 1, 13)),
                    },
                    ValueWithSpan {
                        value: Value::U8(2),
                        span: span!(start = (15, 1, 15), end = (18, 1, 18)),
                    }
                ],
            )
        );
    }

    #[test]
    fn test_tuple() {
        parse_value_ok!(r#"Tuple()"#, Value::Tuple(vec![]));
        parse_value_ok!(
            r#"Tuple("Hello", 123u8)"#,
            Value::Tuple(vec![
                ValueWithSpan {
                    value: Value::String("Hello".into()),
                    span: span!(start = (6, 1, 6), end = (13, 1, 13)),
                },
                ValueWithSpan {
                    value: Value::U8(123),
                    span: span!(start = (15, 1, 15), end = (20, 1, 20)),
                },
            ])
        );
        parse_value_ok!(
            r#"Tuple(1u8, 2u8)"#,
            Value::Tuple(vec![
                ValueWithSpan {
                    value: Value::U8(1),
                    span: span!(start = (6, 1, 6), end = (9, 1, 9)),
                },
                ValueWithSpan {
                    value: Value::U8(2),
                    span: span!(start = (11, 1, 11), end = (14, 1, 14)),
                },
            ])
        );

        // Check we allow trailing commas
        parse_value_ok!(
            r#"Tuple(1u8, 2u8,)"#,
            Value::Tuple(vec![
                ValueWithSpan {
                    value: Value::U8(1),
                    span: span!(start = (6, 1, 6), end = (9, 1, 9)),
                },
                ValueWithSpan {
                    value: Value::U8(2),
                    span: span!(start = (11, 1, 11), end = (14, 1, 14)),
                },
            ])
        );
    }

    #[test]
    fn test_map() {
        parse_value_ok!(
            r#"Map<String, U8>("Hello" => 123u8)"#,
            Value::Map(
                ValueKindWithSpan {
                    value_kind: ValueKind::String,
                    span: span!(start = (4, 1, 4), end = (10, 1, 10)),
                },
                ValueKindWithSpan {
                    value_kind: ValueKind::U8,
                    span: span!(start = (12, 1, 12), end = (14, 1, 14)),
                },
                vec![(
                    ValueWithSpan {
                        value: Value::String("Hello".into()),
                        span: span!(start = (16, 1, 16), end = (23, 1, 23)),
                    },
                    ValueWithSpan {
                        value: Value::U8(123),
                        span: span!(start = (27, 1, 27), end = (32, 1, 32)),
                    }
                )]
            )
        );
        parse_value_ok!(
            r#"Map<String, U8>("Hello" => 123u8, "world!" => 1u8)"#,
            Value::Map(
                ValueKindWithSpan {
                    value_kind: ValueKind::String,
                    span: span!(start = (4, 1, 4), end = (10, 1, 10)),
                },
                ValueKindWithSpan {
                    value_kind: ValueKind::U8,
                    span: span!(start = (12, 1, 12), end = (14, 1, 14)),
                },
                vec![
                    (
                        ValueWithSpan {
                            value: Value::String("Hello".into()),
                            span: span!(start = (16, 1, 16), end = (23, 1, 23)),
                        },
                        ValueWithSpan {
                            value: Value::U8(123),
                            span: span!(start = (27, 1, 27), end = (32, 1, 32)),
                        }
                    ),
                    (
                        ValueWithSpan {
                            value: Value::String("world!".into()),
                            span: span!(start = (34, 1, 34), end = (42, 1, 42)),
                        },
                        ValueWithSpan {
                            value: Value::U8(1),
                            span: span!(start = (46, 1, 46), end = (49, 1, 49)),
                        }
                    )
                ]
            )
        );

        // Check we allow trailing commas
        parse_value_ok!(
            r#"Map<String, U8>("Hello" => 123u8, "world!" => 1u8,)"#,
            Value::Map(
                ValueKindWithSpan {
                    value_kind: ValueKind::String,
                    span: span!(start = (4, 1, 4), end = (10, 1, 10)),
                },
                ValueKindWithSpan {
                    value_kind: ValueKind::U8,
                    span: span!(start = (12, 1, 12), end = (14, 1, 14)),
                },
                vec![
                    (
                        ValueWithSpan {
                            value: Value::String("Hello".into()),
                            span: span!(start = (16, 1, 16), end = (23, 1, 23)),
                        },
                        ValueWithSpan {
                            value: Value::U8(123),
                            span: span!(start = (27, 1, 27), end = (32, 1, 32)),
                        }
                    ),
                    (
                        ValueWithSpan {
                            value: Value::String("world!".into()),
                            span: span!(start = (34, 1, 34), end = (42, 1, 42)),
                        },
                        ValueWithSpan {
                            value: Value::U8(1),
                            span: span!(start = (46, 1, 46), end = (49, 1, 49)),
                        }
                    )
                ]
            )
        );
    }

    #[test]
    fn test_failures() {
        parse_value_error!(r#"Enum<0u8"#, ParserError::UnexpectedEof);
        parse_value_error!(
            r#"Enum<0u8)"#,
            ParserError::UnexpectedToken {
                expected: TokenType::Exact(TokenKind::GreaterThan),
                actual: TokenKind::CloseParenthesis,
                span: Span {
                    start: Position {
                        full_index: 8,
                        line_number: 1,
                        line_char_index: 8,
                    },
                    end: Position {
                        full_index: 9,
                        line_number: 1,
                        line_char_index: 9,
                    }
                },
            }
        );
        parse_value_error!(
            r#"Address("abc", "def")"#,
            ParserError::InvalidNumberOfValues {
                actual: 2,
                expected: 1,
                span: Span {
                    start: Position {
                        full_index: 8,
                        line_number: 1,
                        line_char_index: 8,
                    },
                    end: Position {
                        full_index: 20,
                        line_number: 1,
                        line_char_index: 20,
                    }
                }
            }
        );
    }

    #[test]
    fn test_deep_value_does_not_panic_with_stack_overflow() {
        let depth: usize = 1000;
        let mut value_string = "".to_string();
        for _ in 0..depth {
            value_string.push_str("Tuple(");
        }
        value_string.push_str("0u8");
        for _ in 0..depth {
            value_string.push_str(")");
        }

        // Should actually be an error not a panic
        parse_value_error!(
            &value_string,
            ParserError::MaxDepthExceeded {
                actual: 21,
                max: 20,
                span: Span {
                    start: Position {
                        full_index: 120,
                        line_number: 1,
                        line_char_index: 120
                    },
                    end: Position {
                        full_index: 125,
                        line_number: 1,
                        line_char_index: 125
                    }
                }
            }
        );
    }

    // Instruction parsing tests have been removed as they're largely outdated (inconsistent with the data model),
    // which may lead developers to invalid syntax.
    //
    // It's also not very useful as instruction parsing basically calls `parse_value` recursively
    //
    // That said, all manifest instructions should be tested in `generator.rs` and `e2e.rs`.
}
