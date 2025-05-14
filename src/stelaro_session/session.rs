use std::path::PathBuf;
use std::rc::Rc;

use crate::stelaro_common::source_map::{get_source_map, SourceMap};
use crate::stelaro_diagnostic::emitter::DynEmitter;
use crate::stelaro_diagnostic::{diag::DiagCtxtHandle, DiagCtxt};

use super::config::{Input, OutFileName};

pub struct CompilerPaths {
    pub input: Input,
    pub output_dir: Option<PathBuf>,
    pub output_file: Option<OutFileName>,
    pub temps_dir: Option<PathBuf>,
}

pub struct Session {
    dcx: DiagCtxt,
    pub io: CompilerPaths,
    pub source_map: Rc<SourceMap>,
}

impl Session {
    pub fn new(dcx: DiagCtxt, source_map: Rc<SourceMap>) -> Self {
        unimplemented!()
    }

    pub fn source_map(&self) -> &SourceMap {
        &self.source_map
    }

    pub fn source_map_clone(&self) -> Rc<SourceMap> {
        Rc::clone(&self.source_map)
    }

    pub fn dcx(&self) -> DiagCtxtHandle<'_> {
        self.dcx.handle()
    }
}

pub fn default_emitter(
    source_map: Rc<SourceMap>,
) -> Box<DynEmitter> {
    todo!()
}

pub fn build_session(
    paths: CompilerPaths,
) -> Session {
    let source_map = get_source_map().unwrap();

    let dcx = DiagCtxt::new(todo!());
}
