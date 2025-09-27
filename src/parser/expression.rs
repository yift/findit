use std::iter::Peekable;
use thiserror::Error;

use crate::{
    parser::{
        access::Access,
        between::{Between, build_between},
        binary_expression::BinaryExpression,
        case::{Case, build_case},
        execute::{SpawnOrExecute, build_spawn_or_exec},
        function::{Function, build_function},
        if_expression::{If, build_if},
        is_check::{IsCheck, IsType},
        lexer::{LexerError, LexerItem, lex},
        negate::Negate,
        operator::{ArithmeticOperator, BinaryOperator, LogicalOperator},
        position::{Position, build_position},
        self_divide::SelfDivide,
        span::Span,
        substr::{Substring, build_substring},
        tokens::Token,
    },
    value::Value,
};

#[derive(Debug, PartialEq)]
pub(crate) enum Expression {
    Literal(Value),
    Binary(BinaryExpression),
    Negate(Negate),
    Brackets(Box<Expression>),
    Access(Access),
    IsCheck(IsCheck),
    If(If),
    Case(Case),
    Between(Between),
    Position(Position),
    Substring(Substring),
    Function(Function),
    SpawnOrExecute(SpawnOrExecute),
    SelfDivide(SelfDivide),
}

#[derive(Error, Debug)]
pub enum ParserError {
    #[error("Lexer error: `{0}`")]
    LexerError(#[from] LexerError),
    #[error("Unexpected end of expression")]
    UnexpectedEof,
    #[error("Unexpected token at `{0}`")]
    UnexpectedToken(Span),
    #[error("Case without any branches `{0}`")]
    NoBranches(Span),
}

pub(crate) fn parse_expression(source: &str) -> Result<Expression, ParserError> {
    let mut lexer = lex(source)?;

    build_expression_with_priority(&mut lexer, 0, |f| f.is_none())
}

fn build_brackets(
    lex: &mut Peekable<impl Iterator<Item = LexerItem>>,
) -> Result<Expression, ParserError> {
    let left = build_expression_with_priority(lex, 0, |f| f == Some(&Token::CloseBrackets))?;
    lex.next();
    Ok(Expression::Brackets(Box::new(left)))
}
pub(super) fn build_expression_with_priority(
    lex: &mut Peekable<impl Iterator<Item = LexerItem>>,
    minimum_priority: u8,
    end_condition: fn(Option<&Token>) -> bool,
) -> Result<Expression, ParserError> {
    let mut left = match lex.next() {
        None => return Err(ParserError::UnexpectedEof),
        Some(item) => match item.token {
            Token::Value(value) => Expression::Literal(value),
            Token::OpenBrackets => build_brackets(lex)?,
            Token::If => build_if(lex)?,
            Token::Case => build_case(lex, &item.span)?,
            Token::Position => build_position(lex)?,
            Token::Substring => build_substring(lex)?,
            Token::FunctionName(name) => build_function(name, lex)?,
            Token::Not => {
                let expression = build_expression_with_priority(lex, 30, end_condition)?;
                Expression::Negate(Negate::new(expression))
            }
            Token::BinaryOperator(BinaryOperator::Arithmetic(ArithmeticOperator::Divide)) => {
                let expression = build_expression_with_priority(lex, 30, end_condition)?;
                Expression::SelfDivide(SelfDivide::new(expression))
            }
            Token::SimpleAccess(access) => Expression::Access(access),
            Token::Is => {
                let access = read_prefix_is(lex)?;
                Expression::Access(access)
            }
            Token::Spawn => build_spawn_or_exec(true, lex)?,
            Token::Execute => build_spawn_or_exec(false, lex)?,
            _ => return Err(ParserError::UnexpectedToken(item.span)),
        },
    };

    loop {
        let next = lex.peek();
        if end_condition(next.map(|f| &f.token)) {
            break;
        }

        let operator = match next {
            None => return Err(ParserError::UnexpectedEof),
            Some(item) => match item.token {
                Token::BinaryOperator(operator) => Operator::Binary(operator),
                Token::Is => Operator::PostIs,
                Token::Between => Operator::Between,
                _ => return Err(ParserError::UnexpectedToken(item.span)),
            },
        };
        let priority = operator.priority();
        if priority <= minimum_priority {
            break;
        }
        match operator {
            Operator::Binary(operator) => {
                lex.next();
                let right = build_expression_with_priority(lex, priority, end_condition)?;
                left = Expression::Binary(BinaryExpression::new(left, operator, right));
            }
            Operator::PostIs => {
                left = read_postfix_is(left, lex)?;
            }
            Operator::Between => {
                left = build_between(left, lex)?;
            }
        }
    }
    Ok(left)
}

#[derive(Debug)]
enum Operator {
    Binary(BinaryOperator),
    PostIs,
    Between,
}

impl Operator {
    fn priority(&self) -> u8 {
        match self {
            Operator::Binary(BinaryOperator::Of) => 5,
            Operator::Between => 10,
            Operator::Binary(BinaryOperator::Logical(LogicalOperator::Or)) => 10,
            Operator::Binary(BinaryOperator::Logical(LogicalOperator::Xor)) => 15,
            Operator::Binary(BinaryOperator::Logical(LogicalOperator::And)) => 20,
            Operator::Binary(BinaryOperator::Comparison(_)) => 40,
            Operator::Binary(BinaryOperator::Matches) => 40,
            Operator::PostIs => 40,
            Operator::Binary(BinaryOperator::Arithmetic(ArithmeticOperator::Plus)) => 50,
            Operator::Binary(BinaryOperator::Arithmetic(ArithmeticOperator::Minus)) => 50,
            Operator::Binary(BinaryOperator::BitwiseOperator(_)) => 50,
            Operator::Binary(BinaryOperator::Arithmetic(ArithmeticOperator::Multiply)) => 80,
            Operator::Binary(BinaryOperator::Arithmetic(ArithmeticOperator::Divide)) => 80,
            Operator::Binary(BinaryOperator::Arithmetic(ArithmeticOperator::Module)) => 80,
            Operator::Binary(BinaryOperator::Dot) => 110,
        }
    }
}

fn read_prefix_is(
    lex: &mut Peekable<impl Iterator<Item = LexerItem>>,
) -> Result<Access, ParserError> {
    let Some(next) = lex.next() else {
        return Err(ParserError::UnexpectedEof);
    };
    let (next, negate) = if next.token == Token::Not {
        let Some(next) = lex.next() else {
            return Err(ParserError::UnexpectedEof);
        };
        (next, true)
    } else {
        (next, false)
    };
    let access = match next.token {
        Token::Dir => {
            if negate {
                Access::IsNotDir
            } else {
                Access::IsDir
            }
        }
        Token::File => {
            if negate {
                Access::IsNotFile
            } else {
                Access::IsFile
            }
        }
        Token::Link => {
            if negate {
                Access::IsNotLink
            } else {
                Access::IsLink
            }
        }
        _ => return Err(ParserError::UnexpectedToken(next.span)),
    };
    Ok(access)
}

fn read_postfix_is(
    left: Expression,
    lex: &mut Peekable<impl Iterator<Item = LexerItem>>,
) -> Result<Expression, ParserError> {
    lex.next();
    let Some(next) = lex.next() else {
        return Err(ParserError::UnexpectedEof);
    };
    let (next, negate) = if next.token == Token::Not {
        let Some(next) = lex.next() else {
            return Err(ParserError::UnexpectedEof);
        };
        (next, true)
    } else {
        (next, false)
    };

    let check_type = IsType::try_from(next)?;

    Ok(Expression::IsCheck(IsCheck::new(left, check_type, negate)))
}

#[cfg(test)]
mod tests {

