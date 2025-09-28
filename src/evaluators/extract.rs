use std::{fs, os::unix::fs::MetadataExt};

use std::os::unix::fs::PermissionsExt;
use uzers::{get_group_by_gid, get_user_by_uid};

use crate::{
    evaluators::expr::Evaluator,
    file_wrapper::FileWrapper,
    parser::ast::access::Access,
    value::{Value, ValueType},
};

impl From<&Access> for Box<dyn Evaluator> {
    fn from(access: &Access) -> Self {
        match access {
            Access::Parent => Box::new(ParentExtractor {}),

            Access::Name => Box::new(NameExtractor {}),
            Access::Path => Box::new(PathExtractor {}),
            Access::Extension => Box::new(ExtensionExtractor {}),
            Access::Absolute => Box::new(AbsoluteExtractor {}),
            Access::Me => Box::new(MeExtractor {}),

            Access::Content => Box::new(ContentExtractor {}),
            Access::Length => Box::new(LengthExtractor {}),
            Access::Depth => Box::new(DepthExtractor {}),

            Access::Size => Box::new(SizeExtractor {}),
            Access::Count => Box::new(CountExtractor {}),
            Access::Created => Box::new(CreatedExtractor {}),
            Access::Modified => Box::new(ModifiedExtractor {}),
            Access::Exists => Box::new(ExistsExtractor {}),
            Access::IsDir => Box::new(IsDirExtractor { negate: false }),
            Access::IsFile => Box::new(IsFileExtractor { negate: false }),
            Access::IsLink => Box::new(IsLinkExtractor { negate: false }),
            Access::IsNotDir => Box::new(IsDirExtractor { negate: true }),
            Access::IsNotFile => Box::new(IsFileExtractor { negate: true }),
            Access::IsNotLink => Box::new(IsLinkExtractor { negate: true }),

            Access::Owner => Box::new(OwnerExtractor {}),
            Access::Group => Box::new(GroupExtractor {}),
            Access::Permissions => Box::new(PermissionsExtractor {}),
        }
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
struct MeExtractor {}
impl Evaluator for MeExtractor {
    fn eval(&self, file: &FileWrapper) -> Value {
        file.path().as_path().into()
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
struct IsDirExtractor {
    negate: bool,
}
impl Evaluator for IsDirExtractor {
    fn eval(&self, file: &FileWrapper) -> Value {
        (self.negate ^ file.path().is_dir()).into()
    }
    fn expected_type(&self) -> ValueType {
        ValueType::Bool
    }
}
struct IsFileExtractor {
    negate: bool,
}
impl Evaluator for IsFileExtractor {
    fn eval(&self, file: &FileWrapper) -> Value {
        (self.negate ^ file.path().is_file()).into()
    }
    fn expected_type(&self) -> ValueType {
        ValueType::Bool
    }
}
struct IsLinkExtractor {
    negate: bool,
}
impl Evaluator for IsLinkExtractor {
    fn eval(&self, file: &FileWrapper) -> Value {
        (self.negate ^ file.path().is_symlink()).into()
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

struct PermissionsExtractor {}
impl Evaluator for PermissionsExtractor {
    fn eval(&self, file: &FileWrapper) -> Value {
        file.path()
            .metadata()
            .map(|m| m.permissions().mode())
            .into()
    }
    fn expected_type(&self) -> ValueType {
        ValueType::Number
    }
}

#[cfg(test)]
mod tests {
    use std::{env, path::Path};

    use crate::{errors::FindItError, evaluators::expr::read_expr};

    use super::*;

    #[test]
    fn test_file_creation() -> Result<(), FindItError> {
        let file = env::current_dir()?;
        let creation = file.metadata()?.created()?;
        let exe: Box<dyn Evaluator> = (&Access::Created).into();
        let wrapper = FileWrapper::new(file, 1);

        let value = exe.eval(&wrapper);

        assert_eq!(value, creation.into());

        Ok(())
    }

    fn test_expected_type(name: &str, expected: ValueType) -> Result<(), FindItError> {
        let expr = read_expr(name)?;
        let tp = expr.expected_type();

        assert_eq!(tp, expected);

        Ok(())
    }
    #[test]
    fn test_self_expected_type() -> Result<(), FindItError> {
        test_expected_type("self", ValueType::Path)
    }

    #[test]
    fn test_path_expected_type() -> Result<(), FindItError> {
        test_expected_type("path", ValueType::String)
    }

    #[test]
    fn test_extension_expected_type() -> Result<(), FindItError> {
        test_expected_type("extension", ValueType::String)
    }

    #[test]
    fn test_name_expected_type() -> Result<(), FindItError> {
        test_expected_type("name", ValueType::String)
    }

    #[test]
    fn test_absolute_expected_type() -> Result<(), FindItError> {
        test_expected_type("absolute", ValueType::Path)
    }

    #[test]
    fn test_content_expected_type() -> Result<(), FindItError> {
        test_expected_type("content", ValueType::String)
    }

    #[test]
    fn test_depth_expected_type() -> Result<(), FindItError> {
        test_expected_type("depth", ValueType::Number)
    }

    #[test]
    fn test_size_expected_type() -> Result<(), FindItError> {
        test_expected_type("size", ValueType::Number)
    }

    #[test]
    fn test_count_expected_type() -> Result<(), FindItError> {
        test_expected_type("count", ValueType::Number)
    }

    #[test]
    fn test_length_expected_type() -> Result<(), FindItError> {
        test_expected_type("length", ValueType::Number)
    }

    #[test]
    fn test_created_expected_type() -> Result<(), FindItError> {
        test_expected_type("created", ValueType::Date)
    }

    #[test]
    fn test_modified_expected_type() -> Result<(), FindItError> {
        test_expected_type("modified", ValueType::Date)
    }

    #[test]
    fn test_exists_expected_type() -> Result<(), FindItError> {
        test_expected_type("exists", ValueType::Bool)
    }

    #[test]
    fn test_is_dir_expected_type() -> Result<(), FindItError> {
        test_expected_type("is dir", ValueType::Bool)
    }

    #[test]
    fn test_is_file_expected_type() -> Result<(), FindItError> {
        test_expected_type("is file", ValueType::Bool)
    }

    #[test]
    fn test_is_link_expected_type() -> Result<(), FindItError> {
        test_expected_type("is link", ValueType::Bool)
    }

    #[test]
    fn test_owner_expected_type() -> Result<(), FindItError> {
        test_expected_type("owner", ValueType::String)
    }

    #[test]
    fn test_group_expected_type() -> Result<(), FindItError> {
        test_expected_type("group", ValueType::String)
    }

    #[test]
    fn test_permissions_expected_type() -> Result<(), FindItError> {
        test_expected_type("permissions", ValueType::Number)
    }

    #[test]
    fn test_is_not_dir_expected_type() -> Result<(), FindItError> {
        test_expected_type("is not dir", ValueType::Bool)
    }

    #[test]
    fn test_is_not_file_expected_type() -> Result<(), FindItError> {
        test_expected_type("is not file", ValueType::Bool)
    }

    #[test]
    fn test_is_not_link_expected_type() -> Result<(), FindItError> {
        test_expected_type("is not link", ValueType::Bool)
    }

    #[test]
    fn test_me() -> Result<(), FindItError> {
        let expr = read_expr("me")?;

        let file = env::current_dir()?;
        let wrapper = FileWrapper::new(file.clone(), 1);

        let value = expr.eval(&wrapper);

        assert_eq!(value, Value::Path(file));

        Ok(())
    }

    #[test]
    fn test_owner_with_no_file() -> Result<(), FindItError> {
        let expr = read_expr("owner")?;

        let file = Path::new("/no/such/file");
        let wrapper = FileWrapper::new(file.to_path_buf(), 1);

        let value = expr.eval(&wrapper);

        assert_eq!(value, Value::Empty);

        Ok(())
    }

    #[test]
    fn test_group_with_no_file() -> Result<(), FindItError> {
        let expr = read_expr("group")?;

        let file = Path::new("/no/such/file");
        let wrapper = FileWrapper::new(file.to_path_buf(), 1);

        let value = expr.eval(&wrapper);

        assert_eq!(value, Value::Empty);

        Ok(())
    }
}
