pub mod parser;
mod diagnostics;
mod item;
mod stmt;
mod expr;
mod ty;
mod path;
mod pat;

use std::path::PathBuf;
use std::rc::Rc;

use crate::stelaro_ast::token::TokenStream;
use crate::stelaro_common::source_map::SourceFile;
use crate::stelaro_diagnostics::ErrorEmitted;
use crate::stelaro_lexer::Lexer;
use crate::stelaro_session::ParseSess;

use parser::Parser;


type PResult<T> = Result<T, ErrorEmitted>;

pub fn new_parser_from_source_str(
    psess: &ParseSess,
    name: PathBuf,
    source: String,
) -> Result<Parser<'_>, ErrorEmitted> {
    let source_file = psess.source_map().new_source_file(name, source);
    new_parser_from_source_file(psess, source_file)
}

pub fn new_parser_from_file<'sess>(psess: &'sess ParseSess, path: &std::path::Path) -> Result<Parser<'sess>, ErrorEmitted> {
    let file = psess.source_map().load_file(path).unwrap_or_else(|e| {
        psess.dcx().emit_fatal(format!("{e}"))
    });

    let mut lexer = Lexer::new(psess, file.src.as_ref());

    match lexer.lex() {
        Ok(ts) => Ok(Parser::new(psess, ts)),
        Err(err) => Err(err)?,
    }
}

fn new_parser_from_source_file(
    psess: &ParseSess,
    source_file: Rc<SourceFile>,
) -> Result<Parser<'_>, ErrorEmitted> {
    let stream = source_file_to_stream(psess, source_file)?;
    let parser = Parser::new(psess, stream);

    Ok(parser)
}

fn source_file_to_stream(
    psess: &ParseSess,
    source_file: Rc<SourceFile>,
) -> Result<TokenStream, ErrorEmitted> {
    let mut lexer = Lexer::new(psess, source_file.src.as_ref());

    match lexer.lex() {
        Ok(ts) => Ok(ts),
        Err(err) => Err(err)?,
    }
}