    use crate::parser::{
        case::CaseBranch,
        function_name::{EnvFunctionName, FunctionName, StringFunctionName},
        operator::ComparisonOperator,
    };

    use super::*;

    fn lit_u64(number: u64) -> Expression {
        Expression::Literal(number.into())
    }
    fn lit_b(b: bool) -> Expression {
        Expression::Literal(b.into())
    }
    fn lit_s(s: &str) -> Expression {
        Expression::Literal(s.into())
    }
    fn bin_e(left: Expression, operator: BinaryOperator, right: Expression) -> Expression {
        Expression::Binary(BinaryExpression::new(left, operator, right))
    }
    fn brackets(exp: Expression) -> Expression {
        Expression::Brackets(Box::new(exp))
    }
    fn negate(exp: Expression) -> Expression {
        Expression::Negate(Negate::new(exp))
    }
    fn if2(condition: Expression, then_branch: Expression, else_branch: Expression) -> Expression {
        Expression::If(If::new(condition, then_branch, Some(else_branch)))
    }
    fn if1(condition: Expression, then_branch: Expression) -> Expression {
        Expression::If(If::new(condition, then_branch, None))
    }
    fn case2(branches: Vec<(Expression, Expression)>, default_outcome: Expression) -> Expression {
        let branches: Vec<_> = branches
            .into_iter()
            .map(|(condition, outcome)| CaseBranch::new(condition, outcome))
            .collect();
        Expression::Case(Case::new(branches, Some(default_outcome)))
    }
    fn case(branches: Vec<(Expression, Expression)>) -> Expression {
        let branches: Vec<_> = branches
            .into_iter()
            .map(|(condition, outcome)| CaseBranch::new(condition, outcome))
            .collect();
        Expression::Case(Case::new(branches, None))
    }

