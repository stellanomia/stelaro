use crate::stelaro_ast::token::{Lit, LiteralKind, Token, TokenKind, TokenStream};
use crate::stelaro_common::{span::Span, Symbol};
use crate::stelaro_diagnostic::diag::ErrorEmitted;
use crate::stelaro_session::Session;

use super::cursor::{Cursor, EOF_CHAR};
use super::diagnostics::DiagsLexer;

pub struct Lexer<'src , 'sess> {
    src: &'src str,
    cursor: Cursor<'src>,
    pos: usize,
    sess: &'sess Session,
}

impl<'src, 'sess> Lexer<'src, 'sess> {
    pub fn new(
        src: &'src str,
        sess: &'sess Session
    ) -> Self {
        Self {
            src,
            cursor: Cursor::new(src),
            pos: 0,
            sess,
        }
    }

    pub fn lex(&mut self) -> Result<TokenStream, ErrorEmitted> {
        let mut ts = TokenStream::empty();

        loop {
            match self.next_token() {
                Ok(token) => {
                    ts.push(token);

                    if token.kind == TokenKind::Eof {
                        break;
                    }
                }
                Err(e) => {
                    return Err(e);
                }
            }
        }

        Ok(ts)
    }

    fn next_token(&mut self) -> Result<Token, ErrorEmitted> {
        self.skip_whitespace();

        // 読み始めるトークンの最初の位置を保持する
        let pos = self.pos;

        let token_kind = match self.first() {
            '(' => {
                self.bump();
                TokenKind::LParen
            },
            ')' => {
                self.bump();
                TokenKind::RParen
            },
            '{' => {
                self.bump();
                TokenKind::LBrace
            },
            '}' => {
                self.bump();
                TokenKind::RBrace
            },
            ',' => {
                self.bump();
                TokenKind::Comma
            },
            '.' => {
                self.bump();
                TokenKind::Dot
            },
            '+' => {
                self.bump();
                TokenKind::Plus
            },
            '-' => {
                self.bump();
                TokenKind::Minus
            },
            '*' => {
                self.bump();
                TokenKind::Star
            },
            '%' => {
                self.bump();
                TokenKind::Percent
            }
            ';' => {
                self.bump();
                TokenKind::Semicolon
            },
            '/' => {
                self.bump();

                if self.first() == '/' {
                    self.bump();
                    while self.first() != '\n' {
                        self.bump();
                    }
                    TokenKind::LineComment
                }else {
                    TokenKind::Slash
                }
            },
            '!' => {
                self.bump();

                if self.first() == '=' {
                    self.bump();
                    TokenKind::BangEqual
                }else {
                    TokenKind::Bang
                }
            },
            '=' => {
                self.bump();

                if self.first() == '=' {
                    self.bump();
                    TokenKind::EqualEqual
                }else {
                    TokenKind::Equal
                }
            },
            '>' => {
                self.bump();

                if self.first() == '=' {
                    self.bump();
                    TokenKind::GreaterEqual
                }else {
                    TokenKind::Greater
                }
            },
            '<' => {
                self.bump();

                if self.first() == '=' {
                    self.bump();
                    TokenKind::LessEqual
                }else {
                    TokenKind::Less
                }
            },
            ('0'..='9') => {
                // LiteralKind::Integer, Floatのどちらかをとりうる
                let lit_kind = self.lex_number(pos)?;
                TokenKind::Literal (
                    Lit {
                        kind: lit_kind,
                        symbol: Symbol::intern(&self.src[pos..self.pos])
                    }
                )
            },
            '"' => {
                self.bump();

                // 文字列リテラルの終端まで位置を進める
                self.lex_str_lit(pos)?;

                TokenKind::Literal (
                    Lit {
                        kind: LiteralKind::Str,
                        symbol: Symbol::intern(&self.src[pos..self.pos])
                    }
                )
            },
            c if c.is_alphabetic() => {
                self.bump();
                // キーワード、Identifier、boolean値を解析する
                self.lex_word(pos)?
            },
            EOF_CHAR => {
                self.bump();

                TokenKind::Eof
            }
            c => {
                self.bump();

                Err(
                    DiagsLexer::unexpected_character(
                        self.sess.dcx(),
                        c,
                        (pos..pos+1).into()
                    ).emit()
                )?
            },
        };

        Ok(
            Token {
                kind: token_kind,
                span: Span {
                    start: pos,
                    end: self.pos,
                }
            }
        )
    }

