use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ListToolchains {
    pub project_id: u64,
    pub worktree_id: u64,
    pub language_name: String,
    pub path: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Toolchain {
    pub name: String,
    pub path: String,
    pub raw_json: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ToolchainGroup {
    pub start_index: u64,
    pub name: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ListToolchainsResponse {
    pub toolchains: Vec<Toolchain>,
    pub has_values: bool,
    pub groups: Vec<ToolchainGroup>,
    pub relative_worktree_path: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ActivateToolchain {
    pub project_id: u64,
    pub worktree_id: u64,
    pub toolchain: Toolchain,
    pub language_name: String,
    pub path: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ActiveToolchain {
    pub project_id: u64,
    pub worktree_id: u64,
    pub language_name: String,
    pub path: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ActiveToolchainResponse {
    pub toolchain: Option<Toolchain>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ResolveToolchain {
    pub project_id: u64,
    pub abs_path: String,
    pub language_name: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum ResolveToolchainResponse {
    Toolchain(Toolchain),
    Error(String),
}
