pub mod diag;
pub mod emitter;

pub use diag::{Diag, DiagCtxtHandle, ErrorEmitted};
pub use emitter::{AriadneEmitter, SilentEmitter};

use diag::DiagCtxtInner;
use emitter::DynEmitter;
use std::cell::RefCell;

pub struct DiagCtxt {
    inner: RefCell<DiagCtxtInner>,
}

impl DiagCtxt {
    pub fn new(emitter: Box<DynEmitter>) -> DiagCtxt {
        DiagCtxt {
            inner: RefCell::new(DiagCtxtInner::new(emitter)),
        }
    }

    pub fn handle(&self) -> DiagCtxtHandle<'_> {
        DiagCtxtHandle { dcx: self }
    }
}
