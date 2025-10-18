use crate::parser::ast::access::Access;

impl Access {
    pub(super) fn from_str(name: &str) -> Option<Self> {
        match name {
            "PARENT" => Some(Access::Parent),
            "NAME" => Some(Access::Name),
            "PATH" => Some(Access::Path),
            "EXTENSION" => Some(Access::Extension),
            "CONTENT" => Some(Access::Content),
            "LENGTH" => Some(Access::Length),
            "DEPTH" => Some(Access::Depth),
            "SIZE" => Some(Access::Size),
            "COUNT" => Some(Access::Count),
            "CREATED" => Some(Access::Created),
            "MODIFIED" => Some(Access::Modified),
            "EXISTS" => Some(Access::Exists),
            "OWNER" => Some(Access::Owner),
            "GROUP" => Some(Access::Group),
            "PERMISSIONS" => Some(Access::Permissions),
            "ABSOLUTE" => Some(Access::Absolute),
            "FILES" => Some(Access::Files),
            "ME" | "SELF" | "THIS" => Some(Access::Me),
            _ => None,
        }
    }
}