    fn access(acc: Access) -> Expression {
        Expression::Access(acc)
    }

    fn is_(negate: bool, is_type: IsType, exp: Expression) -> Expression {
        Expression::IsCheck(IsCheck::new(exp, is_type, negate))
    }
    fn between(reference: Expression, lower: Expression, upper: Expression) -> Expression {
        Expression::Between(Between::new(reference, lower, upper))
    }
    fn position(sub_string: Expression, super_string: Expression) -> Expression {
        Expression::Position(Position::new(sub_string, super_string))
    }

    fn sub_string(
        super_string: &str,
        substring_from: Option<u64>,
        substring_for: Option<u64>,
    ) -> Expression {
        let substring_from = substring_from.map(lit_u64);
        let substring_for = substring_for.map(lit_u64);
        let super_string = lit_s(super_string);
        Expression::Substring(Substring::new(super_string, substring_from, substring_for))
    }

    fn func(name: FunctionName, args: Vec<Expression>) -> Expression {
        Expression::Function(Function::new(name, args))
    }

    fn spawn(bin: Expression, args: Vec<Expression>, into: Option<Expression>) -> Expression {
        Expression::SpawnOrExecute(SpawnOrExecute::new(true, bin, args, into))
    }
    fn exec(bin: Expression, args: Vec<Expression>, into: Option<Expression>) -> Expression {
        Expression::SpawnOrExecute(SpawnOrExecute::new(false, bin, args, into))
    }

    #[test]
    fn parse_simple_literal() -> Result<(), ParserError> {
        let str = "23004";
        let exp = parse_expression(str)?;

        assert_eq!(exp, Expression::Literal(Value::Number(23004)));
        Ok(())
    }

    #[test]
    fn parse_simple_with_white_spaces() -> Result<(), ParserError> {
        let str = "  false  ";
        let exp = parse_expression(str)?;

        assert_eq!(exp, Expression::Literal(Value::Bool(false)));
        Ok(())
    }

    #[test]
    fn parse_two_binary_operators_with_the_same_priority() -> Result<(), ParserError> {
        let str = "1+3+4-10";
        let exp = parse_expression(str)?;

        assert_eq!(
            exp,
            bin_e(
                bin_e(
                    bin_e(
                        lit_u64(1),
                        BinaryOperator::Arithmetic(ArithmeticOperator::Plus),
                        lit_u64(3),
                    ),
                    BinaryOperator::Arithmetic(ArithmeticOperator::Plus),
                    lit_u64(4),
                ),
                BinaryOperator::Arithmetic(ArithmeticOperator::Minus),
                lit_u64(10),
            )
        );

        Ok(())
    }

