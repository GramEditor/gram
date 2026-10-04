use serde::{Deserialize, Serialize};

use crate::messages::buffer::{Range, VectorClockEntry};
use crate::messages::worktree::ProjectPath;

#[derive(Default, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct GitBranchesResponse {
    pub branches: Vec<Branch>,
}

#[derive(Default, Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Copy)]
pub enum UpdateDiffBasesMode {
    // No client is using the unstaged diff.
    #[default]
    HeadOnly,
    // No client is using the diff from HEAD.
    IndexOnly,
    // Both the unstaged and uncommitted diffs are demanded,
    // and the contents of the index and HEAD are the same for this path.
    IndexMatchesHead,
    // Both the unstaged and uncommitted diffs are demanded,
    // and the contents of the index and HEAD differ for this path,
    // where None means the path doesn't exist in that state of the repo.
    IndexAndHead,
}

#[derive(Default, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct UpdateDiffBases {
    pub project_id: u64,
    pub buffer_id: u64,
    pub staged_text: Option<String>,
    pub committed_text: Option<String>,
    pub mode: UpdateDiffBasesMode,
}

#[derive(Default, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct OpenUnstagedDiff {
    pub project_id: u64,
    pub buffer_id: u64,
}

#[derive(Default, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct OpenUnstagedDiffResponse {
    pub staged_text: Option<String>,
}

#[derive(Default, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct OpenUncommittedDiff {
    pub project_id: u64,
    pub buffer_id: u64,
}

#[derive(Default, Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Copy)]
pub enum OpenUncommittedDiffResponseMode {
    #[default]
    IndexMatchesHead,
    IndexAndHead,
}

#[derive(Default, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct OpenUncommittedDiffResponse {
    pub staged_text: Option<String>,
    pub committed_text: Option<String>,
    pub mode: OpenUncommittedDiffResponseMode,
}

#[derive(Default, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct SetIndexText {
    pub project_id: u64,
    pub repository_id: u64,
    pub path: String,
    pub text: Option<String>,
}

#[derive(Default, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct GetPermalinkToLine {
    pub project_id: u64,
    pub buffer_id: u64,
    pub selection: Range,
}

#[derive(Default, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct GetPermalinkToLineResponse {
    pub permalink: String,
}

#[derive(Default, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Branch {
    pub is_head: bool,
    pub ref_name: String,
    pub unix_timestamp: Option<u64>,
    pub upstream: Option<GitUpstream>,
    pub most_recent_commit: Option<CommitSummary>,
}

#[derive(Default, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct GitUpstream {
    pub ref_name: String,
    pub tracking: Option<UpstreamTracking>,
}

#[derive(Default, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct UpstreamTracking {
    pub ahead: u64,
    pub behind: u64,
}

#[derive(Default, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct CommitSummary {
    pub sha: String,
    pub subject: String,
    pub commit_timestamp: i64,
    pub author_name: String,
}

#[derive(Default, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct GitBranches {
    pub project_id: u64,
    pub repository: ProjectPath,
}

#[derive(Default, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct UpdateGitBranch {
    pub project_id: u64,
    pub branch_name: String,
    pub repository: ProjectPath,
}

#[derive(Default, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct UpdateRepository {
    pub project_id: u64,
    pub id: u64,
    pub abs_path: String,
    pub entry_ids: Vec<u64>,
    pub branch_summary: Option<Branch>,
    pub updated_statuses: Vec<StatusEntry>,
    pub removed_statuses: Vec<String>,
    pub current_merge_conflicts: Vec<String>,
    pub scan_id: u64,
    pub is_last_update: bool,
    pub head_commit_details: Option<GitCommitDetails>,
    pub merge_message: Option<String>,
    pub stash_entries: Vec<StashEntry>,
    pub remote_upstream_url: Option<String>,
    pub remote_origin_url: Option<String>,
}

#[derive(Default, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct RemoveRepository {
    pub project_id: u64,
    pub id: u64,
}

#[derive(Default, Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Copy)]
pub enum GitStatus {
    #[default]
    Added,
    Modified,
    Conflict,
    Deleted,
    Updated,
    TypeChanged,
    Renamed,
    Copied,
    Unmodified,
}

#[derive(Default, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum GitFileStatus {
    #[default]
    Untracked,
    Ignored,
    Unmerged {
        first_head: GitStatus,
        second_head: GitStatus,
    },
    Tracked {
        index_status: GitStatus,
        worktree_status: GitStatus,
    },
}

#[derive(Default, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct GitGetBranches {
    pub project_id: u64,
    pub repository_id: u64,
}

#[derive(Default, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct GitCreateBranch {
    pub project_id: u64,
    pub repository_id: u64,
    pub branch_name: String,
}

#[derive(Default, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct GitChangeBranch {
    pub project_id: u64,
    pub repository_id: u64,
    pub branch_name: String,
}

