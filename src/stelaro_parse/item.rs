use crate::{stelaro_ast::{ast::{FnRetTy, FnSig, Function, Item, ItemKind, NodeId, Param}, token::TokenKind}, stelaro_common::symbol::Ident};

use super::{diagnostics::DiagsParser, parser::Parser, PResult};


impl<'sess> Parser<'sess> {
    pub fn parse_item(&mut self) -> PResult<Item> {
        match self.token.kind {
            TokenKind::Fn => {
                let start = self.prev_token.span;
                let (ident, f) = self.parse_fn()?;
                Ok(
                    Item {
                        kind: ItemKind::Function(f),
                        span: start.merge(&self.prev_token.span),
                        ident,
                    }
                )
            },
            _ => {
                Err(
                    DiagsParser::unexpected_token_with_expected_any(
                        self.dcx(),
                        self.token.kind,
                        // Itemが最初にとりうるトークンが増えたとき、ここに追加する
                        &[TokenKind::Fn],
                        self.token.span
                    ).emit()
                )?
            }
        }
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
                sig,
                body: Box::new(body),
            },
        ))
    }

    fn parse_fn_sig(&mut self) -> PResult<FnSig> {
        let start= self.prev_token.span;
        let params = self.parse_fn_params()?;

        let ret_ty = if self.token.kind == TokenKind::Equal {
            self.bump();
            self.eat(TokenKind::Greater, self.token.span)?;

            let ty = self.parse_ty()?;

            FnRetTy::Ty(Box::new(ty))
        } else {
            FnRetTy::Default
        };

        Ok(
            FnSig {
                params,
                ret_ty,
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
                    },
                    _ if self.token.kind == TokenKind::RParen => {
                        self.bump();

                        break;
                    },
                    _ => {
                        let mut diag = DiagsParser::unexpected_token(
                            self.dcx(),
                            self.token.kind,
                            self.token.span,
                        );

                        diag.set_label(
                            self.token.span,
                            format!("`,`または`)`を期待しましたが、`{}`が見つかりました",
                                self.token.kind
                            )
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

        Ok(
            Param {
                id: NodeId::dummy(),
                ty: Box::new(ty),
                ident,
                span: start.merge(&self.prev_token.span),
            }
        )
    }
}