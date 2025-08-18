pub mod arena;
pub mod def_id;
pub mod fatal_error;
pub mod fingerprint;
pub mod hashes;
pub mod idx;
pub mod index_vec;
pub mod lit_utils;
pub mod map;
pub mod slice;
pub mod sorted_map;
pub mod source_map;
pub mod span;
pub mod stable_hasher;
pub mod stack;
pub mod symbol;
pub mod unhash;
pub mod visit_utils;
mod diagnostics;

#[cfg(test)]
mod tests;

pub use arena::{Arena, TypedArena};
pub use def_id::{
    DefId, DefIndex, DefPathHash, LOCAL_STELO, LocalDefId, STELO_DEF_ID, STELO_ROOT_INDEX,
    StableSteloId, SteloNum,
};
pub use fatal_error::FatalError;
pub use fingerprint::{Fingerprint, FingerprintComponent};
pub use hashes::{Hash64, Hash128};
pub use idx::{Idx, IntoSliceIdx};
pub use index_vec::IndexVec;
pub use map::IndexMap;
pub use slice::IndexSlice;
pub use sorted_map::SortedMap;
pub use source_map::{FileLoader, RealFileLoader, SourceMap, SourceMapInputs};
pub use span::{DUMMY_SPAN, Span, Spanned};
// impl_hash_stable_trivial は stelaro_common 外部に公開されるべきではない
pub use stable_hasher::{FromStableHash, StableHasher, StableHasherHash};
pub use stack::ensure_sufficient_stack;
pub use symbol::{Ident, Symbol, sym};
pub use visit_utils::VisitorResult;

use std::{fmt, rc::Rc};


scoped_tls::scoped_thread_local!(static SESSION_GLOBALS: SessionGlobals);

pub struct SessionGlobals {
    symbol_interner: symbol::Interner,
    source_map: Option<Rc<SourceMap>>,
}

impl SessionGlobals {
    pub fn new(
        sm_inputs: Option<SourceMapInputs>,
    ) -> SessionGlobals {
        SessionGlobals {
            symbol_interner: symbol::Interner::new(),
            source_map: sm_inputs.map(|inputs| Rc::new(SourceMap::with_inputs(inputs))),
        }
    }
}


pub fn create_session_globals_then<R>(
    sm_inputs: Option<SourceMapInputs>,
    f: impl FnOnce() -> R,
) -> R {
    assert!(
        !SESSION_GLOBALS.is_set(),
        "SESSION_GLOBALS は上書きされるべきではない"
    );
    let session_globals = SessionGlobals::new(sm_inputs);
    SESSION_GLOBALS.set(&session_globals, f)
}


pub fn with_session_globals<R, F>(f: F) -> R
where
    F: FnOnce(&SessionGlobals) -> R,
{
    SESSION_GLOBALS.with(f)
}

/// SourceMap はなし
pub fn create_default_session_globals_then<R>(f: impl FnOnce() -> R) -> R {
    create_session_globals_then(None, f)
}


/// `resume_unwind` で使われる致命的なコンパイルエラーのためのマーカー
pub struct FatalErrorMarker;

/// `&mut Formatter` を引数に取るクロージャを、Displayフォーマット可能なオブジェクトに変換する
pub fn make_display(f: impl Fn(&mut fmt::Formatter<'_>) -> fmt::Result) -> impl fmt::Display {
    struct Printer<F> {
        f: F,
    }
    impl<F> fmt::Display for Printer<F>
    where
        F: Fn(&mut fmt::Formatter<'_>) -> fmt::Result,
    {
        fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
            (self.f)(fmt)
        }
    }

    Printer { f }
}
