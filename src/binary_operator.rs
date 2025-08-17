use chrono::TimeDelta;
use regex::Regex;
use sqlparser::ast::{BinaryOperator, Expr};

use crate::{
    errors::FindItError,
    expr::{Evaluator, get_eval},
    file_wrapper::FileWrapper,
    value::{Value, ValueType},
};

enum ArithmeticOperator {
    Plus,
    Minus,
    Multiply,
    Divide,
    Modulo,
}
enum StringOperator {
    Concat,
    Matches,
}
enum ComparisonOperator {
    Gt,
    Lt,
    Eq,
    NEq,
    GtEq,
    LtEq,
}
enum LogicalOperator {
    And,
    Or,
    Xor,
}
enum SupportedOperators {
    Arithmetic(ArithmeticOperator),
    String(StringOperator),
    Comparison(ComparisonOperator),
    Logical(LogicalOperator),
}
impl TryInto<SupportedOperators> for &BinaryOperator {
    type Error = FindItError;
    fn try_into(self) -> Result<SupportedOperators, Self::Error> {
        match self {
            BinaryOperator::Plus => Ok(SupportedOperators::Arithmetic(ArithmeticOperator::Plus)),
            BinaryOperator::Minus => Ok(SupportedOperators::Arithmetic(ArithmeticOperator::Minus)),
            BinaryOperator::Multiply => {
                Ok(SupportedOperators::Arithmetic(ArithmeticOperator::Multiply))
            }
            BinaryOperator::Divide => {
                Ok(SupportedOperators::Arithmetic(ArithmeticOperator::Divide))
            }
            BinaryOperator::Modulo => {
                Ok(SupportedOperators::Arithmetic(ArithmeticOperator::Modulo))
            }
            BinaryOperator::Gt => Ok(SupportedOperators::Comparison(ComparisonOperator::Gt)),
            BinaryOperator::Lt => Ok(SupportedOperators::Comparison(ComparisonOperator::Lt)),
            BinaryOperator::GtEq => Ok(SupportedOperators::Comparison(ComparisonOperator::GtEq)),
            BinaryOperator::LtEq => Ok(SupportedOperators::Comparison(ComparisonOperator::LtEq)),
            BinaryOperator::Eq => Ok(SupportedOperators::Comparison(ComparisonOperator::Eq)),
            BinaryOperator::NotEq => Ok(SupportedOperators::Comparison(ComparisonOperator::NEq)),
            BinaryOperator::And => Ok(SupportedOperators::Logical(LogicalOperator::And)),
            BinaryOperator::Or => Ok(SupportedOperators::Logical(LogicalOperator::Or)),
            BinaryOperator::Xor => Ok(SupportedOperators::Logical(LogicalOperator::Xor)),
            BinaryOperator::StringConcat => Ok(SupportedOperators::String(StringOperator::Concat)),
            BinaryOperator::Regexp | BinaryOperator::Match => {
                Ok(SupportedOperators::String(StringOperator::Matches))
            }
            _ => Err(FindItError::BadExpression(
                "Operator {operator} is not supported".into(),
            )),
        }
    }
}

pub(crate) fn new_binary_operator(
    left: &Expr,
    operator: &BinaryOperator,
    right: &Expr,
) -> Result<Box<dyn Evaluator>, FindItError> {
    let left = get_eval(left)?;
    let right = get_eval(right)?;
    let operator: SupportedOperators = operator.try_into()?;
    match operator {
        SupportedOperators::Arithmetic(operator) => new_arithmetic_operator(left, &operator, right),
        SupportedOperators::String(operator) => new_string_operator(left, &operator, right),
        SupportedOperators::Comparison(operator) => new_comparison_operator(left, &operator, right),
        SupportedOperators::Logical(operator) => new_logical_operator(left, &operator, right),
    }
}
fn new_arithmetic_operator(
    left: Box<dyn Evaluator>,
    operator: &ArithmeticOperator,
    right: Box<dyn Evaluator>,
) -> Result<Box<dyn Evaluator>, FindItError> {
    match operator {
        ArithmeticOperator::Plus => match (left.expected_type(), right.expected_type()) {
            (ValueType::Number, ValueType::Number) => Ok(Box::new(PlusNumbers { left, right })),
            (ValueType::String, _) => Ok(Box::new(PlusString { left, right })),
            (ValueType::Date, ValueType::Number) => Ok(Box::new(PlusDate { left, right })),
            _ => Err(FindItError::BadExpression(
                "Operator + only support two numbers, string and anything, or date and number"
                    .into(),
            )),
        },
        ArithmeticOperator::Minus => match (left.expected_type(), right.expected_type()) {
            (ValueType::Number, ValueType::Number) => Ok(Box::new(MinusNumbers { left, right })),
            (ValueType::Date, ValueType::Number) => Ok(Box::new(MinusDate { left, right })),
            _ => Err(FindItError::BadExpression(
                "Operator - only support two numbers, or date and number".into(),
            )),
        },
        ArithmeticOperator::Multiply => match (left.expected_type(), right.expected_type()) {
            (ValueType::Number, ValueType::Number) => Ok(Box::new(TimesNumbers { left, right })),
            _ => Err(FindItError::BadExpression(
                "Operator * only support two numbers".into(),
            )),
        },
        ArithmeticOperator::Divide => match (left.expected_type(), right.expected_type()) {
            (ValueType::Number, ValueType::Number) => Ok(Box::new(DivideNumbers { left, right })),
            (ValueType::Path, ValueType::String) => Ok(Box::new(DividePath { left, right })),
            _ => Err(FindItError::BadExpression(
                "Operator / only support two numbers or path and string".into(),
            )),
        },
        ArithmeticOperator::Modulo => match (left.expected_type(), right.expected_type()) {
            (ValueType::Number, ValueType::Number) => Ok(Box::new(ModuloNumbers { left, right })),
            _ => Err(FindItError::BadExpression(
                "Operator % only support two numbers".into(),
            )),
        },
    }
}

