use crate::stelaro_ast::{ast::{BinOp, BinOpKind, Expr, ExprKind, UnOp}, token::{Token, TokenKind}};
use crate::stelaro_common::span::Span;

use super::{diagnostics::DiagsParser, parser::Parser, PResult};

/// 中置演算子（AssocOp）の定義
#[derive(Copy, Clone, Debug, PartialEq)]
enum AssocOp {
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulus,
    And,
    Or,
    Equal,
    Less,
    LessEqual,
    NotEqual,
    Greater,
    GreaterEqual,
    Assign,
    // TODO: AssignOpの実装
    // ?= (e.g. +=, -=)
    // AssignOp
}

enum PrecedenceLimit {
    Inclusive(Precedence), // 優先順位がこの値以上なら許容
    Exclusive(Precedence), // 優先順位がこの値より大きい場合のみ許容
    None, // 制約なし（最初の式をパースする際など）
}

impl AssocOp {
    fn from_token(token: &Token) -> Option<Self> {
        match token.kind {
            TokenKind::Plus => Some(AssocOp::Add),
            TokenKind::Minus => Some(AssocOp::Subtract),
            TokenKind::Star => Some(AssocOp::Multiply),
            TokenKind::Slash => Some(AssocOp::Divide),
            TokenKind::Percent => Some(AssocOp::Modulus),
            TokenKind::BangEqual => Some(AssocOp::NotEqual),
            TokenKind::Equal => Some(AssocOp::Assign),
            TokenKind::EqualEqual => Some(AssocOp::Equal),
            TokenKind::Greater => Some(AssocOp::Greater),
            TokenKind::GreaterEqual => Some(AssocOp::GreaterEqual),
            TokenKind::Less => Some(AssocOp::Less),
            TokenKind::LessEqual => Some(AssocOp::LessEqual),
            TokenKind::And => Some(AssocOp::And),
            TokenKind::Or => Some(AssocOp::Or),
            _ => None,
        }
    }

    pub fn from_binop(op: BinOpKind) -> Self {
        use AssocOp::*;
        match op {
            BinOpKind::Lt => Less,
            BinOpKind::Gt => Greater,
            BinOpKind::Le => LessEqual,
            BinOpKind::Ge => GreaterEqual,
            BinOpKind::Eq => Equal,
            BinOpKind::Ne => NotEqual,
            BinOpKind::Mul => Multiply,
            BinOpKind::Div => Divide,
            BinOpKind::Mod => Modulus,
            BinOpKind::Add => Add,
            BinOpKind::Sub => Subtract,
            BinOpKind::And => And,
            BinOpKind::Or => Or,
        }
    }

    fn fixity(&self) -> Fixity {
        use AssocOp::*;

        match self {
            Assign => Fixity::Right,
            Add | Subtract | Multiply | Divide | Modulus | And | Or => Fixity::Left,
            Equal | Less | LessEqual | NotEqual | Greater | GreaterEqual => Fixity::NonAssoc
        }
    }

    fn precedence(&self) -> Precedence {
        use AssocOp::*;

        match self {
            Add | Subtract => Precedence::Sum,
            Multiply | Divide | Modulus => Precedence::Product,
            And => Precedence::And,
            Or => Precedence::Or,
            Equal | Less | LessEqual | NotEqual | Greater | GreaterEqual => Precedence::Cmp,
            Assign => Precedence::Assign,
        }
    }

    fn is_comparison(&self) -> bool {
        use AssocOp::*;
        matches!(*self, Less | Greater | LessEqual | GreaterEqual | Equal | NotEqual)
    }
}

/// 演算子の結合則
#[derive(Copy, Clone, Debug)]
enum Fixity {
    Left,
    Right,
    /// 比較演算子など、連続した場合に意味が不明確なもの
    NonAssoc,
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
enum Precedence {
    Assign,     // =
    Or,         // or
    And,        // and
    Cmp,        // < > <= >= == !=
    Sum,        // + -
    Product,    // * / %
}

impl Parser<'_> {
    pub fn parse_expr(&mut self) -> PResult<Expr> {
        let node = self.parse_expr_(PrecedenceLimit::None)?;

        if self.token.kind == TokenKind::RParen {
            self.bump();

            Err(
                DiagsParser::unexpected_closing_delimiter(
                    self.dcx(),
                    self.prev_token.span,
                ).emit()
            )?
        }

        Ok(node)
    }

