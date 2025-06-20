use std::ops::Range;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Span {
    pub start: u32,
    pub end: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Hash)]
pub struct Spanned<T> {
    pub node: T,
    pub span: Span,
}


pub const DUMMY_SPAN: Span = Span { start: 0, end: 0 };

impl Span {
    pub fn len(&self) -> u32 {
        self.end - self.start
    }

    pub fn is_empty(&self) -> bool {
        self.start == self.end
    }

    /// Spanをマージして新しいSpanを作成する
    pub fn merge(&self, other: &Span) -> Self {
        Self {
            start: self.start.min(other.start),
            end: self.end.max(other.end),
        }
    }

    pub fn between(&self, other: &Span) -> Self {
        Self {
            start: self.end,
            end: other.start,
        }
    }

    pub fn as_range_usize(&self) -> Range<usize> {
        self.start as usize..self.end as usize
    }
}

impl From<Range<u32>> for Span {
    fn from(value: Range<u32>) -> Self {
        Span { start: value.start, end: value.end }
    }
}

impl From<Span> for Range<u32> {
    fn from(value: Span) -> Self {
        value.start..value.end
    }
}

impl From<Range<usize>> for Span {
    fn from(value: Range<usize>) -> Self {
        (value.start, value.end).into()
    }
}

impl From<Span> for Range<usize> {
    fn from(value: Span) -> Self {
        value.start as usize..value.end as usize
    }
}

impl From<Range<i32>> for Span {
    fn from(value: Range<i32>) -> Self {
        assert!(value.start <= value.end && value.start >= 0);
        (value.start as u32..value.end as u32).into()
    }
}

impl From<(u32, u32)> for Span {
    fn from((start, end): (u32, u32)) -> Self {
        Span { start, end }
    }
}

impl From<(usize, usize)> for Span {
    fn from((start, end): (usize, usize)) -> Self {
        assert!(start <= end && end <= u32::MAX as usize);
        Span { start: start as u32, end: end as u32 }
    }
}