fn new_string_operator(
    left: Box<dyn Evaluator>,
    operator: &StringOperator,
    right: Box<dyn Evaluator>,
) -> Result<Box<dyn Evaluator>, FindItError> {
    if (left.expected_type(), right.expected_type()) != (ValueType::String, ValueType::String) {
        return Err(FindItError::BadExpression(
            "Operator only support two string".into(),
        ));
    }
    match operator {
        StringOperator::Matches => Ok(Box::new(Regexp { left, right })),
        StringOperator::Concat => Ok(Box::new(PlusString { left, right })),
    }
}

fn new_comparison_operator(
    left: Box<dyn Evaluator>,
    operator: &ComparisonOperator,
    right: Box<dyn Evaluator>,
) -> Result<Box<dyn Evaluator>, FindItError> {
    if left.expected_type() != right.expected_type() {
        return Err(FindItError::BadExpression(
            "Cannot compare two different value types".into(),
        ));
    }
    match operator {
        ComparisonOperator::Gt => Ok(Box::new(Gt { left, right })),
        ComparisonOperator::Lt => Ok(Box::new(Lt { left, right })),
        ComparisonOperator::GtEq => Ok(Box::new(GtEq { left, right })),
        ComparisonOperator::LtEq => Ok(Box::new(LtEq { left, right })),
        ComparisonOperator::Eq => Ok(Box::new(Eq { left, right })),
        ComparisonOperator::NEq => Ok(Box::new(NEq { left, right })),
    }
}

fn new_logical_operator(
    left: Box<dyn Evaluator>,
    operator: &LogicalOperator,
    right: Box<dyn Evaluator>,
) -> Result<Box<dyn Evaluator>, FindItError> {
    if (left.expected_type(), right.expected_type()) != (ValueType::Bool, ValueType::Bool) {
        return Err(FindItError::BadExpression(
            "Cannot use logical operation on non logical values".into(),
        ));
    }
    match operator {
        LogicalOperator::And => Ok(Box::new(And { left, right })),
        LogicalOperator::Or => Ok(Box::new(Or { left, right })),
        LogicalOperator::Xor => Ok(Box::new(Xor { left, right })),
    }
}