    #[test]
    fn parse_two_binary_operators_with_different_priority() -> Result<(), ParserError> {
        let str = "1+3*4-10";
        let exp = parse_expression(str)?;

        assert_eq!(
            exp,
            bin_e(
                bin_e(
                    lit_u64(1),
                    BinaryOperator::Arithmetic(ArithmeticOperator::Plus),
                    bin_e(
                        lit_u64(3),
                        BinaryOperator::Arithmetic(ArithmeticOperator::Multiply),
                        lit_u64(4),
                    ),
                ),
                BinaryOperator::Arithmetic(ArithmeticOperator::Minus),
                lit_u64(10),
            )
        );

        Ok(())
    }

    #[test]
    fn parse_with_unexpected_numbers() {
        let str = "1+3 11";
        let err = parse_expression(str).err();

        assert!(err.is_some());
    }

    #[test]
    fn parse_with_unexpected_close_brackets() {
        let str = "1+3 (";
        let err = parse_expression(str).err();

        assert!(err.is_some());
    }

    #[test]
    fn parse_empty() {
        let str = "";
        let err = parse_expression(str).err();

        assert!(err.is_some());
    }

    #[test]
    fn parse_brackets_return_the_correct_order() -> Result<(), ParserError> {
        let str = "(1+3)*(4-10)";
        let exp = parse_expression(str)?;

        assert_eq!(
            exp,
            bin_e(
                brackets(bin_e(
                    lit_u64(1),
                    BinaryOperator::Arithmetic(ArithmeticOperator::Plus),
                    lit_u64(3),
                )),
                BinaryOperator::Arithmetic(ArithmeticOperator::Multiply),
                brackets(bin_e(
                    lit_u64(4),
                    BinaryOperator::Arithmetic(ArithmeticOperator::Minus),
                    lit_u64(10),
                )),
            )
        );

        Ok(())
    }

    #[test]
    fn parse_logical_expression() -> Result<(), ParserError> {
        let str = "10 > 4 OR 12 < 6 XOR NOT 20 = 6 AND true";
        let exp = parse_expression(str)?;

        assert_eq!(
            exp,
            bin_e(
                bin_e(
                    lit_u64(10),
                    BinaryOperator::Comparison(ComparisonOperator::LargerThen),
                    lit_u64(4),
                ),
                BinaryOperator::Logical(LogicalOperator::Or),
                bin_e(
                    bin_e(
                        lit_u64(12),
                        BinaryOperator::Comparison(ComparisonOperator::SmallerThen),
                        lit_u64(6),
                    ),
                    BinaryOperator::Logical(LogicalOperator::Xor),
                    bin_e(
                        negate(bin_e(
                            lit_u64(20),
                            BinaryOperator::Comparison(ComparisonOperator::Eq),
                            lit_u64(6),
                        )),
                        BinaryOperator::Logical(LogicalOperator::And),
                        lit_b(true),
                    ),
                ),
            )
        );
        Ok(())
    }

    #[test]
    fn access_with_of_and_is_some() -> Result<(), ParserError> {
        let str = "content of parent is not some";
        let exp = parse_expression(str)?;

        assert_eq!(
            exp,
            bin_e(
                access(Access::Content),
                BinaryOperator::Of,
                is_(true, IsType::Some, access(Access::Parent)),
            )
        );

        Ok(())
    }

    #[test]
    fn if_with_else() -> Result<(), ParserError> {
        let str = "if 10>=20 Then 30 else 40 end";
        let exp = parse_expression(str)?;

        assert_eq!(
            exp,
            if2(
                bin_e(
                    lit_u64(10),
                    BinaryOperator::Comparison(ComparisonOperator::LargerThenEq),
                    lit_u64(20),
                ),
                lit_u64(30),
                lit_u64(40),
            )
        );

        Ok(())
    }

