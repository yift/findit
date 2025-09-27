use std::fmt::Display;

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
