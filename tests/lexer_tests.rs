use compiler::{common::symbol::Symbol, stelalo_ast::token::{LiteralKind, Token, TokenKind}, steralo_lexer::Lexer};

#[test]
fn test_complex_expression() {
    let input = r#"
fn main() {
    let x = 42.0;
    if x > 10 {
        print "Hello";
    }
    // line comment
    while true {}
}
"#.trim();

    let mut lexer = Lexer::new(input);

    let tokens = lexer.lex().unwrap();

    let expected_kinds = vec![
        TokenKind::Fn,
        TokenKind::Ident(Symbol::new(0)), // main
        TokenKind::LParen,
        TokenKind::RParen,
        TokenKind::LBrace,
        TokenKind::Let,
        TokenKind::Ident(Symbol::new(1)), // x
        TokenKind::Equal,
        TokenKind::Literal {
            kind: LiteralKind::Float,
            symbol: Symbol::new(2), // 42.0
        },
        TokenKind::Semicolon,
        TokenKind::If,
        TokenKind::Ident(Symbol::new(1)), // x
        TokenKind::Greater,
        TokenKind::Literal {
            kind: LiteralKind::Integer,
            symbol: Symbol::new(3), // 10
        },
        TokenKind::LBrace,
        TokenKind::Print,
        TokenKind::Literal {
            kind: LiteralKind::Str,
            symbol: Symbol::new(4) // "Hello"
        },
        TokenKind::Semicolon,
        TokenKind::RBrace,
        TokenKind::LineComment,
        TokenKind::While,
        TokenKind::Literal {
            kind: LiteralKind::Bool(true),
            symbol: Symbol::new(5) // true
        },
        TokenKind::LBrace,
        TokenKind::RBrace,
        TokenKind::RBrace,
        TokenKind::Eof,
    ];

    assert_eq!(
        tokens.map(|t| t.kind).collect::<Vec<_>>(),
        expected_kinds
    );

    assert_eq!("x", Symbol::new(1).as_str());
    assert_eq!("\"Hello\"", Symbol::new(4).as_str());
}


#[test]
fn test_token_pos() {
    let input = r#"
let str = "Hello, World!";
    print "";
fn f() {}
"#.trim();

    let mut lexer = Lexer::new(input);

    let tokens = lexer.lex().unwrap();

    let expected = vec![
        Token {
            kind: TokenKind::Let,
            line: 1,
            start: 0,
            end: 3,
        },
        Token {
            kind: TokenKind::Ident(
                Symbol::new(0),
            ),
            line: 1,
            start: 4,
            end: 7,
        },
        Token {
            kind: TokenKind::Equal,
            line: 1,
            start: 8,
            end: 9,
        },
        Token {
            kind: TokenKind::Literal {
                kind: LiteralKind::Str,
                symbol: Symbol::new(1),
            },
            line: 1,
            start: 10,
            end: 25,
        },
        Token {
            kind: TokenKind::Semicolon,
            line: 1,
            start: 25,
            end: 26,
        },
        Token {
            kind: TokenKind::Print,
            line: 2,
            start: 4,
            end: 9,
        },
        Token {
            kind: TokenKind::Literal {
                kind: LiteralKind::Str,
                symbol: Symbol::new(2),
            },
            line: 2,
            start: 10,
            end: 12,
        },
        Token {
            kind: TokenKind::Semicolon,
            line: 2,
            start: 12,
            end: 13,
        },
        Token {
            kind: TokenKind::Fn,
            line: 3,
            start: 0,
            end: 2,
        },
        Token {
            kind: TokenKind::Ident(
                Symbol::new(3),
            ),
            line: 3,
            start: 3,
            end: 4,
        },
        Token {
            kind: TokenKind::LParen,
            line: 3,
            start: 4,
            end: 5,
        },
        Token {
            kind: TokenKind::RParen,
            line: 3,
            start: 5,
            end: 6,
        },
        Token {
            kind: TokenKind::LBrace,
            line: 3,
            start: 7,
            end: 8,
        },
        Token {
            kind: TokenKind::RBrace,
            line: 3,
            start: 8,
            end: 9,
        },
        Token {
            kind: TokenKind::Eof,
            line: 3,
            start: 9,
            end: 9,
        },
    ];

    assert_eq!(
        tokens.collect::<Vec<Token>>(), expected
    );
}
