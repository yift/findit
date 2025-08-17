use std::{fs, os::unix::fs::MetadataExt};

use sqlparser::ast::Ident;
use std::os::unix::fs::PermissionsExt;
use uzers::{get_group_by_gid, get_user_by_uid};

use crate::{
    errors::FindItError,
    expr::Evaluator,
    file_wrapper::FileWrapper,
    value::{Value, ValueType},
};

pub(crate) fn get_extractor(name: &Ident) -> Result<Box<dyn Evaluator>, FindItError> {
    let name = &name.value;
    if name.is_empty() {
        return Err(FindItError::BadExpression("Empty identifier".into()));
    }
    if let Some(name) = name.strip_prefix("#") {
        let name = name.to_string();
        if name.is_empty() {
            return Err(FindItError::BadExpression("Empty file name".into()));
        }
        return Ok(Box::new(FileExtractor { name }));
    }
    let name = name.to_lowercase();
    match name.as_str() {
        "parent" => Ok(Box::new(ParentExtractor {})),

        "name" => Ok(Box::new(NameExtractor {})),
        "path" => Ok(Box::new(PathExtractor {})),
        "extension" => Ok(Box::new(ExtensionExtractor {})),
        "absolute" => Ok(Box::new(AbsoluteExtractor {})),

        "content" => Ok(Box::new(ContentExtractor {})),
        "length" => Ok(Box::new(LengthExtractor {})),
        "depth" => Ok(Box::new(DepthExtractor {})),

        "size" => Ok(Box::new(SizeExtractor {})),
        "count" => Ok(Box::new(CountExtractor {})),
        "created" => Ok(Box::new(CreatedExtractor {})),
        "modified" => Ok(Box::new(ModifiedExtractor {})),
        "is_exists" => Ok(Box::new(ExistsExtractor {})),
        "is_dir" => Ok(Box::new(IsDirExtractor {})),
        "is_file" => Ok(Box::new(IsFileExtractor {})),
        "is_link" => Ok(Box::new(IsLinkExtractor {})),

        "owner" => Ok(Box::new(OwnerExtractor {})),
        "group" => Ok(Box::new(GroupExtractor {})),
        "readable" => Ok(Box::new(ReadableExtractor {})),
        "executable" => Ok(Box::new(ExecutableExtractor {})),
        "writeable" => Ok(Box::new(WriteableExtractor {})),
        "hidden" => Ok(Box::new(HiddenExtractor {})),

        _ => Err(FindItError::BadExpression(format!(
            "Unknown identifier: {name}",
        ))),
    }
}

struct FileExtractor {
    name: String,
}

impl Evaluator for FileExtractor {
    fn eval(&self, file: &FileWrapper) -> Value {
        file.path().join(&self.name).into()
    }
    fn expected_type(&self) -> ValueType {
        ValueType::Path
    }
}

struct ParentExtractor {}
impl Evaluator for ParentExtractor {
    fn eval(&self, file: &FileWrapper) -> Value {
        file.path().parent().into()
    }
    fn expected_type(&self) -> ValueType {
        ValueType::Path
    }
}

struct NameExtractor {}
impl Evaluator for NameExtractor {
    fn eval(&self, file: &FileWrapper) -> Value {
        file.path().file_name().into()
    }
    fn expected_type(&self) -> ValueType {
        ValueType::String
    }
}

struct PathExtractor {}
impl Evaluator for PathExtractor {
    fn eval(&self, file: &FileWrapper) -> Value {
        file.path().as_os_str().into()
    }
    fn expected_type(&self) -> ValueType {
        ValueType::String
    }
}
struct ExtensionExtractor {}
impl Evaluator for ExtensionExtractor {
    fn eval(&self, file: &FileWrapper) -> Value {
        file.path().extension().into()
    }
    fn expected_type(&self) -> ValueType {
        ValueType::String
    }
}
struct AbsoluteExtractor {}
impl Evaluator for AbsoluteExtractor {
    fn eval(&self, file: &FileWrapper) -> Value {
        fs::canonicalize(file.path()).into()
    }
    fn expected_type(&self) -> ValueType {
        ValueType::Path
    }
}

struct ContentExtractor {}
impl Evaluator for ContentExtractor {
    fn eval(&self, file: &FileWrapper) -> Value {
        file.read().into()
    }
    fn expected_type(&self) -> ValueType {
        ValueType::String
    }
}

struct LengthExtractor {}
impl Evaluator for LengthExtractor {
    fn eval(&self, file: &FileWrapper) -> Value {
        file.read().map(|f| f.chars().count()).into()
    }
    fn expected_type(&self) -> ValueType {
        ValueType::Number
    }
}

