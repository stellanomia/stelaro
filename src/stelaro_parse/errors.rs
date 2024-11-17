use crate::stelaro_common::error_core::ErrorCore;

#[derive(Debug)]
pub struct ParseError(Box<ErrorCore>);

impl From<ParseError> for ErrorCore {
    fn from(value: ParseError) -> Self {
        *value.0
    }
}

impl ParseError {
    fn new(core: ErrorCore) -> Self {
        ParseError(Box::new(core))
    }
}