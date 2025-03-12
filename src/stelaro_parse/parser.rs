use crate::{stelaro_ast::token::{Token, TokenKind, TokenStream}, stelaro_session::Session};

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
            _ => None
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
            And => todo!(),
            Or => todo!(),
            Equal => todo!(),
            Less => todo!(),
            LessEqual => todo!(),
            NotEqual => todo!(),
            Greater => todo!(),
            GreaterEqual => todo!(),
            Assign => todo!(),
        }
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
    Sum,        // + -
    Product,    // * / %
    Prefix,     // ^
}

pub struct Parser<'sess> {
    sess: &'sess Session,
    token_stream: TokenStream,
    token: Token,
    prev_token: Token,
}

impl<'sess> Parser<'sess> {
    pub fn new(
        sess: &'sess Session,
        token_stream: TokenStream,
    ) -> Self {
        let mut parser = Parser {
            sess,
            token_stream,
            token: Token::dummy(),
            prev_token: Token::dummy(),
        };

        parser.bump();

        parser
    }

    pub fn bump(&mut self) {
        self.prev_token = self.token;

        self.token = self.token_stream.next().expect("bug: TokenStreamの範囲外アクセス");
    }

}