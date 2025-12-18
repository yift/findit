use std::rc::Rc;

use crate::{
    errors::FindItError,
    evaluators::expr::{BindingsTypes, Evaluator},
    file_wrapper::FileWrapper,
    parser::ast::methods::LambdaFunction,
    value::{Value, ValueType},
};

struct Debug {
    target: Box<dyn Evaluator>,
    lambda: Rc<Box<dyn Evaluator>>,
}

impl Evaluator for Debug {
    fn expected_type(&self) -> ValueType {
        self.target.expected_type()
    }
    fn eval(&self, file: &FileWrapper) -> Value {
        let value = self.target.eval(file);
        file.debugger().log(&|| {
            let lambda = self.lambda.clone();
            let value = value.clone();
            let new_file = file.with_binding(value);
            lambda.eval(&new_file).to_string()
        });
        value
    }
}

pub(super) fn new_debug(
    target: Box<dyn Evaluator>,
    lambda: &LambdaFunction,
    bindings: &BindingsTypes,
) -> Result<Box<dyn Evaluator>, FindItError> {
    let lambda = lambda.build(bindings, &target.expected_type())?;
    let lambda = Rc::new(lambda);

    Ok(Box::new(Debug { target, lambda }))
}

#[cfg(test)]
mod tests {
    use std::{fmt::Debug, path::PathBuf, rc::Rc};

    use crate::{
        debugger::Debugger,
        errors::FindItError,
        evaluators::expr::read_expr,
        file_wrapper::FileWrapper,
        value::{Value, ValueType},
    };

    struct MyDebugger {
        logs: Rc<std::cell::RefCell<Vec<String>>>,
    }
    impl Debugger for MyDebugger {
        fn log(&self, f: &dyn Fn() -> String) {
            self.logs.borrow_mut().push(f());
        }
    }
    impl Debug for MyDebugger {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "MyDebugger")
        }
    }

    #[test]
    fn test_simple_debug() -> Result<(), FindItError> {
        let logs = Rc::new(std::cell::RefCell::new(Vec::new()));
        let debugger: Rc<Box<dyn Debugger>> = Rc::new(Box::new(MyDebugger { logs: logs.clone() }));
        let expr = read_expr("100.debug($x \"ten is: \" + $x)")?;
        let file = FileWrapper::new_with_debugger(PathBuf::new(), 1, &debugger);

        assert_eq!(expr.eval(&file), Value::Number(100));
        let logs = logs.borrow();
        assert_eq!(logs.len(), 1);
        assert_eq!(logs[0], "ten is: 100");

        Ok(())
    }

    #[test]
    fn debug_return_value() -> Result<(), FindItError> {
        let expr = read_expr("100.debug($x \"ten is: \" + $x)")?;

        assert_eq!(expr.expected_type(), ValueType::Number);

        Ok(())
    }
}
