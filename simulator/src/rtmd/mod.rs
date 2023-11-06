use clap::Parser;
use radix_engine::types::*;
use radix_engine::utils::validate_call_arguments_to_native_components;
use radix_engine_interface::crypto::hash;
use radix_engine_interface::data::manifest::manifest_decode;
use std::path::PathBuf;
use std::str::FromStr;
use transaction::manifest::decompile;
use transaction::prelude::*;

/// Radix transaction manifest decompiler
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None, name = "rtmd")]
pub struct Args {
    /// Path to the output file
    #[clap(short, long)]
    output: PathBuf,

    /// Network to Use [Simulator | Alphanet | Mainnet]
    #[clap(short, long)]
    network: Option<String>,

    /// Whether to export blobs
    #[clap(short, long, action)]
    export_blobs: bool,

    /// Input file
    #[clap(required = true)]
    input: PathBuf,
}

#[derive(Debug)]
pub enum Error {
    IoError(std::io::Error),
    DecodeError(sbor::DecodeError),
    DecompileError(transaction::manifest::DecompileError),
    ParseNetworkError(ParseNetworkError),
    InstructionSchemaValidationError(radix_engine::utils::LocatedInstructionSchemaValidationError),
}

pub fn run() -> Result<(), Error> {
    let args = Args::parse();

    let content = std::fs::read(&args.input).map_err(Error::IoError)?;
    let network = match args.network {
        Some(n) => NetworkDefinition::from_str(&n).map_err(Error::ParseNetworkError)?,
        None => NetworkDefinition::simulator(),
    };

    match manifest_decode::<TransactionManifestV1>(&content).map_err(Error::DecodeError) {
        Ok(manifest) => {
            validate_call_arguments_to_native_components(&manifest.instructions)
                .map_err(Error::InstructionSchemaValidationError)?;

            let result =
                decompile(&manifest.instructions, &network).map_err(Error::DecompileError)?;
            std::fs::write(&args.output, &result).map_err(Error::IoError)?;

            if args.export_blobs {
                let directory = args.output.parent().unwrap();
                for blob in manifest.blobs.values() {
                    let blob_hash = hash(&blob);
                    std::fs::write(directory.join(format!("{}.blob", blob_hash)), &blob)
                        .map_err(Error::IoError)?;
                }
            }

            Ok(())
        }
        Err(e) => {
            // try to decode versioned transaction
            match manifest_decode::<VersionedTransactionPayload>(&content) {
                Ok(manifest) => {
                    let (manifest_instructions, blobs) = match manifest {
                        VersionedTransactionPayload::IntentV1 {
                            instructions,
                            blobs,
                            ..
                        } => (instructions.0, blobs.blobs),
                        VersionedTransactionPayload::SignedIntentV1 { intent, .. } => {
                            (intent.instructions.0, intent.blobs.blobs)
                        }
                        VersionedTransactionPayload::NotarizedTransactionV1 {
                            signed_intent,
                            ..
                        } => (
                            signed_intent.intent.instructions.0,
                            signed_intent.intent.blobs.blobs,
                        ),
                        VersionedTransactionPayload::SystemTransactionV1 {
                            instructions,
                            blobs,
                            ..
                        } => (instructions.0, blobs.blobs),
                    };

                    validate_call_arguments_to_native_components(&manifest_instructions)
                        .map_err(Error::InstructionSchemaValidationError)?;

                    let result = decompile(&manifest_instructions, &network)
                        .map_err(Error::DecompileError)?;
                    std::fs::write(&args.output, &result).map_err(Error::IoError)?;

                    if args.export_blobs {
                        let directory = args.output.parent().unwrap();
                        for blob in blobs {
                            let blob_hash = hash(&blob.0);
                            std::fs::write(directory.join(format!("{}.blob", blob_hash)), &blob.0)
                                .map_err(Error::IoError)?;
                        }
                    }

                    Ok(())
                }
                Err(_) => {
                    // return original error
                    Err(e)
                }
            }
        }
    }
}
