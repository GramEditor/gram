use serde::{Deserialize, Serialize};
use std::default::Default;

use crate::messages::buffer::{Anchor, EditorAnchor, Selection};
use crate::messages::core::{PeerId, User};
use crate::messages::git::{Branch, StatusEntry};
use crate::messages::lsp::LanguageServer;
use crate::messages::worktree::{Entry, WorktreeMetadata};

#[derive(Default, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct CreateRoom {}

#[derive(Default, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct CreateRoomResponse {
    pub room: Room,
}

#[derive(Default, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct JoinRoom {
    pub id: u64,
}

#[derive(Default, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct JoinRoomResponse {
    pub room: Room,
    pub channel_id: Option<u64>,
}

#[derive(Default, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct RejoinRoom {
    pub id: u64,
    pub reshared_projects: Vec<UpdateProject>,
    pub rejoined_projects: Vec<RejoinProject>,
}

#[derive(Default, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct RejoinRemoteProjects {
    pub rejoined_projects: Vec<RejoinProject>,
}

#[derive(Default, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct RejoinRemoteProjectsResponse {
    pub rejoined_projects: Vec<RejoinedProject>,
}

#[derive(Default, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct RejoinProject {
    pub id: u64,
    pub worktrees: Vec<RejoinWorktree>,
    pub repositories: Vec<RejoinRepository>,
}

#[derive(Default, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct RejoinWorktree {
    pub id: u64,
    pub scan_id: u64,
}

#[derive(Default, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct RejoinRepository {
    pub id: u64,
    pub scan_id: u64,
}

#[derive(Default, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct RejoinRoomResponse {
    pub room: Room,
    pub reshared_projects: Vec<ResharedProject>,
    pub rejoined_projects: Vec<RejoinedProject>,
}

#[derive(Default, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ResharedProject {
    pub id: u64,
}

#[derive(Default, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct RejoinedProject {
    pub id: u64,
    pub worktrees: Vec<WorktreeMetadata>,
    pub language_servers: Vec<LanguageServer>,
    pub language_server_capabilities: Vec<String>,
}

#[derive(Default, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct LeaveRoom {}

#[derive(Default, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Room {
    pub id: u64,
    pub participants: Vec<Participant>,
    pub pending_participants: Vec<PendingParticipant>,
    pub followers: Vec<Follower>,
    pub livekit_room: String,
}

#[derive(Default, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Participant {
    pub user_id: u64,
    pub peer_id: PeerId,
    pub projects: Vec<ParticipantProject>,
    pub location: ParticipantLocation,
    pub participant_index: u32,
}

#[derive(Default, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct PendingParticipant {
    pub user_id: u64,
    pub calling_user_id: u64,
    pub initial_project_id: Option<u64>,
}

#[derive(Default, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ParticipantProject {
    pub id: u64,
    pub worktree_root_names: Vec<String>,
}

#[derive(Default, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Follower {
    pub leader_id: PeerId,
    pub follower_id: PeerId,
    pub project_id: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum ParticipantLocation {
    SharedProject(u64),
    UnsharedProject,
    External,
}
impl Default for ParticipantLocation {
    fn default() -> Self {
        Self::SharedProject(Default::default())
    }
}

#[derive(Default, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct UpdateParticipantLocation {
    pub room_id: u64,
    pub location: ParticipantLocation,
}

#[derive(Default, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct RoomUpdated {
    pub room: Room,
}

#[derive(Default, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ShareProject {
    pub room_id: u64,
    pub worktrees: Vec<WorktreeMetadata>,
    pub is_ssh_project: bool,
    pub windows_paths: Option<bool>,
}

#[derive(Default, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ShareProjectResponse {
    pub project_id: u64,
}

#[derive(Default, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct UnshareProject {
    pub project_id: u64,
}

#[derive(Default, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct UpdateProject {
    pub project_id: u64,
    pub worktrees: Vec<WorktreeMetadata>,
}

#[derive(Default, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct JoinProject {
    pub project_id: u64,
    pub committer_email: Option<String>,
    pub committer_name: Option<String>,
}

