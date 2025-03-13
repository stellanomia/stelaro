use std::rc::Rc;

use stelaro::{
    stelaro_ast::{ast::{BinOp, BinOpKind, Expr, ExprKind, NodeId}, token::{Lit, LiteralKind, Token, TokenKind}}, stelaro_common::{source_map::SourceMap, span::Span, symbol::{Ident, Symbol}}, stelaro_diagnostic::DiagCtxt, stelaro_lexer::Lexer, stelaro_parse::{new_parser_from_src, parser::Parser}, stelaro_session::Session
};

fn create_sess(src: Rc<String>) -> Session {
    let dcx = DiagCtxt::new(Rc::clone(&src));
    let source_map = Rc::new(SourceMap::new());
    Session::new(dcx, source_map)
}

fn create_parser(sess: &Session, src: Rc<String>) -> Parser<'_> {
    new_parser_from_src(sess, src.to_string()).unwrap()
}

#[test]
fn test_parse_expr() {
    let src = Rc::new(
        "x = (1 + 2) * 3 == 4 and 5 == 6 or 7 != 8 or 9 == 10 and true".to_string()
    );
    let sess = &create_sess(Rc::clone(&src));
    let mut parser = create_parser(sess, src);
    let expr = parser.parse_expr().unwrap();

    assert_eq!(
        expr,
        Expr {
            id: NodeId::dummy(),
            kind: ExprKind::Assign(
                Box::new(Expr {
                    id: NodeId::dummy(),
                    kind: ExprKind::Ident(
                        Ident::new(Symbol::new(0), (0..1).into())
                    ),
                    span: (0..1).into(),
                }),
                Box::new(Expr {
                    id: NodeId::dummy(),
                    kind: ExprKind::Binary(
                        BinOp {
                            kind: BinOpKind::Or,
                            span: (42..44).into()
                        },
                        Box::new(Expr {
                            id: NodeId::dummy(),
                            kind: ExprKind::Binary(
                                BinOp {
                                    kind: BinOpKind::Or,
                                    span: (32..34).into()
                                },
                                Box::new(Expr {
                                    id: NodeId::dummy(),
                                    kind: ExprKind::Binary(
                                        BinOp {
                                            kind: BinOpKind::And,
                                            span: (21..24).into()
                                        },
                                        Box::new(Expr {
                                            id: NodeId::dummy(),
                                            kind: ExprKind::Binary(
                                                BinOp {
                                                    kind: BinOpKind::Eq,
                                                    span: (16..18).into()
                                                },
                                                Box::new(Expr {
                                                    id: NodeId::dummy(),
                                                    kind: ExprKind::Binary(
                                                        BinOp {
                                                            kind: BinOpKind::Mul,
                                                            span: (12..13).into()
                                                        },
                                                        Box::new(Expr {
                                                            id: NodeId::dummy(),
                                                            kind: ExprKind::Paren(
                                                                Box::new(Expr {
                                                                    id: NodeId::dummy(),
                                                                    kind: ExprKind::Binary(
                                                                        BinOp {
                                                                            kind: BinOpKind::Add,
                                                                            span: (7..8).into()
                                                                        },
                                                                        Box::new(Expr {
                                                                            id: NodeId::dummy(),
                                                                            kind: ExprKind::Lit(
                                                                                Lit {
                                                                                    kind: LiteralKind::Integer,
                                                                                    symbol: Symbol::new(1),
                                                                                }
                                                                            ),
                                                                            span: (5..6).into(),
                                                                        }),
                                                                        Box::new(Expr {
                                                                            id: NodeId::dummy(),
                                                                            kind: ExprKind::Lit(
                                                                                Lit {
                                                                                    kind: LiteralKind::Integer,
                                                                                    symbol: Symbol::new(2),
                                                                                }
                                                                            ),
                                                                            span: (9..10).into(),
                                                                        })
                                                                    ),
                                                                    span: (5..10).into(),
                                                                })
                                                            ),
                                                            span: (4..11).into(),
                                                        }),
                                                        Box::new(Expr {
                                                            id: NodeId::dummy(),
                                                            kind: ExprKind::Lit(
                                                                Lit {
                                                                    kind: LiteralKind::Integer,
                                                                    symbol: Symbol::new(3),
                                                                }
                                                            ),
                                                            span: (14..15).into(),
                                                        })
                                                    ),
                                                    span: (4..15).into(),
                                                }),
                                                Box::new(Expr {
                                                    id: NodeId::dummy(),
                                                    kind: ExprKind::Lit(
                                                        Lit {
                                                            kind: LiteralKind::Integer,
                                                            symbol: Symbol::new(4),
                                                        }
                                                    ),
                                                    span: (19..20).into(),
                                                })
                                            ),
                                            span: (4..20).into(),
                                        }),
                                        Box::new(Expr {
                                            id: NodeId::dummy(),
                                            kind: ExprKind::Binary(
                                                BinOp {
                                                    kind: BinOpKind::Eq,
                                                    span: (27..29).into()
                                                },
                                                Box::new(Expr {
                                                    id: NodeId::dummy(),
                                                    kind: ExprKind::Lit(
                                                        Lit {
                                                            kind: LiteralKind::Integer,
                                                            symbol: Symbol::new(5),
                                                        }
                                                    ),
                                                    span: (25..26).into(),
                                                }),
                                                Box::new(Expr {
                                                    id: NodeId::dummy(),
                                                    kind: ExprKind::Lit(
                                                        Lit {
                                                            kind: LiteralKind::Integer,
                                                            symbol: Symbol::new(6),
                                                        }
                                                    ),
                                                    span: (30..31).into(),
                                                })
                                            ),
                                            span: (25..31).into(),
                                        })
                                    ),
                                    span: (4..31).into(),
                                }),
                                Box::new(Expr {
                                    id: NodeId::dummy(),
                                    kind: ExprKind::Binary(
                                        BinOp {
                                            kind: BinOpKind::Ne,
                                            span: (37..39).into()
                                        },
                                        Box::new(Expr {
                                            id: NodeId::dummy(),
                                            kind: ExprKind::Lit(
                                                Lit {
                                                    kind: LiteralKind::Integer,
                                                    symbol: Symbol::new(7),
                                                }
                                            ),
                                            span: (35..36).into(),
                                        }),
                                        Box::new(Expr {
                                            id: NodeId::dummy(),
                                            kind: ExprKind::Lit(
                                                Lit {
                                                    kind: LiteralKind::Integer,
                                                    symbol: Symbol::new(8),
                                                }
                                            ),
                                            span: (40..41).into(),
                                        })
                                    ),
                                    span: (35..41).into(),
                                })
                            ),
                            span: (4..41).into(),
                        }),
                        Box::new(Expr {
                            id: NodeId::dummy(),
                            kind: ExprKind::Binary(
                                BinOp {
                                    kind: BinOpKind::And,
                                    span: (53..56).into()
                                },
                                Box::new(Expr {
                                    id: NodeId::dummy(),
                                    kind: ExprKind::Binary(
                                        BinOp {
                                            kind: BinOpKind::Eq,
                                            span: (47..49).into()
                                        },
                                        Box::new(Expr {
                                            id: NodeId::dummy(),
                                            kind: ExprKind::Lit(
                                                Lit {
                                                    kind: LiteralKind::Integer,
                                                    symbol: Symbol::new(9),
                                                }
                                            ),
                                            span: (45..46).into(),
                                        }),
                                        Box::new(Expr {
                                            id: NodeId::dummy(),
                                            kind: ExprKind::Lit(
                                                Lit {
                                                    kind: LiteralKind::Integer,
                                                    symbol: Symbol::new(10),
                                                }
                                            ),
                                            span: (50..52).into(),
                                        })
                                    ),
                                    span: (45..52).into(),
                                }),
                                Box::new(Expr {
                                    id: NodeId::dummy(),
                                    kind: ExprKind::Lit(
                                        Lit {
                                            kind: LiteralKind::Bool(true),
                                            symbol: Symbol::new(11),
                                        }
                                    ),
                                    span: (57..61).into(),
                                })
                            ),
                            span: (45..61).into(),
                        })
                    ),
                    span: (4..61).into(),
                })
            ),
            span: (0..61).into(),
        }
    );
}