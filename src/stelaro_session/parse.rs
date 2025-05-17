use std::rc::Rc;

use crate::stelaro_common::SourceMap;
use crate::stelaro_diagnostic::DiagCtxt;


pub struct ParseSess {
    dcx: DiagCtxt,
    source_map: Rc<SourceMap>,
}