#[derive(Default, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct GitRenameBranch {
    pub project_id: u64,
    pub repository_id: u64,
    pub branch: String,
    pub new_name: String,
}

#[derive(Default, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct GitCreateRemote {
    pub project_id: u64,
    pub repository_id: u64,
    pub remote_name: String,
    pub remote_url: String,
}

#[derive(Default, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct GitRemoveRemote {
    pub project_id: u64,
    pub repository_id: u64,
    pub remote_name: String,
}

#[derive(Default, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct GitDeleteBranch {
    pub project_id: u64,
    pub repository_id: u64,
    pub branch_name: String,
}

#[derive(Default, Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Copy)]
pub enum GitDiffType {
    #[default]
    HeadToWorktree,
    HeadToIndex,
}

#[derive(Default, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct GitDiff {
    pub project_id: u64,
    pub repository_id: u64,
    pub diff_type: GitDiffType,
}

#[derive(Default, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct GitDiffResponse {
    pub diff: String,
}

#[derive(Default, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct GitInit {
    pub project_id: u64,
    pub abs_path: String,
    pub fallback_branch_name: String,
}

#[derive(Default, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct GitClone {
    pub project_id: u64,
    pub abs_path: String,
    pub remote_repo: String,
}

#[derive(Default, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct GitCloneResponse {
    pub success: bool,
}

#[derive(Default, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct CheckForPushedCommits {
    pub project_id: u64,
    pub repository_id: u64,
}

#[derive(Default, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct CheckForPushedCommitsResponse {
    pub pushed_to: Vec<String>,
}

#[derive(Default, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct GitShow {
    pub project_id: u64,
    pub repository_id: u64,
    pub commit: String,
}

#[derive(Default, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct GitCommitDetails {
    pub sha: String,
    pub message: String,
    pub commit_timestamp: i64,
    pub author_email: String,
    pub author_name: String,
    pub refs: Vec<String>,
}

#[derive(Default, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct LoadCommitDiff {
    pub project_id: u64,
    pub repository_id: u64,
    pub commit: String,
}

#[derive(Default, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct LoadCommitDiffResponse {
    pub files: Vec<CommitFile>,
}

#[derive(Default, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct CommitFile {
    pub path: String,
    pub old_text: Option<String>,
    pub new_text: Option<String>,
    pub is_binary: bool,
}

#[derive(Default, Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Copy)]
pub enum GitResetMode {
    #[default]
    Soft,
    Mixed,
}

#[derive(Default, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct GitReset {
    pub project_id: u64,
    pub repository_id: u64,
    pub commit: String,
    pub mode: GitResetMode,
}

#[derive(Default, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct GitCheckoutFiles {
    pub project_id: u64,
    pub repository_id: u64,
    pub commit: String,
    pub paths: Vec<String>,
}

#[derive(Default, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct GitFileHistory {
    pub project_id: u64,
    pub repository_id: u64,
    pub path: String,
    pub skip: u64,
    pub limit: Option<u64>,
}

#[derive(Default, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct GitFileHistoryResponse {
    pub entries: Vec<FileHistoryEntry>,
    pub path: String,
}

#[derive(Default, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct GitCommitHistory {
    pub project_id: u64,
    pub repository_id: u64,
    pub path: Option<String>,
    pub skip: u64,
    pub limit: Option<u64>,
}

#[derive(Default, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct GitCommitHistoryResponse {
    pub entries: Vec<FileHistoryEntry>,
    pub path: Option<String>,
}

#[derive(Default, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct FileHistoryEntry {
    pub sha: String,
    pub subject: String,
    pub message: String,
    pub commit_timestamp: i64,
    pub author_name: String,
    pub author_email: String,
    pub refs: Vec<String>,
}

#[derive(Default, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct StatusEntry {
    pub repo_path: String,
    pub simple_status: GitStatus,
    pub status: GitFileStatus,
    pub diff_stat_added: Option<u32>,
    pub diff_stat_deleted: Option<u32>,
}

#[derive(Default, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct StashEntry {
    pub oid: Vec<u8>,
    pub message: String,
    pub branch: Option<String>,
    pub index: u64,
    pub timestamp: i64,
}

#[derive(Default, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Stage {
    pub project_id: u64,
    pub repository_id: u64,
    pub paths: Vec<String>,
}

#[derive(Default, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Unstage {
    pub project_id: u64,
    pub repository_id: u64,
    pub paths: Vec<String>,
}

#[derive(Default, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Stash {
    pub project_id: u64,
    pub repository_id: u64,
    pub paths: Vec<String>,
}

#[derive(Default, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct StashPop {
    pub project_id: u64,
    pub repository_id: u64,
    pub stash_index: Option<u64>,
}

#[derive(Default, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct StashApply {
    pub project_id: u64,
    pub repository_id: u64,
    pub stash_index: Option<u64>,
}

