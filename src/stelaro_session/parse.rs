use std::rc::Rc;

use crate::stelaro_common::SourceMap;
use crate::stelaro_diagnostic::{DiagCtxt, DiagCtxtHandle};


pub struct ParseSess {
    dcx: DiagCtxt,
    source_map: Rc<SourceMap>,
}

impl ParseSess {
    pub fn with_dcx(dcx: DiagCtxt, source_map: Rc<SourceMap>) -> Self {
        Self {
            dcx,
            source_map,
        }
    }

    #[inline]
    pub fn dcx(&self) -> DiagCtxtHandle<'_> {
        self.dcx.handle()
    }

    #[inline]
    pub fn source_map(&self) -> &SourceMap {
        &self.source_map
    }
}