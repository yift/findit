use std::fmt::Display;

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

impl Display for Access {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Access::Parent => write!(f, "Parent"),
            Access::Name => write!(f, "Name"),
            Access::Path => write!(f, "Path"),
            Access::Extension => write!(f, "Extension"),
            Access::Me => write!(f, "Me"),
            Access::Absolute => write!(f, "Absolute"),
            Access::Content => write!(f, "Content"),
            Access::Length => write!(f, "Length"),
            Access::Depth => write!(f, "Depth"),
            Access::Size => write!(f, "Size"),
            Access::Count => write!(f, "Count"),
            Access::Created => write!(f, "Created"),
            Access::Modified => write!(f, "Modified"),
            Access::Exists => write!(f, "Exists"),
            Access::IsDir => write!(f, "IS Dir"),
            Access::IsFile => write!(f, "IS File"),
            Access::IsLink => write!(f, "IS Link"),
            Access::IsNotDir => write!(f, "IS NOT Dir"),
            Access::IsNotFile => write!(f, "IS NOT File"),
            Access::IsNotLink => write!(f, "IS NOT Link"),
            Access::Owner => write!(f, "Owner"),
            Access::Group => write!(f, "Group"),
            Access::Permissions => write!(f, "Permissions"),
        }
    }
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