    fn parse_expr_(&mut self, min_prec: PrecedenceLimit) -> PResult<Expr> {
        let mut lhs = self.parse_expr_primary()?;

        while let Some(op) = AssocOp::from_token(&self.token) {
            let prec = op.precedence();

            let should_break = match min_prec {
                PrecedenceLimit::Inclusive(min_prec) => prec < min_prec,
                PrecedenceLimit::Exclusive(min_prec) => prec <= min_prec,
                PrecedenceLimit::None => false,
            };

            if should_break {
                break;
            }

            self.bump();
            if op.is_comparison() {
                self.check_non_associative_chain(&lhs, op)?
            }

            let next_min_prec = match op.fixity() {
                Fixity::Left | Fixity::NonAssoc => PrecedenceLimit::Exclusive(prec),
                Fixity::Right => PrecedenceLimit::Inclusive(prec)
            };

            let op_token = self.prev_token;
            let rhs = self.parse_expr_(next_min_prec)?;
            let span = lhs.span.merge(&rhs.span);
            lhs = match op {
                AssocOp::Add |
                AssocOp::Subtract |
                AssocOp::Multiply |
                AssocOp::Divide |
                AssocOp::Modulus |
                AssocOp::Or |
                AssocOp::And |
                AssocOp::Equal |
                AssocOp::Less |
                AssocOp::LessEqual |
                AssocOp::Greater |
                AssocOp::GreaterEqual |
                AssocOp::NotEqual => {
                    self.mk_expr(
                        span,
                        ExprKind::Binary(
                            BinOp::from_token(op_token),
                            Box::new(lhs),
                            Box::new(rhs)
                        ),
                    )
                },
                AssocOp::Assign => {
                    self.mk_expr(
                        span,
                        ExprKind::Assign(
                            Box::new(lhs),
                            Box::new(rhs),
                        )
                    )
                }
            };
        }

        Ok(lhs)

    }

    /// 無効な連続した比較演算子を確認 `x < y < z`
    fn check_non_associative_chain(&self, lhs: &Expr, next: AssocOp) -> PResult<()> {
        match &lhs.kind {
            ExprKind::Binary(bin_op, _, _) => {
                if AssocOp::from_binop(bin_op.kind).is_comparison() && next.is_comparison() {
                    Err(
                        DiagsParser::chained_comparison(
                            self.dcx(), bin_op.span, self.prev_token.span
                        ).emit()
                    )
                } else {
                    Ok(())
                }
            },
            _ => {
                Ok(())
            }
        }
    }

    /// 現在のトークンが新しい式の開始として適切かどうかを判定
    fn can_start_expr(&self) -> bool {
        matches!(self.token.kind,
            TokenKind::Literal(_)
            | TokenKind::Ident(_)
            | TokenKind::Minus  // 単項演算子 -
            | TokenKind::Bang   // 単項演算子 !
            | TokenKind::LParen
            | TokenKind::If     // If式
            | TokenKind::LBrace // ブロック式 {}
        )
    }

    /// 単項演算子の解析
    fn parse_expr_primary(&mut self) -> PResult<Expr> {
        match self.token.kind {
            TokenKind::Minus => {
                self.bump();

                let start = self.prev_token.span;

                let node = self.parse_expr_(PrecedenceLimit::None)?;

                Ok(
                    self.mk_expr(
                        start.merge(&node.span),
                        ExprKind::Unary(UnOp::Neg, Box::new(node))
                    )
                )

            },
            TokenKind::Bang => {
                self.bump();

                let start = self.prev_token.span;

                let node = self.parse_expr_(PrecedenceLimit::None)?;

                Ok(
                    self.mk_expr(
                        start.merge(&node.span),
                        ExprKind::Unary(UnOp::Not, Box::new(node))
                    )
                )

            },
            TokenKind::Plus |
            TokenKind::Star |
            TokenKind::Slash => {
                self.bump();

                if self.token.kind == TokenKind::Plus
                    && self.prev_token.kind == TokenKind::Plus {
                    Err(
                        DiagsParser::prefix_increment(
                            self.dcx(),
                            self.prev_token.span.merge(&self.token.span)
                        ).emit()
                    )
                } else {
                    Err(
                        DiagsParser::expect_expression(
                            self.dcx(),
                            self.prev_token,
                            self.prev_token.span
                        ).emit()
                    )
                }
            },
            TokenKind::RParen | TokenKind::RBrace => {
                Err(
                    DiagsParser::unexpected_closing_delimiter(
                        self.dcx(),
                        self.token.span,
                    ).emit()
                )?
            },
            _ if !self.can_start_expr() => {
                Err(
                    DiagsParser::expect_expression(
                        self.dcx(),
                        self.token,
                        self.token.span
                    ).emit()
                )
            },
            _ => {
                self.parse_expr_postfix()
            }
        }
    }