struct DepthExtractor {}
impl Evaluator for DepthExtractor {
    fn eval(&self, file: &FileWrapper) -> Value {
        file.dept().into()
    }
    fn expected_type(&self) -> ValueType {
        ValueType::Number
    }
}

struct SizeExtractor {}
impl Evaluator for SizeExtractor {
    fn eval(&self, file: &FileWrapper) -> Value {
        file.path().metadata().map(|m| m.len()).into()
    }
    fn expected_type(&self) -> ValueType {
        ValueType::Number
    }
}

struct CountExtractor {}
impl Evaluator for CountExtractor {
    fn eval(&self, file: &FileWrapper) -> Value {
        file.count().into()
    }
    fn expected_type(&self) -> ValueType {
        ValueType::Number
    }
}
struct CreatedExtractor {}
impl Evaluator for CreatedExtractor {
    fn eval(&self, file: &FileWrapper) -> Value {
        file.path().metadata().and_then(|m| m.created()).into()
    }
    fn expected_type(&self) -> ValueType {
        ValueType::Date
    }
}
struct ModifiedExtractor {}
impl Evaluator for ModifiedExtractor {
    fn eval(&self, file: &FileWrapper) -> Value {
        file.path().metadata().and_then(|m| m.modified()).into()
    }
    fn expected_type(&self) -> ValueType {
        ValueType::Date
    }
}
struct ExistsExtractor {}
impl Evaluator for ExistsExtractor {
    fn eval(&self, file: &FileWrapper) -> Value {
        file.path().exists().into()
    }
    fn expected_type(&self) -> ValueType {
        ValueType::Bool
    }
}
struct IsDirExtractor {}
impl Evaluator for IsDirExtractor {
    fn eval(&self, file: &FileWrapper) -> Value {
        file.path().is_dir().into()
    }
    fn expected_type(&self) -> ValueType {
        ValueType::Bool
    }
}
struct IsFileExtractor {}
impl Evaluator for IsFileExtractor {
    fn eval(&self, file: &FileWrapper) -> Value {
        file.path().is_file().into()
    }
    fn expected_type(&self) -> ValueType {
        ValueType::Bool
    }
}
struct IsLinkExtractor {}
impl Evaluator for IsLinkExtractor {
    fn eval(&self, file: &FileWrapper) -> Value {
        file.path().is_symlink().into()
    }
    fn expected_type(&self) -> ValueType {
        ValueType::Bool
    }
}

struct OwnerExtractor {}
impl Evaluator for OwnerExtractor {
    fn eval(&self, file: &FileWrapper) -> Value {
        let Ok(m) = fs::metadata(file.path()) else {
            return Value::Empty;
        };
        match get_user_by_uid(m.uid()) {
            None => Value::Empty,
            Some(u) => u.name().into(),
        }
    }
    fn expected_type(&self) -> ValueType {
        ValueType::String
    }
}
struct GroupExtractor {}
impl Evaluator for GroupExtractor {
    fn eval(&self, file: &FileWrapper) -> Value {
        let Ok(m) = fs::metadata(file.path()) else {
            return Value::Empty;
        };
        match get_group_by_gid(m.gid()) {
            None => Value::Empty,
            Some(u) => u.name().into(),
        }
    }
    fn expected_type(&self) -> ValueType {
        ValueType::String
    }
}

struct ExecutableExtractor {}
impl Evaluator for ExecutableExtractor {
    fn eval(&self, file: &FileWrapper) -> Value {
        file.path()
            .metadata()
            .map(|m| m.permissions().mode() & 0o111 != 0)
            .into()
    }
    fn expected_type(&self) -> ValueType {
        ValueType::Bool
    }
}
struct ReadableExtractor {}
impl Evaluator for ReadableExtractor {
    fn eval(&self, file: &FileWrapper) -> Value {
        file.path()
            .metadata()
            .map(|m| m.permissions().mode() & 0o444 != 0)
            .into()
    }
    fn expected_type(&self) -> ValueType {
        ValueType::Bool
    }
}
struct WriteableExtractor {}
impl Evaluator for WriteableExtractor {
    fn eval(&self, file: &FileWrapper) -> Value {
        file.path()
            .metadata()
            .map(|m| m.permissions().mode() & 0o222 != 0)
            .into()
    }
    fn expected_type(&self) -> ValueType {
        ValueType::Bool
    }
}

struct HiddenExtractor {}
impl Evaluator for HiddenExtractor {
    fn eval(&self, file: &FileWrapper) -> Value {
        let name: Value = file.path().file_name().into();
        name.to_string().starts_with('.').into()
    }
    fn expected_type(&self) -> ValueType {
        ValueType::Bool
    }
}