struct PlusNumbers {
    left: Box<dyn Evaluator>,
    right: Box<dyn Evaluator>,
}
impl Evaluator for PlusNumbers {
    fn eval(&self, file: &FileWrapper) -> Value {
        let (Value::Number(left), Value::Number(right)) =
            (self.left.eval(file), self.right.eval(file))
        else {
            return Value::Empty;
        };
        left.checked_add(right).into()
    }
    fn expected_type(&self) -> ValueType {
        ValueType::Number
    }
}
struct PlusString {
    left: Box<dyn Evaluator>,
    right: Box<dyn Evaluator>,
}
impl Evaluator for PlusString {
    fn eval(&self, file: &FileWrapper) -> Value {
        let Value::String(left) = self.left.eval(file) else {
            return Value::Empty;
        };
        let right = self.right.eval(file);
        Value::String(format!("{left}{right}"))
    }
    fn expected_type(&self) -> ValueType {
        ValueType::String
    }
}
struct PlusDate {
    left: Box<dyn Evaluator>,
    right: Box<dyn Evaluator>,
}
impl Evaluator for PlusDate {
    fn eval(&self, file: &FileWrapper) -> Value {
        let (Value::Date(left), Value::Number(right)) =
            (self.left.eval(file), self.right.eval(file))
        else {
            return Value::Empty;
        };
        let Ok(right) = i64::try_from(right) else {
            return Value::Empty;
        };
        let time_delta = TimeDelta::seconds(right);
        left.checked_add_signed(time_delta).into()
    }
    fn expected_type(&self) -> ValueType {
        ValueType::Date
    }
}
struct MinusNumbers {
    left: Box<dyn Evaluator>,
    right: Box<dyn Evaluator>,
}
impl Evaluator for MinusNumbers {
    fn eval(&self, file: &FileWrapper) -> Value {
        let (Value::Number(left), Value::Number(right)) =
            (self.left.eval(file), self.right.eval(file))
        else {
            return Value::Empty;
        };
        left.checked_sub(right).into()
    }
    fn expected_type(&self) -> ValueType {
        ValueType::Number
    }
}

struct MinusDate {
    left: Box<dyn Evaluator>,
    right: Box<dyn Evaluator>,
}
impl Evaluator for MinusDate {
    fn eval(&self, file: &FileWrapper) -> Value {
        let (Value::Date(left), Value::Number(right)) =
            (self.left.eval(file), self.right.eval(file))
        else {
            return Value::Empty;
        };
        let Ok(right) = i64::try_from(right) else {
            return Value::Empty;
        };
        let time_delta = TimeDelta::seconds(right);
        left.checked_sub_signed(time_delta).into()
    }
    fn expected_type(&self) -> ValueType {
        ValueType::Date
    }
}

struct TimesNumbers {
    left: Box<dyn Evaluator>,
    right: Box<dyn Evaluator>,
}
impl Evaluator for TimesNumbers {
    fn eval(&self, file: &FileWrapper) -> Value {
        let (Value::Number(left), Value::Number(right)) =
            (self.left.eval(file), self.right.eval(file))
        else {
            return Value::Empty;
        };
        left.checked_mul(right).into()
    }
    fn expected_type(&self) -> ValueType {
        ValueType::Number
    }
}

struct ModuloNumbers {
    left: Box<dyn Evaluator>,
    right: Box<dyn Evaluator>,
}
impl Evaluator for ModuloNumbers {
    fn eval(&self, file: &FileWrapper) -> Value {
        let (Value::Number(left), Value::Number(right)) =
            (self.left.eval(file), self.right.eval(file))
        else {
            return Value::Empty;
        };
        (left % right).into()
    }
    fn expected_type(&self) -> ValueType {
        ValueType::Number
    }
}

struct DivideNumbers {
    left: Box<dyn Evaluator>,
    right: Box<dyn Evaluator>,
}
impl Evaluator for DivideNumbers {
    fn eval(&self, file: &FileWrapper) -> Value {
        let (Value::Number(left), Value::Number(right)) =
            (self.left.eval(file), self.right.eval(file))
        else {
            return Value::Empty;
        };
        left.checked_div(right).into()
    }
    fn expected_type(&self) -> ValueType {
        ValueType::Number
    }
}

struct DividePath {
    left: Box<dyn Evaluator>,
    right: Box<dyn Evaluator>,
}
impl Evaluator for DividePath {
    fn eval(&self, file: &FileWrapper) -> Value {
        let (Value::Path(left), Value::String(right)) =
            (self.left.eval(file), self.right.eval(file))
        else {
            return Value::Empty;
        };
        left.join(right).into()
    }
    fn expected_type(&self) -> ValueType {
        ValueType::Path
    }
}

