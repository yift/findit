use std::{fmt::Display, rc::Rc};

use ordermap::OrderMap;

use crate::{
    errors::FindItError,
    value::{Value, ValueType},
};

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Hash)]
pub(crate) struct ClassType {
    details: Rc<OrderMap<String, ValueType>>,
}

impl ClassType {
    pub(crate) fn new(fields: &[(String, ValueType)]) -> Self {
        let mut names = OrderMap::new();
        for (name, value) in fields {
            names.insert(name.clone(), value.clone());
        }

        Self {
            details: Rc::new(names),
        }
    }
    pub(crate) fn get_index_and_type(&self, name: &str) -> Result<(usize, ValueType), FindItError> {
        self.details
            .get_full(name)
            .ok_or(FindItError::NoSuchField(name.into()))
            .map(|(index, _, value)| (index, value.clone()))
    }
}
impl Display for ClassType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        "class<".fmt(f)?;
        for (name, tp) in self.details.iter() {
            name.fmt(f)?;
            ":".fmt(f)?;
            tp.fmt(f)?;
            ";".fmt(f)?;
        }

        ">".fmt(f)
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Hash)]
pub(crate) struct Class {
    class: Rc<ClassType>,
    details: Rc<Vec<Value>>,
}

impl Class {
    pub fn new(class: &Rc<ClassType>, details: Vec<Value>) -> Self {
        Self {
            class: class.clone(),
            details: Rc::new(details),
        }
    }
    pub(crate) fn is_empty(&self) -> bool {
        self.details.is_empty()
    }
    pub(crate) fn len(&self) -> usize {
        self.details.len()
    }
    pub(crate) fn get(self, index: usize) -> Value {
        self.details.get(index).cloned().unwrap_or(Value::Empty)
    }
}

impl Display for Class {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        "{".fmt(f)?;
        for (index, ((name, _), val)) in self
            .class
            .details
            .iter()
            .zip(self.details.iter())
            .enumerate()
        {
            if index > 0 {
                ", ".fmt(f)?;
            }
            "\"".fmt(f)?;
            name.fmt(f)?;
            "\"".fmt(f)?;
            ":".fmt(f)?;
            val.fmt(f)?;
        }
        "}".fmt(f)
    }
}
#[cfg(test)]
mod tests {
    use std::{rc::Rc, vec};

    use crate::{
        class_type::{Class, ClassType},
        errors::FindItError,
        value::{Value, ValueType},
    };

    #[test]
    fn get_index_and_type() -> Result<(), FindItError> {
        let fields = vec![
            ("one".into(), ValueType::String),
            ("f2".into(), ValueType::Number),
            ("f3".into(), ValueType::Date),
            ("a2".into(), ValueType::Bool),
            ("a0".into(), ValueType::Number),
        ];
        let cls = ClassType::new(&fields);

        assert_eq!(cls.get_index_and_type("one")?, (0, ValueType::String));
        assert_eq!(cls.get_index_and_type("a2")?, (3, ValueType::Bool));
        assert_eq!(cls.get_index_and_type("a0")?, (4, ValueType::Number));

        Ok(())
    }

    #[test]
    fn get_index_and_type_with_err() -> Result<(), FindItError> {
        let fields = vec![
            ("one".into(), ValueType::String),
            ("f2".into(), ValueType::Number),
            ("f3".into(), ValueType::Date),
            ("a2".into(), ValueType::Bool),
            ("a0".into(), ValueType::Number),
        ];
        let cls = ClassType::new(&fields);

        let err = cls.get_index_and_type("no").err();

        assert!(err.is_some());

        Ok(())
    }

    #[test]
    fn type_display() -> Result<(), FindItError> {
        let fields = vec![
            ("one".into(), ValueType::String),
            ("f2".into(), ValueType::Number),
            ("f3".into(), ValueType::Date),
            ("a2".into(), ValueType::Bool),
            ("a0".into(), ValueType::Number),
        ];
        let cls = ClassType::new(&fields);

        assert_eq!(
            format!("{}", cls),
            "class<one:string;f2:number;f3:date;a2:boolean;a0:number;>"
        );

        Ok(())
    }

    #[test]
    fn is_empty_false() -> Result<(), FindItError> {
        let fields = vec![
            ("one".into(), ValueType::String),
            ("a2".into(), ValueType::Bool),
            ("a0".into(), ValueType::Number),
        ];
        let cls = ClassType::new(&fields);
        let details = vec![
            Value::String("test".into()),
            Value::Bool(true),
            Value::Number(1),
        ];
        let inst = Class::new(&Rc::new(cls), details);

        assert!(!inst.is_empty());

        Ok(())
    }

    #[test]
    fn is_empty_true() -> Result<(), FindItError> {
        let fields = vec![];
        let cls = ClassType::new(&fields);
        let details = vec![];
        let inst = Class::new(&Rc::new(cls), details);

        assert!(inst.is_empty());

        Ok(())
    }

    #[test]
    fn len() -> Result<(), FindItError> {
        let fields = vec![
            ("one".into(), ValueType::String),
            ("a2".into(), ValueType::Bool),
            ("a0".into(), ValueType::Number),
        ];
        let cls = ClassType::new(&fields);
        let details = vec![
            Value::String("test".into()),
            Value::Bool(true),
            Value::Number(1),
        ];
        let inst = Class::new(&Rc::new(cls), details);

        assert_eq!(inst.len(), 3);

        Ok(())
    }

    #[test]
    fn get() -> Result<(), FindItError> {
        let fields = vec![
            ("one".into(), ValueType::String),
            ("a2".into(), ValueType::Bool),
            ("a0".into(), ValueType::Number),
        ];
        let cls = ClassType::new(&fields);
        let details = vec![
            Value::String("test".into()),
            Value::Bool(true),
            Value::Number(1),
        ];
        let inst = Class::new(&Rc::new(cls), details);

        assert_eq!(inst.get(1), Value::Bool(true));

        Ok(())
    }
    #[test]
    fn display() -> Result<(), FindItError> {
        let fields = vec![
            ("one".into(), ValueType::String),
            ("a2".into(), ValueType::Bool),
            ("a0".into(), ValueType::Number),
        ];
        let cls = ClassType::new(&fields);
        let details = vec![
            Value::String("test".into()),
            Value::Bool(true),
            Value::Number(1),
        ];
        let inst = Class::new(&Rc::new(cls), details);

        assert_eq!(format!("{}", inst), "{\"one\":test, \"a2\":true, \"a0\":1}");

        Ok(())
    }
}
