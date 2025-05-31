#![feature(associated_type_defaults)]
#![feature(never_type)]
#![feature(min_specialization)]
#![allow(clippy::should_implement_trait)]
#![allow(clippy::too_many_arguments)]
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


use stelaro_common::{create_session_globals_then, RealFileLoader, Arena, SourceMapInputs, StableSteloId, Symbol};
use stelaro_context::TyCtxt;
use stelaro_parse::new_parser_from_source_str;
use stelaro_resolve::{Resolver, ResolverArenas};
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
        let name = "temp";

        let mut parser = new_parser_from_source_str(
            &sess.psess,
            name.into(),
            src,
        ).unwrap();

        let stelo = parser.parse_stelo().unwrap();
        let arena = Arena::new();
        let stable_stelo_id = StableSteloId::new(Symbol::intern(name));

        let gcx = &TyCtxt::create_global_ctxt(
            &sess,
            stable_stelo_id,
            &arena,
        );

        let tcx = TyCtxt::new(gcx);
        let arenas = &ResolverArenas::default();

        let mut resolver = Resolver::new(
            tcx,
            stelo.span.inner_span,
            arenas,
        );

        resolver.resolve_stelo(&stelo);

        // dbg!(stelo);
    });
}


pub fn run() {
}