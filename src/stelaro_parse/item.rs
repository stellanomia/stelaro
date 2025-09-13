use crate::stelaro_ast::{ast::*, token::TokenKind};
use crate::stelaro_common::Ident;

use super::{diagnostics::DiagsParser, parser::Parser, PResult};


impl<'sess> Parser<'sess> {
    pub fn parse_item(&mut self) -> PResult<Option<Item>> {
        match self.token.kind {
            TokenKind::Fn => {
                let start = self.token.span;
                let (ident, f) = self.parse_fn()?;
                Ok(Some(
                    Item {
                        kind: ItemKind::Fn(Box::new(f)),
                        id: self.next_node_id(),
                        span: start.merge(&self.prev_token.span),
                        ident,
                    }
                ))
            },
            TokenKind::Mod => {
                let start = self.token.span;
                let (ident, items, mod_span) = self.parse_mod()?;

                Ok(Some(
                    Item {
                        kind: ItemKind::Mod(
                            ident,
                            ModKind::Inline(
                                items,
                                mod_span
                            )
                        ),
                        id: self.next_node_id(),
                        span: start.merge(&self.prev_token.span),
                        ident,
                    }
                ))
            },
            _ => {
                Ok(None)
            }
        }
    }

    // Itemが最初にとりうるトークンが増えたとき、ここに追加する
    pub fn can_start_item(&self) -> bool {
        matches!(self.token.kind,
            TokenKind::Fn |
            TokenKind::Mod
        )
    }

    pub fn parse_fn(&mut self) -> PResult<(Ident, Function)> {
        let start = self.token.span;
        self.eat(TokenKind::Fn, start)?;

        let ident = self.parse_ident()?;

        if ident.is_underscore() {
            Err(
                DiagsParser::cannot_use_underscore_as_identifier(
                    self.dcx(),
                    ident.span,
                ).emit()
            )?
        }

        if self.token.kind == TokenKind::LBrace {
            Err(
                DiagsParser::missing_function_parentheses(
                    self.dcx(),
                    ident,
                    self.token.span,
                ).emit()
            )?
        }

        let sig = self.parse_fn_sig()?;

        let prev_span = self.prev_token.span;

        if self.token.kind != TokenKind::LBrace {
            Err(
                DiagsParser::missing_function_body(
                    self.dcx(),
                    (prev_span.end..prev_span.end+1).into(),
                ).emit()
            )?
        }

        let body = self.parse_block()?;

        Ok((
            ident,
            Function {
                span: start.merge(&self.prev_token.span),
                ident,
                sig,
                body: Box::new(body),
            },
        ))
    }

    fn parse_fn_sig(&mut self) -> PResult<FnSig> {
        let start = self.prev_token.span;
        let params = self.parse_fn_params()?;

        let ret_ty = if self.token.kind == TokenKind::Colon {
            self.bump();

            let ty = self.parse_ty()?;

            FnRetTy::Ty(Box::new(ty))
        } else {
            // 返り値が入るべき場所を指す Span を生成する
            let start = self.prev_token.span.end;
            FnRetTy::Default((start..start).into())
        };

        Ok(
            FnSig {
                decl: FnDecl {
                    inputs: params,
                    output: ret_ty
                },
                span: start.merge(&self.prev_token.span)
            }
        )
    }

    fn parse_fn_params(&mut self) -> PResult<Vec<Param>> {
        if self.token.kind != TokenKind::LParen {
            Err(
                DiagsParser::unexpected_token_with_expected(
                    self.dcx(),
                    self.token.kind,
                    TokenKind::LParen,
                    self.token.span,
                ).emit()
            )?
        }

        self.bump();

        if self.token.kind == TokenKind::RParen {
            self.bump();

            Ok(Vec::with_capacity(0))
        } else {
            // f(,) を許可しない
            let mut params = vec![self.parse_fn_param()?];

            loop {
                match self.token.kind {
                    TokenKind::Comma => {
                        self.bump();

                        // f (x: i32, y: i32,) 及び
                        // f (
                        //   x: i32,
                        //   y: i32,
                        // ) を許可
                        if self.token.kind == TokenKind::RParen {
                            self.bump();

                            break;
                        }
                    }
                    _ if self.token.kind == TokenKind::RParen => {
                        self.bump();

                        break;
                    }
                    _ => {
                        let mut diag = DiagsParser::unexpected_token(
                            self.dcx(),
                            self.token.kind,
                            self.token.span,
                        );

                        diag.set_label(
                            self.token.span,
                            format!(
                                "`,`または`)`を期待しましたが、`{}`が見つかりました",
                                self.token.kind
                            ),
                        );
                        Err(diag.emit())?
                    }
                }

                let param = self.parse_fn_param()?;
                params.push(param);
            }

            Ok(params)
        }
    }

    fn parse_fn_param(&mut self) -> PResult<Param> {
        let start = self.token.span;

        let ident = self.parse_ident()?;

        if ident.is_underscore() {
            Err(
                DiagsParser::cannot_use_underscore_as_identifier(
                    self.dcx(),
                    ident.span,
                ).emit()
            )?
        }

        self.eat(TokenKind::Colon, self.token.span)?;

        let ty = self.parse_ty()?;

        Ok(Param {
            id: self.next_node_id(),
            ty: Box::new(ty),
            pat: Pat {
                id: self.next_node_id(),
                kind: PatKind::Ident(ident),
                span: ident.span,
            },
                span: start.merge(&self.prev_token.span),
        })
    }

    pub fn parse_mod(&mut self) -> PResult<(Ident, Vec<Box<Item>>, ModSpan)> {
        self.eat(TokenKind::Mod, self.token.span)?;

        let ident = self.parse_ident()?;

        self.eat(TokenKind::LBrace, self.token.span)?;
        let brace_span = self.prev_token.span;
        let mut inner_span = brace_span;

        let mut items = vec![];

        loop {
            match self.token.kind {
                TokenKind::RBrace => {
                    inner_span = inner_span.merge(&self.token.span);
                    self.bump();
                    break;
                }
                TokenKind::Semicolon => {
                    self.bump();
                    continue;
                }
                TokenKind::Eof => {
                    Err(
                        DiagsParser::unclosed_delimiter(
                            self.dcx(),
                            self.token,
                            brace_span
                        ).emit()
                    )?
                }
                _ => {
                    match self.parse_item()? {
                        Some(item) => items.push(Box::new(item)),
                        None => {
                            Err(
                                DiagsParser::unexpected_token_for_item(
                                    self.dcx(),
                                    self.token.kind,
                                    self.token.span
                                ).emit()
                            )?
                        },
                    }
                }
            }
        }

        Ok(
            (
                ident,
                items,
                ModSpan{
                    inner_span,
                }
            )
        )
    }
}
