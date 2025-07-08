use crate::stelaro_ast::token::{Lit, LitKind, Token, TokenKind, TokenStream};
use crate::stelaro_common::Symbol;
use crate::stelaro_diagnostics::ErrorEmitted;
use crate::stelaro_session::ParseSess;

use super::cursor::{Cursor, EOF_CHAR};
use super::diagnostics::DiagsLexer;

pub struct Lexer<'src, 'sess> {
    src: &'src str,
    cursor: Cursor<'src>,
    pos: usize,
    psess: &'sess ParseSess,
}

impl<'src, 'sess> Lexer<'src, 'sess> {
    pub fn new(
        psess: &'sess ParseSess,
        src: &'src str,
    ) -> Self {
        Self {
            src,
            cursor: Cursor::new(src),
            pos: 0,
            psess,
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
        self.skip_whitespace_and_comment();

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
            '/' => {
                self.bump();

                // コメントは skip_whitespace_and_comment で捨てられる
                TokenKind::Slash
            },
            '%' => {
                self.bump();
                TokenKind::Percent
            }
            ';' => {
                self.bump();
                TokenKind::Semicolon
            },
            ':' => {
                self.bump();
                if self.first() == ':' {
                    self.bump();
                    TokenKind::PathSep
                } else {
                    TokenKind::Colon
                }
            },
            '!' => {
                self.bump();

                if self.first() == '=' {
                    self.bump();
                    TokenKind::BangEqual
                } else {
                    TokenKind::Bang
                }
            },
            '=' => {
                self.bump();

                if self.first() == '=' {
                    self.bump();
                    TokenKind::EqualEqual
                } else {
                    TokenKind::Equal
                }
            },
            '>' => {
                self.bump();

                if self.first() == '=' {
                    self.bump();
                    TokenKind::GreaterEqual
                } else {
                    TokenKind::Greater
                }
            },
            '<' => {
                self.bump();

                if self.first() == '=' {
                    self.bump();
                    TokenKind::LessEqual
                } else {
                    TokenKind::Less
                }
            },
            ('0'..='9') => {
                // LitKind::Integer, Floatのどちらかをとりうる
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
                        kind: LitKind::Str,
                        symbol: Symbol::intern(&self.src[pos..self.pos])
                    }
                )
            },
            '\'' => {
                self.bump();
                let symbol = self.lex_char_lit(pos)?;
                TokenKind::Literal(
                    Lit {
                        kind: LitKind::Char,
                        symbol
                    }
                )
            }
            c if c.is_alphabetic() || c == '_' => {
                self.bump();
                // キーワード、Identifier、boolean値を解析する
                self.lex_word(pos)?
            },
            EOF_CHAR => {
                TokenKind::Eof
            }
            c => {
                self.bump();

                Err(
                    DiagsLexer::unexpected_character(
                        self.psess.dcx(),
                        c,
                        (pos..pos+1).into()
                    ).emit()
                )?
            },
        };

        Ok(
            Token {
                kind: token_kind,
                span: (pos, self.pos).into()
            }
        )
    }

    fn first(&self) -> char {
        self.cursor.first()
    }

    fn second(&self) -> char {
        self.cursor.second()
    }

    fn prev(&self) -> char {
        self.cursor.prev
    }

    fn bump(&mut self) -> Option<char> {
        self.pos += 1;
        self.cursor.bump()
    }

    fn skip_whitespace_and_comment(&mut self) {
        loop {
            match self.first() {
                '/' if self.second() == '/' => {
                    while !matches!(self.first(), '\n' | EOF_CHAR) {
                        self.bump();
                    }
                },
                c if c.is_whitespace() => {
                    self.bump();
                }
                _ => {
                    break
                }
            }
        }
    }

    fn lex_number(&mut self, pos: usize) -> Result<LitKind, ErrorEmitted> {
        if let '0'..='9' = self.first() {
            self.bump();

            let mut is_float = false;

            while let c @ ('0'..='9') | c @ '.' = self.first() {
                match c {
                    '0'..='9' => {
                        self.bump();
                    },
                    '.' => {
                        if is_float {
                            Err(
                                DiagsLexer::invalid_float_format(
                                    self.psess.dcx(),
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
                        self.psess.dcx(),
                        (pos..self.pos).into(),
                    ).emit()
                )?
            }

            if is_float {
                Ok(LitKind::Float)
            } else {
                Ok(LitKind::Integer)
            }
        } else {
            // 最初に'.'が入力になることはない
            unreachable!()
        }
    }

    fn lex_escape_sequence(&mut self) -> Result<(), ErrorEmitted> {
        if '\\' == self.first() {
            self.bump();
            match self.first() {
                'n' | 'r' | 't' | '0' | '\'' | '"' | '\\' => {
                    self.bump();
                    Ok(())
                },
                _ => {
                    self.bump();
                    Err(
                        DiagsLexer::invalid_escape_sequence(
                            self.psess.dcx(),
                            self.prev(),
                            (self.pos-1..self.pos).into()
                        ).emit()
                    )?
                }
            }
        } else {
            unreachable!()
        }
    }

    fn lex_str_lit(&mut self, pos: usize) -> Result<(), ErrorEmitted> {
        loop {
            match self.first() {
                '\\' => {
                    self.lex_escape_sequence()?;
                },
                '"' => {
                    self.bump();
                    break Ok(());
                },
                '\n' | EOF_CHAR => {
                    self.bump();

                    // 通常の文字列リテラル中に改行が見つかった場合はエラー
                    Err(
                        DiagsLexer::unterminated_string_literal(
                            self.psess.dcx(),
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

    fn lex_char_lit(&mut self, pos: usize) -> Result<Symbol, ErrorEmitted> {
        let symbol = match self.first() {
            '\\' => {
                self.lex_escape_sequence()?;
                Symbol::intern(&self.src[pos..self.pos])
            },
            '\n' => {
                self.bump();

                Err(
                    DiagsLexer::unexpected_quote(
                        self.psess.dcx(),
                        (pos..self.pos-1).into()
                    ).emit()
                )?
            },
            _ => {
                self.bump();
                Symbol::intern(&self.src[pos..self.pos])
            }
        };

        if self.first() != '\'' {
            // 内部的にはただのイテレータで、コストは低い
            let mut cursor = self.cursor.clone();
            let mut next = cursor.first();
            let mut quote_not_found = false;
            let mut end = self.pos;

            while next != '\'' {
                if next == '\n' || next == EOF_CHAR {
                    quote_not_found = true;
                    break;
                }
                end += 1;

                cursor.bump();
                next = cursor.first();
            }

            if quote_not_found {
                Err(
                    DiagsLexer::unterminated_char_literal(
                        self.psess.dcx(),
                        (end-1..end).into()
                    ).emit()
                )?
            } else {
                Err(
                    DiagsLexer::multiple_characters_in_char_literal(
                        self.psess.dcx(),
                        (pos..end+1).into()
                    ).emit()
                )?
            }
        } else {
            self.bump();
        }

        Ok(symbol)
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
                                kind: LitKind::Bool(true),
                                symbol: Symbol::intern(&self.src[pos..self.pos]),
                            }
                        )
                    } else if keyword_or_ident == "false" {
                        TokenKind::Literal (
                            Lit {
                                kind: LitKind::Bool(false),
                                symbol: Symbol::intern(&self.src[pos..self.pos]),
                            }
                        )
                    } else {
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
            "fn" => Some(TokenKind::Fn),
            "mod" => Some(TokenKind::Mod),
            "break" => Some(TokenKind::Break),
            "return" => Some(TokenKind::Return),
            "let" => Some(TokenKind::Let),
            "if" => Some(TokenKind::If),
            "else" => Some(TokenKind::Else),
            "and" => Some(TokenKind::And),
            "or" => Some(TokenKind::Or),
            "for" => Some(TokenKind::For),
            "loop" => Some(TokenKind::Loop),
            "while" => Some(TokenKind::While),
            _ => None,
        }
    }
}