    #[test]
    fn if_without_else() -> Result<(), ParserError> {
        let str = "if 10>=20 Then 30 end";
        let exp = parse_expression(str)?;

        assert_eq!(
            exp,
            if1(
                bin_e(
                    lit_u64(10),
                    BinaryOperator::Comparison(ComparisonOperator::LargerThenEq),
                    lit_u64(20),
                ),
                lit_u64(30),
            )
        );

        Ok(())
    }

    #[test]
    fn case_with_else() -> Result<(), ParserError> {
        let str = "case when 10 == 10 then 1 when 20 != 20 then 2 when 30 <> 30 then 3 else 4 end";
        let exp = parse_expression(str)?;

        assert_eq!(
            exp,
            case2(
                vec![
                    (
                        bin_e(
                            lit_u64(10),
                            BinaryOperator::Comparison(ComparisonOperator::Eq),
                            lit_u64(10),
                        ),
                        lit_u64(1),
                    ),
                    (
                        bin_e(
                            lit_u64(20),
                            BinaryOperator::Comparison(ComparisonOperator::Neq),
                            lit_u64(20),
                        ),
                        lit_u64(2),
                    ),
                    (
                        bin_e(
                            lit_u64(30),
                            BinaryOperator::Comparison(ComparisonOperator::Neq),
                            lit_u64(30),
                        ),
                        lit_u64(3),
                    ),
                ],
                lit_u64(4),
            )
        );

        Ok(())
    }

    #[test]
    fn case_without_else() -> Result<(), ParserError> {
        let str = "case when 10 == 10 then 1 when 20 != 20 then 2 when 30 <> 30 then 3 end";
        let exp = parse_expression(str)?;

        assert_eq!(
            exp,
            case(vec![
                (
                    bin_e(
                        lit_u64(10),
                        BinaryOperator::Comparison(ComparisonOperator::Eq),
                        lit_u64(10),
                    ),
                    lit_u64(1),
                ),
                (
                    bin_e(
                        lit_u64(20),
                        BinaryOperator::Comparison(ComparisonOperator::Neq),
                        lit_u64(20),
                    ),
                    lit_u64(2),
                ),
                (
                    bin_e(
                        lit_u64(30),
                        BinaryOperator::Comparison(ComparisonOperator::Neq),
                        lit_u64(30),
                    ),
                    lit_u64(3),
                ),
            ])
        );

        Ok(())
    }

    #[test]
    fn parse_case_with_no_branches() {
        let str = "CASE END";
        let err = parse_expression(str).err();

        assert!(err.is_some());
    }

    #[test]
    fn parse_case_with_no_end() {
        let str = "CASE WHEN 10 > 20 THEN 30 ELSE 40";
        let err = parse_expression(str).err();

        assert!(err.is_some());
    }

    #[test]
    fn is_file() -> Result<(), ParserError> {
        let str = "IS FILE";
        let exp = parse_expression(str)?;

        assert_eq!(exp, access(Access::IsFile));

        Ok(())
    }

    #[test]
    fn is_not_file() -> Result<(), ParserError> {
        let str = "IS NOT file";
        let exp = parse_expression(str)?;

        assert_eq!(exp, access(Access::IsNotFile));

        Ok(())
    }

    #[test]
    fn is_dir() -> Result<(), ParserError> {
        let str = "IS dir";
        let exp = parse_expression(str)?;

        assert_eq!(exp, access(Access::IsDir));

        Ok(())
    }

    #[test]
    fn is_not_dir() -> Result<(), ParserError> {
        let str = "IS NOT DIR";
        let exp = parse_expression(str)?;

        assert_eq!(exp, access(Access::IsNotDir));

        Ok(())
    }

    #[test]
    fn is_link() -> Result<(), ParserError> {
        let str = "IS Link";
        let exp = parse_expression(str)?;

        assert_eq!(exp, access(Access::IsLink));

        Ok(())
    }

    #[test]
    fn is_not_link() -> Result<(), ParserError> {
        let str = "IS NOT link";
        let exp = parse_expression(str)?;

        assert_eq!(exp, access(Access::IsNotLink));

        Ok(())
    }

    #[test]
    fn start_with_binary_operator() {
        let str = "+ 20";
        let err = parse_expression(str).err();

        assert!(err.is_some());
    }

