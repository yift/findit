#[cfg(test)]
mod tests {

    use crate::{
        parser::{
            ast::order_by::{OrderByDirection, OrderByExpression, OrderByItem},
            ast::{access::Access, expression::Expression},
            parse_order_by,
            parser_error::ParserError,
        },
        value::Value,
    };

    #[test]
    fn test_order_by_single_item() -> Result<(), ParserError> {
        let source = "1";
        let ast = parse_order_by(source)?;

        assert_eq!(
            ast,
            OrderByExpression {
                items: vec![OrderByItem {
                    expression: Expression::Literal(Value::Number(1)),
                    direction: OrderByDirection::Asc,
                }]
            }
        );

        Ok(())
    }

    #[test]
    fn test_order_by_three_items() -> Result<(), ParserError> {
        let source = "1, false, name";
        let ast = parse_order_by(source)?;

        assert_eq!(
            ast,
            OrderByExpression {
                items: vec![
                    OrderByItem {
                        expression: Expression::Literal(Value::Number(1)),
                        direction: OrderByDirection::Asc,
                    },
                    OrderByItem {
                        expression: Expression::Literal(Value::Bool(false)),
                        direction: OrderByDirection::Asc,
                    },
                    OrderByItem {
                        expression: Expression::Access(Access::Name),
                        direction: OrderByDirection::Asc,
                    },
                ]
            }
        );

        Ok(())
    }

    #[test]
    fn test_order_by_three_items_asc_and_desc() -> Result<(), ParserError> {
        let source = "1 asc, false, name desc";
        let ast = parse_order_by(source)?;

        assert_eq!(
            ast,
            OrderByExpression {
                items: vec![
                    OrderByItem {
                        expression: Expression::Literal(Value::Number(1)),
                        direction: OrderByDirection::Asc,
                    },
                    OrderByItem {
                        expression: Expression::Literal(Value::Bool(false)),
                        direction: OrderByDirection::Asc,
                    },
                    OrderByItem {
                        expression: Expression::Access(Access::Name),
                        direction: OrderByDirection::Desc,
                    },
                ]
            }
        );

        Ok(())
    }

    #[test]
    fn test_order_by_no_comma() {
        let source = "10 20";
        let err = parse_order_by(source).err();

        assert!(err.is_some());
    }

    #[test]
    fn test_order_by_desc_no_comma() {
        let source = "10 desc 20";
        let err = parse_order_by(source).err();

        assert!(err.is_some());
    }
}
