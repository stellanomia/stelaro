use std::rc::Rc;

use stelaro::stelaro_session::Session;
use stelaro::stelaro_lexer::Lexer;
use stelaro::stelaro_diagnostic::DiagCtxt;
use stelaro::stelaro_common::{source_map::SourceMap, span::Span, symbol::Symbol};
use stelaro::stelaro_ast::token::{Lit, LiteralKind, Token, TokenKind};

fn create_sess(src: String) -> Session {
    let src = Rc::new(src);
    let dcx = DiagCtxt::new(Rc::clone(&src));
    let source_map = Rc::new(SourceMap::new());
    Session::new(dcx, source_map)
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

    let sess = &create_sess(src.to_string());

    let mut lexer = Lexer::new(sess, src);

    let tokens = lexer.lex().unwrap();

    let expected_kinds = vec![
        TokenKind::Fn,
        TokenKind::Ident(Symbol::new(1)), // main
        TokenKind::LParen,
        TokenKind::RParen,
        TokenKind::LBrace,
        TokenKind::Let,
        TokenKind::Ident(Symbol::new(2)), // x
        TokenKind::Equal,
        TokenKind::Literal (
            Lit {
                kind: LiteralKind::Float,
                symbol: Symbol::new(3), // 42.0
            }
        ),
        TokenKind::Semicolon,
        TokenKind::If,
        TokenKind::Ident(Symbol::new(2)), // x
        TokenKind::Greater,
        TokenKind::Literal (
            Lit {
                kind: LiteralKind::Integer,
                symbol: Symbol::new(4), // 10
            }
        ),
        TokenKind::LBrace,
        TokenKind::Literal (
            Lit {
                kind: LiteralKind::Str,
                symbol: Symbol::new(5) // "Hello"
            }
        ),
        TokenKind::Semicolon,
        TokenKind::RBrace,
        TokenKind::LineComment,
        TokenKind::While,
        TokenKind::Literal (
            Lit {
                kind: LiteralKind::Bool(true),
                symbol: Symbol::new(6) // true
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

    assert_eq!("x", Symbol::new(2).as_str());
    assert_eq!("\"Hello\"", Symbol::new(5).as_str());
}


#[test]
fn test_token_pos() {
    let src = r#"
let str = "Hello, World!";
          "";
"#.trim().to_string();

    let sess = &create_sess(src.to_string());
    let mut lexer = Lexer::new(sess, &src);


    let tokens = lexer.lex().unwrap();
    let expected = vec![
        Token {
            kind: TokenKind::Let,
            span: Span {
                start: 0,
                end: 3,
            },
        },
        Token {
            kind: TokenKind::Ident(
                Symbol::new(1),
            ),
            span: Span {
                start: 4,
                end: 7,
            },
        },
        Token {
            kind: TokenKind::Equal,
            span: Span {
                start: 8,
                end: 9,
            },
        },
        Token {
            kind: TokenKind::Literal (
                Lit {
                    kind: LiteralKind::Str,
                    symbol: Symbol::new(2),
                }
            ),
            span: Span {
                start: 10,
                end: 25,
            },
        },
        Token {
            kind: TokenKind::Semicolon,
            span: Span {
                start: 25,
                end: 26,
            },
        },
        Token {
            kind: TokenKind::Literal (
                Lit {
                    kind: LiteralKind::Str,
                    symbol: Symbol::new(3),
                }
            ),
            span: Span {
                start: 37,
                end: 39,
            },
        },
        Token {
            kind: TokenKind::Semicolon,
            span: Span {
                start: 39,
                end: 40,
            },
        },
        Token {
            kind: TokenKind::Eof,
            span: Span {
                start: 40,
                end: 40,
            },
        },
    ];

    assert_eq!(
        tokens.collect::<Vec<Token>>(), expected
    );
}
