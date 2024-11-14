
use crate::{common::Symbol, stelalo_ast::token::{LiteralKind, Token, TokenKind, TokenStream}, steralo_lexer::cursor::Cursor};
use super::errors::LexerError;


pub struct Lexer<'src> {
    src: &'src str,
    cursor: Cursor<'src>,
    pos: u32,
    line: u32,
    col: u32,
    is_terminated: bool,
}

impl<'src> Lexer<'src> {
    pub fn new(
        src: &'src str,
    ) -> Self {
        Self {
            src,
            cursor: Cursor::new(src),
            pos: 0,
            line: 1,
            col: 0,
            is_terminated: false,
        }
    }

    pub fn lex(&mut self) -> Result<TokenStream, Vec<LexerError>> {
        let mut ts = TokenStream::empty();
        let mut errors: Vec<LexerError> = Vec::new();

        loop {
            match self.next_token() {
                Ok(token) => {
                    ts.push(token);
                },
                Err(e) => {
                    errors.push(e);
                },
            }

            if self.is_terminated {
                break;
            }
        }

        if !errors.is_empty() {
            return Err(errors);
        }

        if !self.cursor.is_eof() {
            let remaining = &self.src[self.pos as usize..];
            errors.push(
                LexerError::unconsumed_input(
                    self.line,
                    self.col,
                    remaining
                )
            );
            return Err(errors);
        }


        Ok(ts)
    }

    fn next_token(&mut self) -> Result<Token, LexerError> {
        self.skip_whitespace();

        let pos = self.pos;
        let col = self.col;
        let line = self.line;

        let token_kind = match self.cursor.first() {
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
            ';' => {
                self.bump();
                TokenKind::Semicolon
            },
            '/' => {
                self.bump();

                if self.cursor.first() == '/' {
                    self.bump();
                    while self.cursor.first() != '\n' {
                        self.bump();
                    }
                    TokenKind::LineComment
                }else {
                    TokenKind::Slash
                }
            },
            '!' => {
                self.bump();

                if self.cursor.first() == '=' {
                    self.bump();
                    TokenKind::BangEqual
                }else {
                    TokenKind::Bang
                }
            },
            '=' => {
                self.bump();

                if self.cursor.first() == '=' {
                    self.bump();
                    TokenKind::EqualEqual
                }else {
                    TokenKind::Equal
                }
            },
            '>' => {
                self.bump();

                if self.cursor.first() == '=' {
                    self.bump();
                    TokenKind::GreaterEqual
                }else {
                    TokenKind::Greater
                }
            },
            '<' => {
                self.bump();

                if self.cursor.first() == '=' {
                    self.bump();
                    TokenKind::LessEqual
                }else {
                    TokenKind::Less
                }
            },
            ('0'..='9') => {
                // LiteralKind::Integer, Floatのどちらかをとりうる
                let lit_kind = self.lex_number(col)?;
                TokenKind::Literal {
                    kind: lit_kind,
                    symbol: Symbol::intern(&self.src[pos as usize..self.pos as usize])
                }
            },
            '"' => {
                self.bump();

                self.lex_str_lit(line, col)?;

                TokenKind::Literal {
                    kind: LiteralKind::Str,
                    symbol: Symbol::intern(&self.src[pos as usize..self.pos as usize])
                }
            },
            c if c.is_alphabetic() => {
                self.bump();
                self.lex_word(pos)?
            },
            '\0' => {
                self.bump();
                self.is_terminated = true;

                TokenKind::Eof
            }
            c => {
                self.bump();
                self.is_terminated = true;

                Err(
                    LexerError::unexpected_character(
                        line,
                        col,
                        self.col,
                        c,
                    )
                )?
            },
        };

        Ok(
            Token {
                kind: token_kind,
                line,
                start: col,
                end: self.col,
            }
        )
    }

    fn bump(&mut self) -> Option<char> {
        let c = self.cursor.bump()?;

        self.pos += 1;
        self.col += 1;

        Some(c)
    }

    fn skip_whitespace(&mut self) {
        while self.cursor.first().is_whitespace() {
            match self.cursor.first() {
                '\n' => {
                    self.cursor.bump();
                    self.pos += 1;
                    self.col = 0;
                    self.line += 1;
                },
                _ => {
                    self.cursor.bump();
                    self.pos += 1;
                    self.col += 1;
                }
            }
        }
    }

    fn lex_number(&mut self, start: u32) -> Result<LiteralKind, LexerError> {
        // 最初に'.'が入力になることはない
        if let '0'..='9' = self.cursor.first() {
            self.bump();

            let mut is_float = false;

            while let c @ ('0'..='9') | c@ '.' = self.cursor.first() {
                match c {
                    '0'..='9' => {
                        self.bump();
                    },
                    '.' => {
                        if is_float {
                            Err(
                                LexerError::invalid_float_format(
                                    self.line,
                                    start,
                                    self.col,
                                )
                            )?
                        }

                        is_float = true;
                        self.bump();
                    },
                    _ => unreachable!()
                }

            }

            //最後の入力が'.'である(e.g. "123.")
            if self.cursor.prev == '.' {
                Err(
                    LexerError::missing_fractional_part(
                        self.line,
                        start,
                        self.col
                    )
                )?
            }

            if is_float {
                Ok(LiteralKind::Float)
            } else {
                Ok(LiteralKind::Integer)
            }
        } else {
            unreachable!()
        }
    }

    fn lex_str_lit(&mut self, line: u32, col: u32) -> Result<(), LexerError> {
        loop {
            match self.cursor.first() {
                '\\' => {
                    self.bump();
                    match self.cursor.first() {
                        'n' | 'r' | 't' | '0' | '\'' | '"' | '\\' => {
                            self.bump();
                        },
                        _ => {
                            self.bump();
                            self.is_terminated = true;
                            Err(
                                LexerError::invalid_escape_sequence(
                                    line,
                                    self.col - 1,
                                    self.col,
                                )
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
                    self.is_terminated = true;
                    Err(
                        LexerError::unterminated_string_literal(
                            line,
                            self.col,
                            col,
                        )
                    )?
                }
                _ => {
                    self.bump();
                }
            }
        }
    }

    fn lex_word(&mut self, pos: u32 ) -> Result<TokenKind, LexerError> {

        while matches!(self.cursor.first(), c if c.is_alphabetic() || c == '_' || c.is_numeric()) {
            self.bump();
        }

        let keyword_or_ident = &self.src[pos as usize..self.pos as usize];

        Ok(
            match self.as_keyword(keyword_or_ident) {
                Some(keyword) => keyword,
                None => {
                    if keyword_or_ident == "true" {
                        TokenKind::Literal {
                            kind: LiteralKind::Bool(true),
                            symbol: Symbol::intern(&self.src[pos as usize..self.pos as usize]),
                        }
                    } else if keyword_or_ident == "false" {
                        TokenKind::Literal {
                            kind: LiteralKind::Bool(false),
                            symbol: Symbol::intern(&self.src[pos as usize..self.pos as usize]),
                        }
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
