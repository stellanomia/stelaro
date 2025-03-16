use crate::{stelaro_ast::{ast::{BinOp, BinOpKind, Expr, ExprKind, NodeId, UnOp}, token::{Token, TokenKind}}, stelaro_common::{span::Span, symbol::Ident}};

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

        if self.can_start_expr() {
            Err(
                DiagsParser::missing_operator(
                    self.dcx(),
                    node.span.between(&self.token.span)
                ).emit()
            )?
        }

        Ok(node)
    }

    fn parse_expr_(&mut self, min_prec: PrecedenceLimit) -> PResult<Expr> {
        let mut lhs = self.parse_primary()?;

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
                self.check_non_associative_chain(&lhs)?
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

    fn check_non_associative_chain(&self, lhs: &Expr) -> PResult<()> {
        match &lhs.kind {
            ExprKind::Binary(bin_op, _, _) => {
                if AssocOp::from_binop(bin_op.kind).is_comparison() {
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
            | TokenKind::Minus       // 単項演算子 -
            | TokenKind::Bang        // 単項演算子 !
            | TokenKind::LParen
        )
    }

    fn parse_primary(&mut self) -> PResult<Expr> {
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
            TokenKind::Ident(symbol) => {
                self.bump();
                let ident_span = self.prev_token.span;

                Ok(
                    self.mk_expr(
                    ident_span,
                    ExprKind::Ident(
                        Ident::new(symbol, ident_span)
                    ))
                )
            }
            TokenKind::Minus => {
                self.bump();

                let start = self.prev_token.span;

                let node = self.parse_expr()?;

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

                let node = self.parse_expr()?;

                Ok(
                    self.mk_expr(
                        start.merge(&node.span),
                        ExprKind::Unary(UnOp::Not, Box::new(node))
                    )
                )

            },
            TokenKind::LParen => {
                self.bump();
                let start = self.prev_token.span;

                let node = self.parse_expr()?;

                let span = start.merge(&node.span).merge(&self.token.span);

                self.eat(TokenKind::RParen, self.token.span)?;

                Ok(
                    self.mk_expr(
                        self.prev_token.span.merge(&start.merge(&span)),
                        ExprKind::Paren(Box::new(node))
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
                }else {
                    Err(
                        DiagsParser::expect_expression(
                            self.dcx(),
                            self.prev_token,
                            self.prev_token.span
                        ).emit()
                    )
                }
            }
            _ => {
                Err(
                    DiagsParser::unexpected_token(
                        self.dcx(),
                        self.token,
                        self.token.span
                    ).emit()
                )
            }
        }
    }

    #[inline]
    fn mk_expr(&self, span: Span, kind: ExprKind) -> Expr {
        Expr { kind, span, id: NodeId::dummy() }
    }
}