use core::fmt;
use std::error::Error;

#[derive(Debug)]
pub struct ErrorCore {
    pub msg: String,
    pub line: u32,
    pub start: u32,
    pub end: u32,
}

impl ErrorCore {}

impl fmt::Display for ErrorCore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

impl Error for ErrorCore {}