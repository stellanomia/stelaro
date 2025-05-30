pub mod arena;
pub mod def_id;
pub mod fingerprint;
pub mod hashes;
pub mod idx;
pub mod index_vec;
pub mod map;
pub mod slice;
pub mod source_map;
pub mod span;
pub mod stable_hasher;
pub mod symbol;
pub mod unhash;

#[cfg(test)]
mod tests;

pub use arena::{Arena, TypedArena};
pub use def_id::{DefId, DefPathHash, StableSteloId, LocalDefId, DefIndex, SteloNum, LOCAL_STELO, STELO_DEF_ID, STELO_ROOT_INDEX};
pub use fingerprint::{Fingerprint, FingerprintComponent};
pub use hashes::{Hash64, Hash128};
pub use idx::{Idx, IntoSliceIdx};
pub use index_vec::IndexVec;
pub use map::IndexMap;
pub use slice::IndexSlice;
pub use source_map::{SourceMap, SourceMapInputs, RealFileLoader};
pub use span::{Span, DUMMY_SPAN};
// impl_hash_stable_trivial は stelaro_common 外部に公開されるべきではない
pub use stable_hasher::{StableHasher, StableHasherHash, FromStableHash};
pub use symbol::{Symbol, Ident};

use std::rc::Rc;


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

