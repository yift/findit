use std::{cmp::max, cmp::min, fmt::Display, ops::Add};

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Span {
    pub(crate) start: usize,
    pub(crate) end: usize,
}

impl Display for Span {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "At {} - {}", self.start, self.end)
    }
}

impl Add<&Span> for Span {
    type Output = Span;
    fn add(self, rhs: &Span) -> Self::Output {
        Span {
            start: min(self.start, rhs.start),
            end: max(self.end, rhs.end),
        }
    }
}
