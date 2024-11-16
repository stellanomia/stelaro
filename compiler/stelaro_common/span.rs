#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Span {
    pub line: u32,
    pub start: u32,
    pub end: u32,
}

impl Span {
    pub fn len(&self) -> usize {
        (self.end - self.start) as usize
    }

    /// Spanをマージして新しいSpanを作成する
    pub fn merge(&self, other: &Span) -> Self {
        Self {
            start: self.start.min(other.start),
            end: self.end.max(other.end),
            //TODO: マルチラインに対応させる
            line: self.line,
        }
    }
}