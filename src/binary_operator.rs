use chrono::TimeDelta;

use crate::{
    errors::FindItError,
    expr::{Evaluator, get_eval},
    file_wrapper::FileWrapper,
    parser::{
        binary_expression::BinaryExpression,
        operator::{
            ArithmeticOperator, BinaryOperator, BitwiseOperator, ComparisonOperator,
            LogicalOperator,
        },
    },
    string_functions::new_regex,
    value::{Value, ValueType},
};

impl TryFrom<&BinaryExpression> for Box<dyn Evaluator> {
    type Error = FindItError;
    fn try_from(operator: &BinaryExpression) -> Result<Self, Self::Error> {
        let left = get_eval(&operator.left)?;
        let right = get_eval(&operator.right)?;

        match operator.operator {
            BinaryOperator::Arithmetic(operator) => new_arithmetic_operator(left, &operator, right),
            BinaryOperator::Logical(operator) => new_logical_operator(left, &operator, right),
            BinaryOperator::Comparison(operator) => new_comparison_operator(left, &operator, right),
            BinaryOperator::Matches => new_regex(left, right),
            BinaryOperator::Of => new_of(left, right),
            BinaryOperator::Dot => new_of(right, left),
            BinaryOperator::BitwiseOperator(operator) => {
                new_bitwise_operator(left, &operator, right)
            }
        }
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
        ArithmeticOperator::Module => match (left.expected_type(), right.expected_type()) {
            (ValueType::Number, ValueType::Number) => Ok(Box::new(ModuloNumbers { left, right })),
            _ => Err(FindItError::BadExpression(
                "Operator % only support two numbers".into(),
            )),
        },
    }
}

