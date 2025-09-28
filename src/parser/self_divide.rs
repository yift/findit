use crate::parser::ast::{expression::Expression, self_divide::SelfDivide};

impl SelfDivide {
    pub(super) fn new(expression: Expression) -> Self {
        Self {
            expression: Box::new(expression),
        }
    }
}