    // TODO: インデックスアクセス、`.`によるメソッド呼び出しのサポート
    fn parse_expr_postfix(&mut self) -> PResult<Expr> {
        let node = self.parse_expr_bottom()?;

        match self.token.kind {
            TokenKind::LParen => self.parse_expr_fn_call(node.span, node),
            _ => Ok(node)
        }
    }

    // TODO: タプル、while(for, loop)式、配列 のサポート
    /// 優先順位が最も低く、括弧で囲まれた式などを解析する
    fn parse_expr_bottom(&mut self) -> PResult<Expr> {
        match self.token.kind {
            TokenKind::Literal(lit) => {
                self.bump();

                Ok(
                    self.mk_expr(
                        self.prev_token.span,
                        ExprKind::Lit(lit)
                    )
                )
            },
            TokenKind::Ident(_) => {
                let start = self.token.span;

                let path = self.parse_path()?;

                Ok(
                    self.mk_expr(
                    start.merge(&self.prev_token.span),
                    ExprKind::Path(
                        path
                    ))
                )
            }
            TokenKind::LParen => {
                self.bump();
                let start = self.prev_token.span;

                let node = self.parse_expr_(PrecedenceLimit::None)?;

                let span = start.merge(&self.token.span);

                self.eat(TokenKind::RParen, self.token.span)?;

                Ok(
                    self.mk_expr(
                        self.prev_token.span.merge(&start.merge(&span)),
                        ExprKind::Paren(Box::new(node))
                    )
                )
            },
            TokenKind::LBrace => {
                let start = self.token.span;

                let block = self.parse_block()?;

                let span = start.merge(&self.prev_token.span);

                Ok(
                    self.mk_expr(
                        span,
                        ExprKind::Block(
                            Box::new(block)
                        )
                    )
                )
            },
            TokenKind::If => {
                self.parse_if()
            },
            _ => {
                Err(
                    DiagsParser::unexpected_token(
                        self.dcx(),
                        self.token.kind,
                        self.token.span
                    ).emit()
                )
            }
        }
    }

    fn parse_expr_fn_call(&mut self, start: Span, f: Expr) -> PResult<Expr> {
        let seq = self.parse_delim_comma_seq(TokenKind::LParen, TokenKind::RParen)?;

        Ok(
            self.mk_expr(
                start.merge(&self.prev_token.span),
                ExprKind::Call(
                    Box::new(f),
                    seq
                ),
            )
        )
    }

    /// コンマで区切られた列をパース
    fn parse_delim_comma_seq(&mut self, open: TokenKind, close: TokenKind) -> PResult<Vec<Expr>> {
        if self.token.kind != open {
            Err(
                DiagsParser::unexpected_token_with_expected(
                    self.dcx(),
                    self.token.kind,
                    open,
                    self.token.span,
                ).emit()
            )?
        }

        self.bump();

        if self.token.kind == close {
            self.bump();

            Ok(Vec::with_capacity(0))
        } else {
            // f(,) を許可しない
            let mut args = vec![self.parse_expr_(PrecedenceLimit::None)?];

            loop {
                match self.token.kind {
                    TokenKind::Comma => {
                        self.bump();

                        // f (0,1,) 及び
                        // f (
                        //   0,
                        //   1,
                        // ) を許可
                        if self.token.kind == close {
                            self.bump();

                            break;
                        }
                    },
                    _ if self.token.kind == close => {
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

                let arg = self.parse_expr_(PrecedenceLimit::None)?;
                args.push(arg);
            }

            Ok(args)
        }
    }

    pub fn parse_if(&mut self) -> PResult<Expr> {
        self.eat(TokenKind::If, self.token.span)?;
        let start = self.prev_token.span;

        let cond = self.parse_expr()?;

        let block = self.parse_block()?;

        let else_branch = if self.token.kind == TokenKind::Else {
            self.bump();

            // else ifの場合
            if self.token.kind == TokenKind::If {
                Some(Box::new(self.parse_if()?))
            } else {
                // 通常のelseブロック
                let block = self.parse_block()?;

                Some(
                    Box::new(
                        self.mk_expr(
                            block.span,
                            ExprKind::Block(Box::new(block))
                        )
                    )
                )
            }
        } else {
            None
        };

        Ok(
            self.mk_expr(
                start.merge(&self.prev_token.span),
                ExprKind::If (
                    Box::new(cond),
                    Box::new(block),
                    else_branch,
                )
            )
        )
    }
}