struct Gt {
    left: Box<dyn Evaluator>,
    right: Box<dyn Evaluator>,
}
impl Evaluator for Gt {
    fn eval(&self, file: &FileWrapper) -> Value {
        let left = self.left.eval(file);
        if left == Value::Empty {
            return Value::Empty;
        }
        let right = self.right.eval(file);
        if right == Value::Empty {
            return Value::Empty;
        }

        (left > right).into()
    }
    fn expected_type(&self) -> ValueType {
        ValueType::Bool
    }
}
struct Lt {
    left: Box<dyn Evaluator>,
    right: Box<dyn Evaluator>,
}
impl Evaluator for Lt {
    fn eval(&self, file: &FileWrapper) -> Value {
        let left = self.left.eval(file);
        if left == Value::Empty {
            return Value::Empty;
        }
        let right = self.right.eval(file);
        if right == Value::Empty {
            return Value::Empty;
        }

        (left < right).into()
    }
    fn expected_type(&self) -> ValueType {
        ValueType::Bool
    }
}
struct GtEq {
    left: Box<dyn Evaluator>,
    right: Box<dyn Evaluator>,
}
impl Evaluator for GtEq {
    fn eval(&self, file: &FileWrapper) -> Value {
        let left = self.left.eval(file);
        if left == Value::Empty {
            return Value::Empty;
        }
        let right = self.right.eval(file);
        if right == Value::Empty {
            return Value::Empty;
        }

        (left >= right).into()
    }
    fn expected_type(&self) -> ValueType {
        ValueType::Bool
    }
}
struct LtEq {
    left: Box<dyn Evaluator>,
    right: Box<dyn Evaluator>,
}
impl Evaluator for LtEq {
    fn eval(&self, file: &FileWrapper) -> Value {
        let left = self.left.eval(file);
        if left == Value::Empty {
            return Value::Empty;
        }
        let right = self.right.eval(file);
        if right == Value::Empty {
            return Value::Empty;
        }

        (left <= right).into()
    }
    fn expected_type(&self) -> ValueType {
        ValueType::Bool
    }
}
struct Eq {
    left: Box<dyn Evaluator>,
    right: Box<dyn Evaluator>,
}
impl Evaluator for Eq {
    fn eval(&self, file: &FileWrapper) -> Value {
        let left = self.left.eval(file);
        if left == Value::Empty {
            return Value::Empty;
        }
        let right = self.right.eval(file);
        if right == Value::Empty {
            return Value::Empty;
        }

        (left == right).into()
    }
    fn expected_type(&self) -> ValueType {
        ValueType::Bool
    }
}
struct NEq {
    left: Box<dyn Evaluator>,
    right: Box<dyn Evaluator>,
}
impl Evaluator for NEq {
    fn eval(&self, file: &FileWrapper) -> Value {
        let left = self.left.eval(file);
        if left == Value::Empty {
            return Value::Empty;
        }
        let right = self.right.eval(file);
        if right == Value::Empty {
            return Value::Empty;
        }

        (left != right).into()
    }
    fn expected_type(&self) -> ValueType {
        ValueType::Bool
    }
}
struct And {
    left: Box<dyn Evaluator>,
    right: Box<dyn Evaluator>,
}
impl Evaluator for And {
    fn eval(&self, file: &FileWrapper) -> Value {
        let Value::Bool(left) = self.left.eval(file) else {
            return Value::Empty;
        };
        if !left {
            return Value::Bool(false);
        }
        let Value::Bool(right) = self.right.eval(file) else {
            return Value::Empty;
        };
        right.into()
    }
    fn expected_type(&self) -> ValueType {
        ValueType::Bool
    }
}
struct Or {
    left: Box<dyn Evaluator>,
    right: Box<dyn Evaluator>,
}
impl Evaluator for Or {
    fn eval(&self, file: &FileWrapper) -> Value {
        let Value::Bool(left) = self.left.eval(file) else {
            return Value::Empty;
        };
        if left {
            return Value::Bool(true);
        }
        let Value::Bool(right) = self.right.eval(file) else {
            return Value::Empty;
        };
        right.into()
    }
    fn expected_type(&self) -> ValueType {
        ValueType::Bool
    }
}
struct Xor {
    left: Box<dyn Evaluator>,
    right: Box<dyn Evaluator>,
}
impl Evaluator for Xor {
    fn eval(&self, file: &FileWrapper) -> Value {
        let Value::Bool(left) = self.left.eval(file) else {
            return Value::Empty;
        };
        let Value::Bool(right) = self.right.eval(file) else {
            return Value::Empty;
        };
        (left ^ right).into()
    }
    fn expected_type(&self) -> ValueType {
        ValueType::Bool
    }
}

struct Regexp {
    left: Box<dyn Evaluator>,
    right: Box<dyn Evaluator>,
}
impl Evaluator for Regexp {
    fn eval(&self, file: &FileWrapper) -> Value {
        let Value::String(left) = self.left.eval(file) else {
            return Value::Empty;
        };
        let Value::String(right) = self.right.eval(file) else {
            return Value::Empty;
        };
        let Ok(regexp) = Regex::new(&right) else {
            return Value::Empty;
        };
        regexp.is_match(&left).into()
    }
    fn expected_type(&self) -> ValueType {
        ValueType::Bool
    }
}
