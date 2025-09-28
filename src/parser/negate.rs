use crate::parser::ast::{expression::Expression, negate::Negate};
impl Negate {
    pub(super) fn new(expression: Expression) -> Self {
        Self {
            expression: Box::new(expression),
        }
    }
}
