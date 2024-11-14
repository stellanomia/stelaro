use crate::common::error::ErrorCore;

#[derive(Debug)]
pub struct LexerError(Box<ErrorCore>);

impl From<LexerError> for ErrorCore {
    fn from(value: LexerError) -> Self {
        *value.0
    }
}

impl LexerError {
    fn new(core: ErrorCore) -> Self {
        LexerError(Box::new(core))
    }

    pub fn unexpected_character(line: u32, start: u32, end: u32, found: char) -> Self {
        LexerError::new(
            ErrorCore {
                msg: format!("Unexpected character '{}'.", found),
                line,
                start,
                end,
            }
        )
    }

    pub fn invalid_float_format(line: u32, start: u32, end: u32) -> Self {
        LexerError::new(
            ErrorCore {
                msg: "The format of the floating-point number is invalid.".to_string(),
                line,
                start,
                end,
            }
        )
    }

    pub fn missing_fractional_part(line: u32, start: u32, end: u32) -> Self {
        LexerError::new(
            ErrorCore {
                msg: "Missing fractional part in numeric literal. Digits must follow the decimal point.".to_string(),
                line,
                start,
                end,
            }
        )
    }

    pub fn invalid_escape_sequence(line: u32, start: u32, end: u32) -> Self {
        LexerError::new(
            ErrorCore {
                msg: "Invalid escape sequence. Supported escape sequences are: \\n, \\r, \\t, \\0, \\\", \\\\".to_string(),
                line,
                start,
                end,
            }
        )
    }

    pub fn unterminated_string_literal(line: u32, start: u32, end: u32) -> Self {
        LexerError::new(
            ErrorCore {
                msg: "Unterminated string literal. String literals must be closed with a double quote (\"). Consider checking for a missing closing quote or an unescaped newline.".to_string(),
                line,
                start,
                end,
            }
        )
    }

    pub fn unconsumed_input(line: u32, col: u32, remaining: &str) -> Self {
        LexerError::new(
            ErrorCore {
                msg: format!("Unconsumed input {}.", remaining),
                line,
                start: col,
                end: col + remaining.len() as u32,
            }
        )
    }
}