#[derive(Default, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct StashDrop {
    pub project_id: u64,
    pub repository_id: u64,
    pub stash_index: Option<u64>,
}

#[derive(Default, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct CommitOptions {
    pub amend: bool,
    pub signoff: bool,
}

#[derive(Default, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Commit {
    pub project_id: u64,
    pub repository_id: u64,
    pub name: Option<String>,
    pub email: Option<String>,
    pub message: String,
    pub options: Option<CommitOptions>,
    pub askpass_id: u64,
}

#[derive(Default, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct OpenCommitMessageBuffer {
    pub project_id: u64,
    pub repository_id: u64,
}

#[derive(Default, Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Copy)]
pub enum PushOptions {
    #[default]
    SetUpstream,
    Force,
}

#[derive(Default, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Push {
    pub project_id: u64,
    pub repository_id: u64,
    pub remote_name: String,
    pub branch_name: String,
    pub options: Option<PushOptions>,
    pub askpass_id: u64,
    pub remote_branch_name: String,
}

#[derive(Default, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Fetch {
    pub project_id: u64,
    pub repository_id: u64,
    pub askpass_id: u64,
    pub remote: Option<String>,
}

#[derive(Default, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct GetRemotes {
    pub project_id: u64,
    pub repository_id: u64,
    pub branch_name: Option<String>,
    pub is_push: bool,
}

#[derive(Default, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct GetRemotesResponse {
    pub remotes: Vec<String>,
}

#[derive(Default, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Pull {
    pub project_id: u64,
    pub repository_id: u64,
    pub remote_name: String,
    pub branch_name: Option<String>,
    pub askpass_id: u64,
    pub rebase: bool,
}

#[derive(Default, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct RemoteMessageResponse {
    pub stdout: String,
    pub stderr: String,
}

#[derive(Default, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct BlameBuffer {
    pub project_id: u64,
    pub buffer_id: u64,
    pub version: Vec<VectorClockEntry>,
}

#[derive(Default, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct BlameEntry {
    pub sha: Vec<u8>,

    pub start_line: u32,
    pub end_line: u32,
    pub original_line_number: u32,

    pub author: Option<String>,
    pub author_mail: Option<String>,
    pub author_time: Option<i64>,
    pub author_tz: Option<String>,

    pub committer: Option<String>,
    pub committer_mail: Option<String>,
    pub committer_time: Option<i64>,
    pub committer_tz: Option<String>,

    pub summary: Option<String>,
    pub previous: Option<String>,

    pub filename: String,
}

#[derive(Default, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct CommitMessage {
    pub oid: Vec<u8>,
    pub message: String,
}

#[derive(Default, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct CommitPermalink {
    pub oid: Vec<u8>,
    pub permalink: String,
}

#[derive(Default, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct BlameResponse {
    pub entries: Vec<BlameEntry>,
    pub messages: Vec<CommitMessage>,
}

#[derive(Default, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct BlameBufferResponse {
    pub blame_response: Option<BlameResponse>,
}

#[derive(Default, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct GetDefaultBranch {
    pub project_id: u64,
    pub repository_id: u64,
}

#[derive(Default, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct GetDefaultBranchResponse {
    pub branch: Option<String>,
}

#[derive(Default, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct GetTreeDiff {
    pub project_id: u64,
    pub repository_id: u64,
    pub is_merge: bool,
    pub base: String,
    pub head: String,
}

#[derive(Default, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct GetTreeDiffResponse {
    pub entries: Vec<TreeDiffStatus>,
}

#[derive(Default, Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Copy)]
pub enum DiffStatus {
    #[default]
    Added,
    Modified,
    Deleted,
}

#[derive(Default, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct TreeDiffStatus {
    pub status: DiffStatus,
    pub path: String,
    pub oid: Option<String>,
}

#[derive(Default, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct GetBlobContent {
    pub project_id: u64,
    pub repository_id: u64,
    pub oid: String,
}

#[derive(Default, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct GetBlobContentResponse {
    pub content: String,
}

#[derive(Default, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct GitGetWorktrees {
    pub project_id: u64,
    pub repository_id: u64,
}

#[derive(Default, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct GitWorktreesResponse {
    pub worktrees: Vec<Worktree>,
}

#[derive(Default, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Worktree {
    pub path: String,
    pub ref_name: String,
    pub sha: String,
}

#[derive(Default, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct GitCreateWorktree {
    pub project_id: u64,
    pub repository_id: u64,
    pub name: String,
    pub directory: String,
    pub commit: Option<String>,
}

#[derive(Default, Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Copy)]
pub enum GitHook {
    #[default]
    PreCommit,
}

#[derive(Default, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct RunGitHook {
    pub project_id: u64,
    pub repository_id: u64,
    pub hook: GitHook,
}