#[derive(Default, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct JoinProjectResponse {
    pub project_id: u64,
    pub replica_id: u32,
    pub worktrees: Vec<WorktreeMetadata>,
    pub language_servers: Vec<LanguageServer>,
    pub language_server_capabilities: Vec<String>,
    pub windows_paths: bool,
}

#[derive(Default, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct LeaveProject {
    pub project_id: u64,
}

#[derive(Default, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct UpdateWorktree {
    pub project_id: u64,
    pub worktree_id: u64,
    pub root_name: String,
    pub updated_entries: Vec<Entry>,
    pub removed_entries: Vec<u64>,
    pub updated_repositories: Vec<RepositoryEntry>, // deprecated
    pub removed_repositories: Vec<u64>,             // deprecated
    pub scan_id: u64,
    pub is_last_update: bool,
    pub abs_path: String,
}

// deprecated
#[derive(Default, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct RepositoryEntry {
    pub repository_id: u64,
    pub updated_statuses: Vec<StatusEntry>,
    pub removed_statuses: Vec<String>,
    pub current_merge_conflicts: Vec<String>,
    pub branch_summary: Option<Branch>,
}

#[derive(Default, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct GetUsers {
    pub user_ids: Vec<u64>,
}

#[derive(Default, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct FuzzySearchUsers {
    pub query: String,
}

#[derive(Default, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct UsersResponse {
    pub users: Vec<User>,
}

#[derive(Default, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Follow {
    pub room_id: u64,
    pub project_id: Option<u64>,
    pub leader_id: PeerId,
}

#[derive(Default, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct FollowResponse {
    pub active_view: View,
    // TODO: Remove after version 0.145.x stabilizes.
    pub active_view_id: Option<ViewId>,
    pub views: Vec<View>,
}

#[derive(Default, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Unfollow {
    pub room_id: u64,
    pub project_id: Option<u64>,
    pub leader_id: PeerId,
}

#[derive(Default, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ViewId {
    pub creator: PeerId,
    pub id: u64,
}

#[derive(Default, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct UpdateActiveView {
    pub id: Option<ViewId>,
    pub leader_id: Option<PeerId>,
    pub view: View,
}

#[derive(Default, Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Copy)]
pub enum PanelId {
    #[default]
    AssistantPanel,
    DebugPanel,
}

#[derive(Default, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct UpdateViewEditor {
    pub inserted_excerpts: Vec<ExcerptInsertion>,
    pub deleted_excerpts: Vec<u64>,
    pub selections: Vec<Selection>,
    pub pending_selection: Option<Selection>,
    pub scroll_top_anchor: EditorAnchor,
    pub scroll_x: f64,
    pub scroll_y: f64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum UpdateViewVariant {
    Editor(UpdateViewEditor),
}
impl Default for UpdateViewVariant {
    fn default() -> Self {
        Self::Editor(Default::default())
    }
}

#[derive(Default, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct UpdateView {
    pub id: ViewId,
    pub leader_id: Option<PeerId>,
    pub variant: UpdateViewVariant,
}

#[derive(Default, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ViewEditor {
    pub singleton: bool,
    pub title: Option<String>,
    pub excerpts: Vec<Excerpt>,
    pub selections: Vec<Selection>,
    pub pending_selection: Option<Selection>,
    pub scroll_top_anchor: EditorAnchor,
    pub scroll_x: f64,
    pub scroll_y: f64,
}

#[derive(Default, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ViewContextEditor {
    pub context_id: String,
    pub editor: ViewEditor,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum ViewVariant {
    Editor(ViewEditor),
    ContextEditor(ViewContextEditor),
}
impl Default for ViewVariant {
    fn default() -> Self {
        Self::Editor(Default::default())
    }
}

#[derive(Default, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct View {
    pub id: ViewId,
    pub leader_id: Option<PeerId>,
    pub panel_id: Option<PanelId>,
    pub variant: ViewVariant,
}

#[derive(Default, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ExcerptInsertion {
    pub excerpt: Excerpt,
    pub previous_excerpt_id: Option<u64>,
}

#[derive(Default, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Excerpt {
    pub id: u64,
    pub buffer_id: u64,
    pub context_start: Anchor,
    pub context_end: Anchor,
    pub primary_start: Anchor,
    pub primary_end: Anchor,
}
