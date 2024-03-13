use crate::internal_prelude::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CompileError {
    LexerError(lexer::LexerError),
    ParserError(parser::ParserError),
    GeneratorError(generator::GeneratorError),
}

pub fn compile_error_diagnostics(s: &str, err: CompileError) -> String {
    match err {
        CompileError::LexerError(err) => lexer::lexer_error_diagnostics(s, err),
        CompileError::ParserError(err) => parser::parser_error_diagnostics(s, err),
        CompileError::GeneratorError(err) => generator::generator_error_diagnostics(s, err),
    }
}

pub fn compile<B>(
    s: &str,
    network: &NetworkDefinition,
    blobs: B,
) -> Result<TransactionManifestV1, CompileError>
where
    B: IsBlobProvider,
{
    let address_bech32_decoder = AddressBech32Decoder::new(network);

    let tokens = lexer::tokenize(s).map_err(CompileError::LexerError)?;
    let instructions = parser::Parser::new(tokens, parser::PARSER_MAX_DEPTH)
        .parse_manifest()
        .map_err(CompileError::ParserError)?;
    generator::generate_manifest(&instructions, &address_bech32_decoder, blobs)
        .map_err(CompileError::GeneratorError)
}
