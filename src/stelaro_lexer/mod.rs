pub mod cursor;
pub mod lexer;
mod errors;

pub use lexer::Lexer;

#[cfg(test)]
mod tests {
    use errors::LexerError;

    use super::*;

    #[test]
    fn test_unterminated_string_literal() {
        let src = r#"
let num = "123;
print
"#.trim();
        let mut lexer = Lexer::new(src);
        assert_eq!(
            lexer.lex().unwrap_err().pop().unwrap(),
            LexerError::unterminated_string_literal(1, 10, 15)
        )
    }
}