#[derive(Debug, PartialEq, Clone, Copy)]
pub(crate) enum Access {
    Parent,
    Name,
    Path,
    Absolute,
    Me,
    Extension,
    Content,
    Length,
    Depth,
    Size,
    Count,
    Created,
    Modified,
    Exists,
    IsDir,
    IsFile,
    IsLink,
    IsNotDir,
    IsNotFile,
    IsNotLink,
    Owner,
    Group,
    Permissions,
}

impl Access {
    pub(crate) fn from_str(name: &str) -> Option<Self> {
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
            "ME" | "SELF" | "THIS" => Some(Access::Me),
            _ => None,
        }
    }
}
