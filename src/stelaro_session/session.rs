use std::path::PathBuf;
use std::rc::Rc;

use crate::stelaro_common::source_map::{get_source_map, SourceMap};
use crate::stelaro_diagnostic::emitter::{AriadneEmitter, DynEmitter};
use crate::stelaro_diagnostic::{diag::DiagCtxtHandle, DiagCtxt};

use super::config::{Input, OutFileName};
use super::parse::ParseSess;

pub struct CompilerPaths {
    pub input: Input,
    pub output_dir: Option<PathBuf>,
    pub output_file: Option<OutFileName>,
    pub temps_dir: Option<PathBuf>,
}

pub struct Session {
    pub psess: ParseSess,
    pub paths: CompilerPaths,
}

impl Session {

    #[inline]
    pub fn dcx(&self) -> DiagCtxtHandle<'_> {
        self.psess.dcx()
    }

    #[inline]
    pub fn source_map(&self) -> &SourceMap {
        self.psess.source_map()
    }
}

pub fn default_emitter(
    source_map: Rc<SourceMap>,
) -> Box<DynEmitter> {
    Box::new(AriadneEmitter::new(source_map))
}

pub fn build_session(
    paths: CompilerPaths,
) -> Session {
    let source_map = get_source_map().unwrap();
    let emitter = default_emitter(Rc::clone(&source_map));
    let dcx = DiagCtxt::new(emitter);

    let psess = ParseSess::with_dcx(dcx, source_map);

    Session {
        psess,
        paths:
            CompilerPaths {
                input: paths.input,
                output_dir: paths.output_dir,
                output_file: paths.output_file,
                temps_dir: paths.temps_dir,
            },
    }
}
