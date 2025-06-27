use std::rc::Rc;

use stelaro::stelaro_ast::token::{Lit, LitKind, TokenKind};
use stelaro::stelaro_common::{SourceMap, Symbol, create_default_session_globals_then};
use stelaro::stelaro_session::ParseSess;
use stelaro::stelaro_lexer::Lexer;
use stelaro::stelaro_diagnostics::{DiagCtxt, SilentEmitter};

fn create_test_context() -> ParseSess {
    let source_map = Rc::new(SourceMap::new());
    let emitter = SilentEmitter::new();
    let dcx = DiagCtxt::new(Box::new(emitter));
    ParseSess::with_dcx(dcx, source_map)
}

#[test]
fn test_complex_syntax() {
    let src = r#"
fn main() {
    let x = 42.0;
    if x > 10 {
        "Hello";
    }
    // line comment
    while true {}
}
"#.trim();

    let psess = create_test_context();

    create_default_session_globals_then(|| {
        let mut lexer = Lexer::new(&psess, src);
        let tokens = lexer.lex().unwrap();
        let expected_kinds = vec![
            TokenKind::Fn,
            TokenKind::Ident(Symbol::intern("main")),
            TokenKind::LParen,
            TokenKind::RParen,
            TokenKind::LBrace,
            TokenKind::Let,
            TokenKind::Ident(Symbol::intern("x")),
            TokenKind::Equal,
            TokenKind::Literal (
                Lit {
                    kind: LitKind::Float,
                    symbol: Symbol::intern("42.0"),
                }
            ),
            TokenKind::Semicolon,
            TokenKind::If,
            TokenKind::Ident(Symbol::intern("x")),
            TokenKind::Greater,
            TokenKind::Literal (
                Lit {
                    kind: LitKind::Integer,
                    symbol: Symbol::intern("10"),
                }
            ),
            TokenKind::LBrace,
            TokenKind::Literal (
                Lit {
                    kind: LitKind::Str,
                    symbol: Symbol::intern("\"Hello\""),
                }
            ),
            TokenKind::Semicolon,
            TokenKind::RBrace,
            TokenKind::While,
            TokenKind::Literal (
                Lit {
                    kind: LitKind::Bool(true),
                    symbol: Symbol::intern("true")
                }
            ),
            TokenKind::LBrace,
            TokenKind::RBrace,
            TokenKind::RBrace,
            TokenKind::Eof,
        ];

        assert_eq!(
            tokens.map(|t| t.kind).collect::<Vec<_>>(),
            expected_kinds
        );
    });
}
