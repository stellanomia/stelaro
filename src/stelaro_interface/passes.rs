use crate::stelaro_ast::ast;
use crate::stelaro_common::{sym, Arena, StableSteloId, Symbol};
use crate::stelaro_context::TyCtxt;
use crate::stelaro_parse::{new_parser_from_file, new_parser_from_source_str};
use crate::stelaro_session::{config::Input, Session};


pub fn parse(sess: &Session) -> ast::Stelo {
    let parser = match &sess.paths.input {
        Input::File(file) => new_parser_from_file(&sess.psess, file),
        Input::Str { input, name } => {
            new_parser_from_source_str(&sess.psess, name.into(), input.clone())
        }
    };

    match parser.and_then(|mut parser| parser.parse_stelo()) {
        Ok(stelo) => stelo,
        Err(err) => err.raise_fatal(),
    }
}

pub fn create_and_enter_global_ctxt<T, F: for<'tcx> FnOnce(TyCtxt<'tcx>) -> T>(
    sess: &Session,
    f: F,
) -> T {
    let stelo_name = get_stelo_name(sess);

    let arena = Arena::new();
    let sir_arena = Arena::new();
    let stable_stelo_id = StableSteloId::new(stelo_name);

    let gcx = TyCtxt::create_global_ctxt(
        sess,
        stable_stelo_id,
        &arena,
        &sir_arena,
    );

    let tcx = TyCtxt::new(&gcx);

    f(tcx)
}

pub fn get_stelo_name(sess: &Session) -> Symbol {
    if let Some(stelo_name) = &sess.opts.stelo_name {
        return Symbol::intern(stelo_name);
    }

    if let Input::File(ref path) = sess.paths.input
        && let Some(file_stem) = path.file_stem().and_then(|s| s.to_str())
    {
        if file_stem.starts_with('-') {
            sess.dcx().emit_fatal(
                format!("クレート名は `-` で始まってはいけませんが、`{file_stem}` は先頭にハイフンがあります")
            );
        } else {
            return Symbol::intern(&file_stem.replace('-', "_"));
        }
    }

    sym::STELARO_OUT
}

