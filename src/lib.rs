#![feature(associated_type_defaults)]
#![feature(never_type)]
#![feature(min_specialization)]
#![allow(clippy::should_implement_trait)]
#![allow(clippy::type_complexity)]

pub mod stelaro_ast;
pub mod stelaro_common;
pub mod stelaro_context;
pub mod stelaro_diagnostic;
pub mod stelaro_interface;
pub mod stelaro_lexer;
pub mod stelaro_parse;
pub mod stelaro_resolve;
pub mod stelaro_session;
pub mod stelaro_sir;
pub mod stelaro_ty;

use stelaro_common::{create_session_globals_then, source_map::RealFileLoader, SourceMapInputs};
use stelaro_parse::new_parser_from_source_str;
use stelaro_session::{config::Input, session::{build_session, CompilerPaths}, Session};

pub fn temp(src: String) {
    let paths = CompilerPaths {
        input: Input::Str {
            name: "temp".to_string(),
            input: src.to_string()
        },
        output_dir: None,
        output_file: None,
        temps_dir: None,
    };

    let file_loader = Box::new(RealFileLoader);

    create_session_globals_then(Some(SourceMapInputs { file_loader }),|| {
        let sess = build_session(paths);

        let mut parser = new_parser_from_source_str(
            &sess.psess,
            "temp".into(),
            src,
        ).unwrap();

        let stelo = parser.parse_stelo().unwrap();

        dbg!(stelo);
    });
}


pub fn run() {
}