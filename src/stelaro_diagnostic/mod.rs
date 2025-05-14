pub mod diag;
pub mod emitter;

use std::cell::RefCell;
use diag::{DiagCtxtHandle, DiagCtxtInner};
use emitter::DynEmitter;


pub struct DiagCtxt {
    inner: RefCell<DiagCtxtInner>,
}

impl DiagCtxt {
    pub fn new(emitter: Box<DynEmitter>) -> DiagCtxt {
        DiagCtxt {
            inner: RefCell::new(
                DiagCtxtInner::new(emitter)
            ),
        }
    }

    pub fn handle(&self) -> DiagCtxtHandle<'_> {
        DiagCtxtHandle {
            dcx: self,
        }
    }
}
