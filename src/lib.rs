#![feature(array_windows)]
#![feature(associated_type_defaults)]
#![feature(box_patterns)]
#![feature(debug_closure_helpers)]
#![feature(never_type)]
#![feature(min_specialization)]

pub mod stelaro_ast;
pub mod stelaro_ast_lowering;
pub mod stelaro_codegen;
pub mod stelaro_common;
pub mod stelaro_context;
pub mod stelaro_diagnostics;
pub mod stelaro_interface;
pub mod stelaro_lexer;
pub mod stelaro_parse;
pub mod stelaro_resolve;
pub mod stelaro_session;
pub mod stelaro_sir;
pub mod stelaro_sir_typecheck;
pub mod stelaro_ty;

use clap::Parser;
use std::path::PathBuf;

use crate::stelaro_ast_lowering::lower_to_sir;
use crate::stelaro_interface::passes::create_and_enter_global_ctxt;
use crate::stelaro_interface::{interface, passes};
use crate::stelaro_resolve::{Resolver, ResolverArenas};
use crate::stelaro_session::{Input, config};

#[derive(Parser, Debug)]
#[command(version)]
pub(crate) struct Args {
    input_file: PathBuf,

    #[arg(short, long)]
    output: Option<PathBuf>,

    #[arg(short = 'd', long)]
    output_dir: Option<PathBuf>,

    #[arg(long)]
    stelo_name: Option<String>,
}

pub fn run() {
    let args = Args::parse();
    let opts = config::build_session_options(&args);

    let input = Input::File(args.input_file);
    let odir = args.output_dir;
    let ofile = args.output;

    let config = interface::Config {
        opts,
        input,
        output_dir: odir,
        output_file: ofile,
        file_loader: None,
    };

    stelaro_interface::run_compiler(config, |sess| {
        let stelo = passes::parse(sess);

        create_and_enter_global_ctxt(sess, |tcx| {
            let arenas = &ResolverArenas::default();

            let mut resolver = Resolver::new(
                tcx,
                stelo.span.inner_span,
                arenas,
            );

            resolver.resolve_stelo(&stelo);

            let resolver = resolver.into_outputs().ast_lowering;
            let sir_stelo = lower_to_sir(tcx, resolver, stelo);
            let stelo = tcx.sir_arena.alloc(sir_stelo);
            tcx.sir_stelo.replace(Some(stelo));

            sess.dcx().abort_if_errors();

            dbg!(&tcx.sir_stelo.borrow().unwrap());
        });
    });
}