    #[test]
    fn nothing_to_add() {
        let str = "20 +";
        let err = parse_expression(str).err();

        assert!(err.is_some());
    }

    #[test]
    fn nothing_with_is() {
        let str = "IS";
        let err = parse_expression(str).err();

        assert!(err.is_some());
    }

    #[test]
    fn nothing_with_is_not() {
        let str = "IS NOT";
        let err = parse_expression(str).err();

        assert!(err.is_some());
    }

    #[test]
    fn is_then() {
        let str = "IS THEN";
        let err = parse_expression(str).err();

        assert!(err.is_some());
    }

    #[test]
    fn is_what() {
        let str = "true IS";
        let err = parse_expression(str).err();

        assert!(err.is_some());
    }

    #[test]
    fn is_then_two() {
        let str = "true IS THEN";
        let err = parse_expression(str).err();

        assert!(err.is_some());
    }

    #[test]
    fn is_not_what() {
        let str = "true IS NOT";
        let err = parse_expression(str).err();

        assert!(err.is_some());
    }

    #[test]
    fn just_case() {
        let str = "case";
        let err = parse_expression(str).err();

        assert!(err.is_some());
    }

    #[test]
    fn just_if() {
        let str = "if";
        let err = parse_expression(str).err();

        assert!(err.is_some());
    }

    #[test]
    fn is_true() -> Result<(), ParserError> {
        let str = "true is true";
        let exp = parse_expression(str)?;

        assert_eq!(exp, is_(false, IsType::True, lit_b(true)));

        Ok(())
    }

    #[test]
    fn is_false() -> Result<(), ParserError> {
        let str = "true is false";
        let exp = parse_expression(str)?;

        assert_eq!(exp, is_(false, IsType::False, lit_b(true)));

        Ok(())
    }

    #[test]
    fn is_not_true() -> Result<(), ParserError> {
        let str = "true is NOT true";
        let exp = parse_expression(str)?;

        assert_eq!(exp, is_(true, IsType::True, lit_b(true)));

        Ok(())
    }

    #[test]
    fn is_not_false() -> Result<(), ParserError> {
        let str = "true is not false";
        let exp = parse_expression(str)?;

        assert_eq!(exp, is_(true, IsType::False, lit_b(true)));

        Ok(())
    }

    #[test]
    fn is_none() -> Result<(), ParserError> {
        let str = "true is none";
        let exp = parse_expression(str)?;

        assert_eq!(exp, is_(false, IsType::None, lit_b(true)));

        Ok(())
    }

    #[test]
    fn test_between() -> Result<(), ParserError> {
        let str = "20 between 10 and 50";
        let exp = parse_expression(str)?;

        assert_eq!(exp, between(lit_u64(20), lit_u64(10), lit_u64(50)));

        Ok(())
    }

    #[test]
    fn test_position() -> Result<(), ParserError> {
        let str = "position(\"str\" in \"string\")";
        let exp = parse_expression(str)?;

        assert_eq!(exp, position(lit_s("str"), lit_s("string")));

        Ok(())
    }

    #[test]
    fn test_substring_from_for() -> Result<(), ParserError> {
        let str = "substr(\"str\" from 5 for 4)";
        let exp = parse_expression(str)?;

        assert_eq!(exp, sub_string("str", Some(5), Some(4)));

        Ok(())
    }

    #[test]
    fn test_substring_for_from() -> Result<(), ParserError> {
        let str = "substr(\"str\" for 5 from 4)";
        let exp = parse_expression(str)?;

        assert_eq!(exp, sub_string("str", Some(4), Some(5)));

        Ok(())
    }

    #[test]
    fn test_substring_for_only() -> Result<(), ParserError> {
        let str = "substr(\"str\" for 5)";
        let exp = parse_expression(str)?;

        assert_eq!(exp, sub_string("str", None, Some(5)));

        Ok(())
    }

    #[test]
    fn test_substring_from_only() -> Result<(), ParserError> {
        let str = "substr(\"str\" from 5)";
        let exp = parse_expression(str)?;

        assert_eq!(exp, sub_string("str", Some(5), None));

        Ok(())
    }

