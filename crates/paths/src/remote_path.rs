use crate::PathStyle;
use std::fmt::Display;

#[derive(Debug, Clone)]
pub struct RemotePathBuf {
    style: PathStyle,
    string: String,
}

impl RemotePathBuf {
    pub fn new(string: String, style: PathStyle) -> Self {
        Self { style, string }
    }

    pub fn from_str(path: &str, style: PathStyle) -> Self {
        Self::new(path.to_string(), style)
    }

    pub fn path_style(&self) -> PathStyle {
        self.style
    }

    pub fn into_proto(self) -> String {
        self.string
    }
}

impl Display for RemotePathBuf {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.string)
    }
}
