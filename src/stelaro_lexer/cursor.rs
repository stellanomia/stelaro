use std::str::Chars;

pub const EOF_CHAR: char = '\0';

#[derive(Debug, Clone)]
pub struct Cursor<'a> {
    chars: Chars<'a>,
    pub prev: char,
}

impl<'a> Cursor<'a> {
    pub fn new(src: &'a str) -> Self {
        Cursor {
            chars: src.chars(),
            prev: EOF_CHAR
        }
    }

    /// 入力から次の文字を読み取る。
    pub fn first(&self) -> char {
        // .nth(0)` よりも `.next()` の方が最適化される。
        self.chars.clone().next().unwrap_or(EOF_CHAR)
    }

    /// 入力から2つ次の文字を読み取る。
    pub fn second(&self) -> char {
        let mut chars = self.chars.clone();
        chars.next();
        chars.next().unwrap_or(EOF_CHAR)
    }

    //入力文字を一つ進める
    pub fn bump(&mut self) -> Option<char> {
        let c = self.chars.next()?;

        self.prev = c;

        Some(c)
    }
}