fn new_comparison_operator(
    left: Box<dyn Evaluator>,
    operator: &ComparisonOperator,
    right: Box<dyn Evaluator>,
) -> Result<Box<dyn Evaluator>, FindItError> {
    if left.expected_type() != right.expected_type() {
        return Err(FindItError::BadExpression(format!(
            "Cannot compare two different value types, left type is: {} while right type is {}",
            left.expected_type(),
            right.expected_type()
        )));
    }
    match operator {
        ComparisonOperator::LargerThen => Ok(Box::new(Gt { left, right })),
        ComparisonOperator::SmallerThen => Ok(Box::new(Lt { left, right })),
        ComparisonOperator::LargerThenEq => Ok(Box::new(GtEq { left, right })),
        ComparisonOperator::SmallerThenEq => Ok(Box::new(LtEq { left, right })),
        ComparisonOperator::Eq => Ok(Box::new(Eq { left, right })),
        ComparisonOperator::Neq => Ok(Box::new(NEq { left, right })),
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

fn new_bitwise_operator(
    left: Box<dyn Evaluator>,
    operator: &BitwiseOperator,
    right: Box<dyn Evaluator>,
) -> Result<Box<dyn Evaluator>, FindItError> {
    if (left.expected_type(), right.expected_type()) != (ValueType::Number, ValueType::Number) {
        return Err(FindItError::BadExpression(
            "Cannot use bitwise operation on non numeric values".into(),
        ));
    }
    match operator {
        BitwiseOperator::And => Ok(Box::new(BitwiseAnd { left, right })),
        BitwiseOperator::Or => Ok(Box::new(BitwiseOr { left, right })),
        BitwiseOperator::Xor => Ok(Box::new(BitwiseXor { left, right })),
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

struct BitwiseOr {
    left: Box<dyn Evaluator>,
    right: Box<dyn Evaluator>,
}
impl Evaluator for BitwiseOr {
    fn eval(&self, file: &FileWrapper) -> Value {
        let (Value::Number(left), Value::Number(right)) =
            (self.left.eval(file), self.right.eval(file))
        else {
            return Value::Empty;
        };
        Value::Number(left | right)
    }
    fn expected_type(&self) -> ValueType {
        ValueType::Number
    }
}

struct BitwiseAnd {
    left: Box<dyn Evaluator>,
    right: Box<dyn Evaluator>,
}
impl Evaluator for BitwiseAnd {
    fn eval(&self, file: &FileWrapper) -> Value {
        let (Value::Number(left), Value::Number(right)) =
            (self.left.eval(file), self.right.eval(file))
        else {
            return Value::Empty;
        };
        Value::Number(left & right)
    }
    fn expected_type(&self) -> ValueType {
        ValueType::Number
    }
}

struct BitwiseXor {
    left: Box<dyn Evaluator>,
    right: Box<dyn Evaluator>,
}
impl Evaluator for BitwiseXor {
    fn eval(&self, file: &FileWrapper) -> Value {
        let (Value::Number(left), Value::Number(right)) =
            (self.left.eval(file), self.right.eval(file))
        else {
            return Value::Empty;
        };
        Value::Number(left ^ right)
    }
    fn expected_type(&self) -> ValueType {
        ValueType::Number
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

struct Of {
    access: Box<dyn Evaluator>,
    of: Box<dyn Evaluator>,
}
fn new_of(
    access: Box<dyn Evaluator>,
    of: Box<dyn Evaluator>,
) -> Result<Box<dyn Evaluator>, FindItError> {
    if of.expected_type() != ValueType::Path {
        return Err(FindItError::BadExpression("of must refer to a path".into()));
    }
    Ok(Box::new(Of { access, of }))
}

impl Evaluator for Of {
    fn expected_type(&self) -> ValueType {
        self.access.expected_type()
    }
    fn eval(&self, file: &FileWrapper) -> Value {
        let Value::Path(path) = self.of.eval(file) else {
            return Value::Empty;
        };
        let wrapper = FileWrapper::new(path, file.dept() + 1);
        self.access.eval(&wrapper)
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use crate::{
        errors::FindItError,
        expr::read_expr,
        file_wrapper::FileWrapper,
        value::{Value, ValueType},
    };

    #[test]
    fn unsupported_binary_operator() {
        let err = read_expr("1  <=> 1").err();
        assert!(err.is_some())
    }

    #[test]
    fn unsupported_plus_files() {
        let err = read_expr("parent + parent").err();
        assert!(err.is_some())
    }

    #[test]
    fn unsupported_minus_files() {
        let err = read_expr("parent - parent").err();
        assert!(err.is_some())
    }

    #[test]
    fn unsupported_multiply_string() {
        let err = read_expr("\"a\" * 3").err();
        assert!(err.is_some())
    }

    #[test]
    fn unsupported_divide_string() {
        let err = read_expr("\"a\" / 3").err();
        assert!(err.is_some())
    }

    #[test]
    fn unsupported_modulo_string() {
        let err = read_expr("\"a\" % 3").err();
        assert!(err.is_some())
    }

    #[test]
    fn unsupported_concat_numbers() {
        let err = read_expr("4 || 3").err();
        assert!(err.is_some())
    }

    #[test]
    fn unsupported_compare_different_type() {
        let err = read_expr("4 > \"four\"").err();
        assert!(err.is_some())
    }

    #[test]
    fn unsupported_logical_numbers() {
        let err = read_expr("4 AND TRUE").err();
        assert!(err.is_some())
    }

    #[test]
    fn plus_return_empty_if_not_a_number() {
        let eval = read_expr("parent.length + 200").unwrap();
        let path = Path::new("no/such/file");
        let wrapper = FileWrapper::new(path.to_path_buf(), 2);
        let value = eval.eval(&wrapper);
        assert_eq!(value, Value::Empty)
    }

    #[test]
    fn plus_date_return_empty_if_not_a_number() {
        let eval = read_expr("[2025-04-19 08:42:00] + parent.length").unwrap();
        let path = Path::new("no/such/file");
        let wrapper = FileWrapper::new(path.to_path_buf(), 2);
        let value = eval.eval(&wrapper);
        assert_eq!(value, Value::Empty)
    }

    #[test]
    fn plus_date_return_empty_if_number_is_too_large() {
        let eval = read_expr("[2025-04-19 08:42:00] + 18446744073709551613").unwrap();
        let path = Path::new("no/such/file");
        let wrapper = FileWrapper::new(path.to_path_buf(), 2);
        let value = eval.eval(&wrapper);
        assert_eq!(value, Value::Empty)
    }

    #[test]
    fn minus_return_empty_if_not_a_number() {
        let eval = read_expr("parent.length - 200").unwrap();
        let path = Path::new("no/such/file");
        let wrapper = FileWrapper::new(path.to_path_buf(), 2);
        let value = eval.eval(&wrapper);
        assert_eq!(value, Value::Empty)
    }

    #[test]
    fn minus_date_return_empty_if_not_a_number() {
        let eval = read_expr("[2025-04-19 08:42:00] - parent.length").unwrap();
        let path = Path::new("no/such/file");
        let wrapper = FileWrapper::new(path.to_path_buf(), 2);
        let value = eval.eval(&wrapper);
        assert_eq!(value, Value::Empty)
    }

    #[test]
    fn minus_date_return_empty_if_number_is_too_large() {
        let eval = read_expr("[2025-04-19 08:42:00] - 18446744073709551613").unwrap();
        let path = Path::new("no/such/file");
        let wrapper = FileWrapper::new(path.to_path_buf(), 2);
        let value = eval.eval(&wrapper);
        assert_eq!(value, Value::Empty)
    }

    #[test]
    fn divide_path_return_nothing_if_no_such_file() {
        let eval = read_expr("parent / parent.content").unwrap();
        let path = Path::new("no/such/file");
        let wrapper = FileWrapper::new(path.to_path_buf(), 2);
        let value = eval.eval(&wrapper);
        assert_eq!(value, Value::Empty)
    }

    #[test]
    fn divide_path_expect_path() {
        let eval = read_expr("parent / parent.name").unwrap();
        assert_eq!(eval.expected_type(), ValueType::Path)
    }

    #[test]
    fn gt_return_empty_for_empty_right() {
        let eval = read_expr("\"12\" > parent.content").unwrap();
        let path = Path::new("no/such/file");
        let wrapper = FileWrapper::new(path.to_path_buf(), 2);
        let value = eval.eval(&wrapper);
        assert_eq!(value, Value::Empty)
    }

    #[test]
    fn lt_return_empty_for_empty_right() {
        let eval = read_expr("\"12\" < parent.content").unwrap();
        let path = Path::new("no/such/file");
        let wrapper = FileWrapper::new(path.to_path_buf(), 2);
        let value = eval.eval(&wrapper);
        assert_eq!(value, Value::Empty)
    }

    #[test]
    fn lt_return_empty_for_empty_left() {
        let eval = read_expr("parent.content < \"44\"").unwrap();
        let path = Path::new("no/such/file");
        let wrapper = FileWrapper::new(path.to_path_buf(), 2);
        let value = eval.eval(&wrapper);
        assert_eq!(value, Value::Empty)
    }

    #[test]
    fn gte_return_empty_for_empty_right() {
        let eval = read_expr("\"12\" >= parent.content").unwrap();
        let path = Path::new("no/such/file");
        let wrapper = FileWrapper::new(path.to_path_buf(), 2);
        let value = eval.eval(&wrapper);
        assert_eq!(value, Value::Empty)
    }

    #[test]
    fn lte_return_empty_for_empty_right() {
        let eval = read_expr("\"12\" <= parent.content").unwrap();
        let path = Path::new("no/such/file");
        let wrapper = FileWrapper::new(path.to_path_buf(), 2);
        let value = eval.eval(&wrapper);
        assert_eq!(value, Value::Empty)
    }

    #[test]
    fn lte_return_empty_for_empty_left() {
        let eval = read_expr("parent.content <= \"44\"").unwrap();
        let path = Path::new("no/such/file");
        let wrapper = FileWrapper::new(path.to_path_buf(), 2);
        let value = eval.eval(&wrapper);
        assert_eq!(value, Value::Empty)
    }

    #[test]
    fn eq_return_empty_for_empty_right() {
        let eval = read_expr("\"12\" = parent.content").unwrap();
        let path = Path::new("no/such/file");
        let wrapper = FileWrapper::new(path.to_path_buf(), 2);
        let value = eval.eval(&wrapper);
        assert_eq!(value, Value::Empty)
    }

    #[test]
    fn neq_return_empty_for_empty_right() {
        let eval = read_expr("\"12\" <> parent.content").unwrap();
        let path = Path::new("no/such/file");
        let wrapper = FileWrapper::new(path.to_path_buf(), 2);
        let value = eval.eval(&wrapper);
        assert_eq!(value, Value::Empty)
    }

    #[test]
    fn gt_expect_bool() {
        let eval = read_expr("1 > 20").unwrap();
        assert_eq!(eval.expected_type(), ValueType::Bool)
    }

    #[test]
    fn lt_expect_bool() {
        let eval = read_expr("1 < 20").unwrap();
        assert_eq!(eval.expected_type(), ValueType::Bool)
    }

    #[test]
    fn gte_expect_bool() {
        let eval = read_expr("1 >= 20").unwrap();
        assert_eq!(eval.expected_type(), ValueType::Bool)
    }

    #[test]
    fn lte_expect_bool() {
        let eval = read_expr("1 <= 20").unwrap();
        assert_eq!(eval.expected_type(), ValueType::Bool)
    }

    #[test]
    fn eq_expect_bool() {
        let eval = read_expr("1 = 20").unwrap();
        assert_eq!(eval.expected_type(), ValueType::Bool)
    }

    #[test]
    fn neq_expect_bool() {
        let eval = read_expr("1 <> 20").unwrap();
        assert_eq!(eval.expected_type(), ValueType::Bool)
    }

    #[test]
    fn or_return_empty_for_empty_left() {
        let eval = read_expr("\"12\" = parent.content OR TRUE").unwrap();
        let path = Path::new("no/such/file");
        let wrapper = FileWrapper::new(path.to_path_buf(), 2);
        let value = eval.eval(&wrapper);
        assert_eq!(value, Value::Empty)
    }

    #[test]
    fn or_return_empty_for_empty_right() {
        let eval = read_expr("FALSE OR \"12\" = parent.content").unwrap();
        let path = Path::new("no/such/file");
        let wrapper = FileWrapper::new(path.to_path_buf(), 2);
        let value = eval.eval(&wrapper);
        assert_eq!(value, Value::Empty)
    }

    #[test]
    fn or_expect_bool() {
        let eval = read_expr("TRUE OR FALSE").unwrap();
        assert_eq!(eval.expected_type(), ValueType::Bool)
    }

    #[test]
    fn and_return_empty_for_empty_left() {
        let eval = read_expr("(\"12\" = content OF parent) AND TRUE").unwrap();
        let path = Path::new("no/such/file");
        let wrapper = FileWrapper::new(path.to_path_buf(), 2);
        let value = eval.eval(&wrapper);
        assert_eq!(value, Value::Empty)
    }

    #[test]
    fn and_return_empty_for_empty_right() {
        let eval = read_expr("TRUE AND \"12\" = parent.content").unwrap();
        let path = Path::new("no/such/file");
        let wrapper = FileWrapper::new(path.to_path_buf(), 2);
        let value = eval.eval(&wrapper);
        assert_eq!(value, Value::Empty)
    }

    #[test]
    fn and_expect_bool() {
        let eval = read_expr("TRUE AND FALSE").unwrap();
        assert_eq!(eval.expected_type(), ValueType::Bool)
    }

    #[test]
    fn xor_return_empty_for_empty_left() {
        let eval = read_expr("(\"12\" = parent.content) XOR TRUE").unwrap();
        let path = Path::new("no/such/file");
        let wrapper = FileWrapper::new(path.to_path_buf(), 2);
        let value = eval.eval(&wrapper);
        assert_eq!(value, Value::Empty)
    }

    #[test]
    fn xor_return_empty_for_empty_right() {
        let eval = read_expr("TRUE XOR (\"12\" = parent.content)").unwrap();
        let path = Path::new("no/such/file");
        let wrapper = FileWrapper::new(path.to_path_buf(), 2);
        let value = eval.eval(&wrapper);
        assert_eq!(value, Value::Empty)
    }

    #[test]
    fn xor_expect_bool() {
        let eval = read_expr("TRUE XOR FALSE").unwrap();
        assert_eq!(eval.expected_type(), ValueType::Bool)
    }

    #[test]
    fn non_numeric_bitwise_left_operator() {
        let err = read_expr("parent & 10").err();
        assert!(err.is_some())
    }

    #[test]
    fn non_numeric_bitwise_right_operator() {
        let err = read_expr("10 ^ parent").err();
        assert!(err.is_some())
    }

    #[test]
    fn or_bitwise_expected_type() -> Result<(), FindItError> {
        let expr = read_expr("10 | 20")?;

        assert_eq!(expr.expected_type(), ValueType::Number);

        Ok(())
    }

    #[test]
    fn and_bitwise_expected_type() -> Result<(), FindItError> {
        let expr = read_expr("10 & 20")?;

        assert_eq!(expr.expected_type(), ValueType::Number);

        Ok(())
    }

    #[test]
    fn xor_bitwise_expected_type() -> Result<(), FindItError> {
        let expr = read_expr("10 ^ 20")?;

        assert_eq!(expr.expected_type(), ValueType::Number);

        Ok(())
    }

    #[test]
    fn or_bitwise() -> Result<(), FindItError> {
        let expr = read_expr("0x10 | 0x20")?;
        let file = FileWrapper::new(Path::new("/no/such/file").to_path_buf(), 1);

        let val = expr.eval(&file);

        assert_eq!(val, Value::Number(0x30));

        Ok(())
    }

    #[test]
    fn and_bitwise() -> Result<(), FindItError> {
        let expr = read_expr("0x30 & 0x23")?;
        let file = FileWrapper::new(Path::new("/no/such/file").to_path_buf(), 1);

        let val = expr.eval(&file);

        assert_eq!(val, Value::Number(0x20));

        Ok(())
    }

    #[test]
    fn xor_bitwise() -> Result<(), FindItError> {
        let expr = read_expr("0x3 ^ 0x5")?;
        let file = FileWrapper::new(Path::new("/no/such/file").to_path_buf(), 1);

        let val = expr.eval(&file);

        assert_eq!(val, Value::Number(0x6));

        Ok(())
    }

    #[test]
    fn or_bitwise_no_left() -> Result<(), FindItError> {
        let expr = read_expr("length | 0x20")?;
        let file = FileWrapper::new(Path::new("/no/such/file").to_path_buf(), 1);

        let val = expr.eval(&file);

        assert_eq!(val, Value::Empty);

        Ok(())
    }

    #[test]
    fn or_bitwise_no_right() -> Result<(), FindItError> {
        let expr = read_expr("0x20 | length")?;
        let file = FileWrapper::new(Path::new("/no/such/file").to_path_buf(), 1);

        let val = expr.eval(&file);

        assert_eq!(val, Value::Empty);

        Ok(())
    }

    #[test]
    fn and_bitwise_no_left() -> Result<(), FindItError> {
        let expr = read_expr("length & 0x20")?;
        let file = FileWrapper::new(Path::new("/no/such/file").to_path_buf(), 1);

        let val = expr.eval(&file);

        assert_eq!(val, Value::Empty);

        Ok(())
    }

    #[test]
    fn and_bitwise_no_right() -> Result<(), FindItError> {
        let expr = read_expr("0x20 & length")?;
        let file = FileWrapper::new(Path::new("/no/such/file").to_path_buf(), 1);

        let val = expr.eval(&file);

        assert_eq!(val, Value::Empty);

        Ok(())
    }

    #[test]
    fn xor_bitwise_no_left() -> Result<(), FindItError> {
        let expr = read_expr("length ^ 0x20")?;
        let file = FileWrapper::new(Path::new("/no/such/file").to_path_buf(), 1);

        let val = expr.eval(&file);

        assert_eq!(val, Value::Empty);

        Ok(())
    }

    #[test]
    fn xor_bitwise_no_right() -> Result<(), FindItError> {
        let expr = read_expr("0x20 ^ length")?;
        let file = FileWrapper::new(Path::new("/no/such/file").to_path_buf(), 1);

        let val = expr.eval(&file);

        assert_eq!(val, Value::Empty);

        Ok(())
    }

    #[test]
    fn of_of_number() {
        let err = read_expr("parent of 10").err();
        assert!(err.is_some())
    }
}