    #[test]
    fn test_substring_no_args() {
        let str = "substr(\"str\")";
        let err = parse_expression(str).is_err();

        assert!(err);
    }

    #[test]
    fn test_function_no_args() -> Result<(), ParserError> {
        let str = "random()";
        let exp = parse_expression(str)?;

        assert_eq!(exp, func(FunctionName::Env(EnvFunctionName::Rand), vec![]));

        Ok(())
    }

    #[test]
    fn test_function_single_arg() -> Result<(), ParserError> {
        let str = "trim(\"text\")";
        let exp = parse_expression(str)?;

        assert_eq!(
            exp,
            func(
                FunctionName::String(StringFunctionName::Trim),
                vec![lit_s("text")]
            )
        );

        Ok(())
    }

    #[test]
    fn test_function_multiple_arg() -> Result<(), ParserError> {
        let str = "COALESCE(1, 2, 3, 4, 5)";
        let exp = parse_expression(str)?;

        assert_eq!(
            exp,
            func(
                FunctionName::Env(EnvFunctionName::Coalesce),
                vec![lit_u64(1), lit_u64(2), lit_u64(3), lit_u64(4), lit_u64(5)]
            )
        );

        Ok(())
    }

    #[test]
    fn test_function_multiple_arg_trailing_comma() -> Result<(), ParserError> {
        let str = "COALESCE(1, 2, 3, 4,)";
        let exp = parse_expression(str)?;

        assert_eq!(
            exp,
            func(
                FunctionName::Env(EnvFunctionName::Coalesce),
                vec![lit_u64(1), lit_u64(2), lit_u64(3), lit_u64(4)]
            )
        );

        Ok(())
    }

    #[test]
    fn test_spawn_into() -> Result<(), ParserError> {
        let str = "SPAWN(\"/bin/bash\" INTO \"/dev/null\")";
        let exp = parse_expression(str)?;

        assert_eq!(
            exp,
            spawn(lit_s("/bin/bash"), vec![], Some(lit_s("/dev/null")),)
        );

        Ok(())
    }

    #[test]
    fn test_spawn_multiple_arguments() -> Result<(), ParserError> {
        let str = "SPAWN(\"echo\", 1, 2, 3)";
        let exp = parse_expression(str)?;

        assert_eq!(
            exp,
            spawn(
                lit_s("echo"),
                vec![lit_u64(1), lit_u64(2), lit_u64(3)],
                None
            )
        );

        Ok(())
    }

    #[test]
    fn test_exec_single_arg() -> Result<(), ParserError> {
        let str = "execute(\"echo\")";
        let exp = parse_expression(str)?;

        assert_eq!(exp, exec(lit_s("echo"), vec![], None),);

        Ok(())
    }

    #[test]
    fn test_exec_multiple_arguments_trailing_comma() -> Result<(), ParserError> {
        let str = "exec(\"echo\", 1, 2)";
        let exp = parse_expression(str)?;

        assert_eq!(exp, exec(lit_s("echo"), vec![lit_u64(1), lit_u64(2)], None));

        Ok(())
    }

    #[test]
    fn test_exec_multiple_arguments_trailing_comma_into() -> Result<(), ParserError> {
        let str = "exec(\"echo\", 1, 2, into \"test\")";
        let exp = parse_expression(str)?;

        assert_eq!(
            exp,
            exec(
                lit_s("echo"),
                vec![lit_u64(1), lit_u64(2)],
                Some(lit_s("test"))
            )
        );

        Ok(())
    }

    #[test]
    fn test_spawn_multiple_arguments_into() -> Result<(), ParserError> {
        let str = "spawn(\"echo\", 1, 2, 4 into \"test\")";
        let exp = parse_expression(str)?;

        assert_eq!(
            exp,
            spawn(
                lit_s("echo"),
                vec![lit_u64(1), lit_u64(2), lit_u64(4)],
                Some(lit_s("test"))
            )
        );

        Ok(())
    }
}
