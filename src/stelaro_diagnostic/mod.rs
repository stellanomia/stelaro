pub mod diag;

use std::{cell::RefCell, rc::Rc};
use diag::{DiagCtxtHandle, DiagCtxtInner};


#[derive(Debug)]
pub struct DiagCtxt {
    inner: RefCell<DiagCtxtInner>,
}

impl DiagCtxt {
    pub fn new(src: Rc<String>) -> DiagCtxt {
        DiagCtxt {
            inner: RefCell::new(
                DiagCtxtInner::new(src)
            ),
        }
    }

    pub fn handle(&self) -> DiagCtxtHandle<'_> {
        DiagCtxtHandle {
            dcx: self,
        }
    }
}
