use std::rc::Rc;

use crate::stelaro_common::source_map::SourceMap;


pub struct Session {
    source_map: Rc<SourceMap>,
}

impl Session {
    pub fn new() -> Self {
        let source_map = Rc::new(SourceMap::new());

        Self { source_map }
    }

    pub fn source_map(&self) -> &SourceMap {
        &self.source_map
    }

    pub fn source_map_clone(&self) -> Rc<SourceMap> {
        Rc::clone(&self.source_map)
    }
}