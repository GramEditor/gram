use serde::{Serialize, Deserialize};

#[derive(Default, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Timestamp {
    pub seconds: u64,
    pub nanos: u32,
}

#[derive(Default, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct File {
    pub worktree_id: u64,
    pub entry_id: Option<u64>,
    pub path: String,
    pub mtime: Timestamp,
    pub is_deleted: bool,
    pub is_historic: bool,
}

#[derive(Default, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Entry {
    pub id: u64,
    pub is_dir: bool,
    pub path: String,
    pub inode: u64,
    pub mtime: Timestamp,
    pub is_ignored: bool,
    pub is_external: bool,
    pub is_fifo: bool,
    pub size: Option<u64>,
    pub canonical_path: Option<String>,
    pub is_hidden: bool,
}

#[derive(Default, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct AddWorktree {
    pub path: String,
    pub project_id: u64,
    pub visible: bool,
}

#[derive(Default, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct AddWorktreeResponse {
    pub worktree_id: u64,
    pub canonicalized_path: String,
}

#[derive(Default, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct RemoveWorktree {
    pub worktree_id: u64,
}

#[derive(Default, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct GetPathMetadata {
    pub project_id: u64,
    pub path: String,
}

#[derive(Default, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct GetPathMetadataResponse {
    pub exists: bool,
    pub path: String,
    pub is_dir: bool,
}

#[derive(Default, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct WorktreeMetadata {
    pub id: u64,
    pub root_name: String,
    pub visible: bool,
    pub abs_path: String,
}

#[derive(Default, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ProjectPath {
    pub worktree_id: u64,
    pub path: String,
}

#[derive(Default, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ListRemoteDirectoryConfig {
    pub is_dir: bool,
}

#[derive(Default, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ListRemoteDirectory {
    pub dev_server_id: u64,
    pub path: String,
    pub config: ListRemoteDirectoryConfig,
}

#[derive(Default, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct EntryInfo {
    pub is_dir: bool,
}

#[derive(Default, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ListRemoteDirectoryResponse {
    pub entries: Vec<String>,
    pub entry_info: Vec<EntryInfo>,
}

#[derive(Default, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct CreateProjectEntry {
    pub project_id: u64,
    pub worktree_id: u64,
    pub path: String,
    pub is_directory: bool,
    pub content: Option<Vec<u8>>,
}

#[derive(Default, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct RenameProjectEntry {
    pub project_id: u64,
    pub entry_id: u64,
    pub new_path: String,
    pub new_worktree_id: u64,
}

#[derive(Default, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct CopyProjectEntry {
    pub project_id: u64,
    pub entry_id: u64,
    pub new_path: String,
    pub new_worktree_id: u64,
}

#[derive(Default, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct DeleteProjectEntry {
    pub project_id: u64,
    pub entry_id: u64,
    pub use_trash: bool,
}

#[derive(Default, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ExpandProjectEntry {
    pub project_id: u64,
    pub entry_id: u64,
}

#[derive(Default, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ExpandProjectEntryResponse {
    pub worktree_scan_id: u64,
}

#[derive(Default, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ExpandAllForProjectEntry {
    pub project_id: u64,
    pub entry_id: u64,
}

#[derive(Default, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ExpandAllForProjectEntryResponse {
    pub worktree_scan_id: u64,
}

#[derive(Default, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct RescanDirectory {
    pub project_id: u64,
    pub worktree_id: u64,
    pub path: String,
}

#[derive(Default, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct RescanDirectoryResponse {
    pub worktree_scan_id: u64,
}

#[derive(Default, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ProjectEntryResponse {
    pub entry: Option<Entry>,
    pub worktree_scan_id: u64,
}

#[derive(Default, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct UpdateWorktreeSettings {
    pub project_id: u64,
    pub worktree_id: u64,
    pub path: String,
    pub content: Option<String>,
    pub kind: Option<LocalSettingsKind>,
}

#[derive(Default, Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Copy)]
pub enum LocalSettingsKind {
    #[default]
    Settings,
    Tasks,
    Editorconfig,
    Debug,
}

#[derive(Default, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct UpdateUserSettings {
    pub project_id: u64,
    pub contents: String,
}