    fn first(&self) -> char {
        self.cursor.first()
    }

    fn prev(&self) -> char {
        self.cursor.prev
    }

    fn bump(&mut self) -> Option<char> {
        self.pos += 1;
        self.cursor.bump()
    }

    fn skip_whitespace(&mut self) {
        while self.first().is_whitespace() {
            self.bump();
        }
    }

    fn lex_number(&mut self, pos: usize) -> Result<LiteralKind, ErrorEmitted> {
        if let '0'..='9' = self.first() {
            self.bump();

            let mut is_float = false;

            while let c @ ('0'..='9') | c@ '.' = self.first() {
                match c {
                    '0'..='9' => {
                        self.bump();
                    },
                    '.' => {
                        if is_float {
                            Err(
                                DiagsLexer::invalid_float_format(
                                    self.sess.dcx(),
                                    (pos..self.pos).into()
                                ).emit()
                            )?
                        }

                        is_float = true;
                        self.bump();
                    },
                    _ => unreachable!()
                }
            }

            //最後の入力が'.'である(e.g. "123.")
            if self.prev() == '.' {
                Err(
                    DiagsLexer::missing_fractional_part(
                        self.sess.dcx(),
                        (pos..self.pos).into(),
                    ).emit()
                )?
            }

            if is_float {
                Ok(LiteralKind::Float)
            } else {
                Ok(LiteralKind::Integer)
            }
        } else {
            // 最初に'.'が入力になることはない
            unreachable!()
        }
    }

    fn lex_str_lit(&mut self, pos: usize) -> Result<(), ErrorEmitted> {
        loop {
            match self.first() {
                '\\' => {
                    self.bump();
                    match self.first() {
                        'n' | 'r' | 't' | '0' | '\'' | '"' | '\\' => {
                            self.bump();
                        },
                        _ => {
                            self.bump();
                            Err(
                                DiagsLexer::invalid_escape_sequence(
                                    self.sess.dcx(),
                                    self.first(),
                                    (pos..self.pos).into()
                                ).emit()
                            )?
                        }

                    }
                },
                '"' => {
                    self.bump();
                    break Ok(());
                },
                '\n' => {
                    self.bump();

                    // 通常の文字列リテラル中に改行が見つかった場合はエラー
                    Err(
                        DiagsLexer::unterminated_string_literal(
                            self.sess.dcx(),
                            (pos..self.pos-1).into()
                        ).emit()
                    )?
                }
                _ => {
                    self.bump();
                }
            }
        }
    }

    fn lex_word(&mut self, pos: usize) -> Result<TokenKind, ErrorEmitted> {
        // アンダースコア、数字がここに来ることはない
        while matches!(self.first(), c if c.is_alphabetic() || c == '_' || c.is_numeric()) {
            self.bump();
        }

        let keyword_or_ident = &self.src[pos..self.pos];

        Ok(
            match self.as_keyword(keyword_or_ident) {
                Some(keyword) => keyword,
                None => {
                    if keyword_or_ident == "true" {
                        TokenKind::Literal (
                            Lit {
                                kind: LiteralKind::Bool(true),
                                symbol: Symbol::intern(&self.src[pos..self.pos]),
                            }
                        )
                    } else if keyword_or_ident == "false" {
                        TokenKind::Literal (
                            Lit {
                                kind: LiteralKind::Bool(false),
                                symbol: Symbol::intern(&self.src[pos..self.pos]),
                            }
                        )
                    }else {
                        TokenKind::Ident(
                            Symbol::intern(keyword_or_ident)
                        )
                    }
                },
            }
        )
    }

    fn as_keyword(&self, string: &str) -> Option<TokenKind> {
        match string {
            "null" => Some(TokenKind::Null),
            "fn" => Some(TokenKind::Fn),
            "return" => Some(TokenKind::Return),
            "let" => Some(TokenKind::Let),
            "if" => Some(TokenKind::If),
            "else" => Some(TokenKind::Else),
            "and" => Some(TokenKind::And),
            "or" => Some(TokenKind::Or),
            "for" => Some(TokenKind::For),
            "print" => Some(TokenKind::Print),
            "while" => Some(TokenKind::While),
            _ => None,
        }
    }
}
