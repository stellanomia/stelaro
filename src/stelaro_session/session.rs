use std::rc::Rc;

use crate::{stelaro_common::source_map::SourceMap, stelaro_diagnostic::DiagCtxt};


pub struct Session {
    source_map: Rc<SourceMap>,
    dcx: DiagCtxt,
}

impl Session {
    pub fn new(dcx: DiagCtxt, source_map: Rc<SourceMap>) -> Self {
        Self { source_map, dcx }
    }

    pub fn source_map(&self) -> &SourceMap {
        &self.source_map
    }

    pub fn source_map_clone(&self) -> Rc<SourceMap> {
        Rc::clone(&self.source_map)
    }
}