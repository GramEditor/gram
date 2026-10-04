use serde::{Deserialize, Serialize};
use std::default::Default;

use crate::messages::core::PeerId;
use crate::messages::worktree::{File, ProjectPath, Timestamp};

#[derive(Default, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct OpenNewBuffer {
    pub project_id: u64,
}

#[derive(Default, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct OpenBufferResponse {
    pub buffer_id: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum CreateBufferForPeerVariant {
    State(BufferState),
    Chunk(BufferChunk),
}
impl Default for CreateBufferForPeerVariant {
    fn default() -> Self {
        Self::State(Default::default())
    }
}

#[derive(Default, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct CreateBufferForPeer {
    pub project_id: u64,
    pub peer_id: PeerId,
    pub variant: CreateBufferForPeerVariant,
}

#[derive(Default, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct UpdateBuffer {
    pub project_id: u64,
    pub buffer_id: u64,
    pub operations: Vec<Operation>,
}

#[derive(Default, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct OpenBufferByPath {
    pub project_id: u64,
    pub worktree_id: u64,
    pub path: String,
}

#[derive(Default, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct OpenBufferById {
    pub project_id: u64,
    pub id: u64,
}

#[derive(Default, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct UpdateBufferFile {
    pub project_id: u64,
    pub buffer_id: u64,
    pub file: File,
}

#[derive(Default, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct SaveBuffer {
    pub project_id: u64,
    pub buffer_id: u64,
    pub version: Vec<VectorClockEntry>,
    pub new_path: Option<ProjectPath>,
}

#[derive(Default, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct CloseBuffer {
    pub project_id: u64,
    pub buffer_id: u64,
}

#[derive(Default, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct BufferSaved {
    pub project_id: u64,
    pub buffer_id: u64,
    pub version: Vec<VectorClockEntry>,
    pub mtime: Option<Timestamp>,
}

#[derive(Default, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct BufferReloaded {
    pub project_id: u64,
    pub buffer_id: u64,
    pub version: Vec<VectorClockEntry>,
    pub mtime: Option<Timestamp>,
    pub line_ending: LineEnding,
}

#[derive(Default, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ReloadBuffers {
    pub project_id: u64,
    pub buffer_ids: Vec<u64>,
}

#[derive(Default, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ReloadBuffersResponse {
    pub transaction: ProjectTransaction,
}

#[derive(Default, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct SynchronizeBuffers {
    pub project_id: u64,
    pub buffers: Vec<BufferVersion>,
}

#[derive(Default, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct SynchronizeBuffersResponse {
    pub buffers: Vec<BufferVersion>,
}

#[derive(Default, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct BufferVersion {
    pub id: u64,
    pub version: Vec<VectorClockEntry>,
}

#[derive(Default, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct BufferState {
    pub id: u64,
    pub file: Option<File>,
    pub base_text: String,
    pub line_ending: LineEnding,
    pub saved_version: Vec<VectorClockEntry>,
    pub saved_mtime: Option<Timestamp>,
}

#[derive(Default, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct BufferChunk {
    pub buffer_id: u64,
    pub operations: Vec<Operation>,
    pub is_last: bool,
}

#[derive(Default, Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Copy)]
pub enum LineEnding {
    #[default]
    Unix,
    Windows,
}

#[derive(Default, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct VectorClockEntry {
    pub replica_id: u32,
    pub timestamp: u32,
}

#[derive(Default, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct UndoMapEntry {
    pub replica_id: u32,
    pub local_timestamp: u32,
    pub counts: Vec<UndoCount>,
}

#[derive(Default, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct UndoCount {
    pub replica_id: u32,
    pub lamport_timestamp: u32,
    pub count: u32,
}

#[derive(Default, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct OperationEdit {
    pub replica_id: u32,
    pub lamport_timestamp: u32,
    pub version: Vec<VectorClockEntry>,
    pub ranges: Vec<Range>,
    pub new_text: Vec<String>,
}

#[derive(Default, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct OperationUndo {
    pub replica_id: u32,
    pub lamport_timestamp: u32,
    pub version: Vec<VectorClockEntry>,
    pub counts: Vec<UndoCount>,
}

#[derive(Default, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct OperationUpdateSelections {
    pub replica_id: u32,
    pub lamport_timestamp: u32,
    pub selections: Vec<Selection>,
    pub line_mode: bool,
    pub cursor_shape: CursorShape,
}

#[derive(Default, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct OperationUpdateCompletionTriggers {
    pub replica_id: u32,
    pub lamport_timestamp: u32,
    pub triggers: Vec<String>,
    pub language_server_id: u64,
}

#[derive(Default, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct OperationUpdateLineEnding {
    pub replica_id: u32,
    pub lamport_timestamp: u32,
    pub line_ending: LineEnding,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum Operation {
    Edit(OperationEdit),
    Undo(OperationUndo),
    UpdateSelections(OperationUpdateSelections),
    UpdateDiagnostics(UpdateDiagnostics),
    UpdateCompletionTriggers(OperationUpdateCompletionTriggers),
    UpdateLineEnding(OperationUpdateLineEnding),
}
impl Default for Operation {
    fn default() -> Self {
        Self::Edit(Default::default())
    }
}

#[derive(Default, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ProjectTransaction {
    pub buffer_ids: Vec<u64>,
    pub transactions: Vec<Transaction>,
}

#[derive(Default, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Transaction {
    pub id: LamportTimestamp,
    pub edit_ids: Vec<LamportTimestamp>,
    pub start: Vec<VectorClockEntry>,
}

#[derive(Default, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct LamportTimestamp {
    pub replica_id: u32,
    pub value: u32,
}

#[derive(Default, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Range {
    pub start: u64,
    pub end: u64,
}

#[derive(Default, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Selection {
    pub id: u64,
    pub start: EditorAnchor,
    pub end: EditorAnchor,
    pub reversed: bool,
}

#[derive(Default, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct EditorAnchor {
    pub excerpt_id: u64,
    pub anchor: Anchor,
}

#[derive(Default, Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Copy)]
pub enum CursorShape {
    #[default]
    CursorBar,
    CursorBlock,
    CursorUnderscore,
    CursorHollow,
}

#[derive(Default, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct UpdateDiagnostics {
    pub replica_id: u32,
    pub lamport_timestamp: u32,
    pub server_id: u64,
    pub diagnostics: Vec<Diagnostic>,
}

#[derive(Default, Serialize, Deserialize, Clone, Debug, PartialEq, Copy)]
pub struct Anchor {
    pub replica_id: u32,
    pub timestamp: u32,
    pub offset: u64,
    pub bias: Bias,
    pub buffer_id: Option<u64>,
}

#[derive(Default, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct AnchorRange {
    pub start: Anchor,
    pub end: Anchor,
}

#[derive(Default, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Location {
    pub buffer_id: u64,
    pub start: Anchor,
    pub end: Anchor,
}

#[derive(Default, Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Copy)]
pub enum Bias {
    #[default]
    Left,
    Right,
}

#[derive(Default, Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Copy)]
pub enum DiagnosticSourceKind {
    #[default]
    Pulled,
    Pushed,
    Other,
}

#[derive(Default, Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Copy)]
pub enum DiagnosticSeverity {
    #[default]
    None,
    Error,
    Warning,
    Information,
    Hint,
}

#[derive(Default, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Diagnostic {
    pub start: Anchor,
    pub end: Anchor,
    pub source: Option<String>,
    pub registration_id: Option<String>,

    pub source_kind: DiagnosticSourceKind,
    pub severity: DiagnosticSeverity,
    pub message: String,
    pub code: Option<String>,
    pub group_id: u64,
    pub is_primary: bool,

    pub is_disk_based: bool,
    pub is_unnecessary: bool,
    pub underline: bool,

    pub data: Option<String>,
    pub code_description: Option<String>,
    pub markdown: Option<String>,
}

#[derive(Default, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct SearchQuery {
    pub query: String,
    pub regex: bool,
    pub whole_word: bool,
    pub case_sensitive: bool,
    pub files_to_include: Vec<String>,
    pub files_to_exclude: Vec<String>,
    pub match_full_paths: bool,
    pub include_ignored: bool,
    pub files_to_include_legacy: String,
    pub files_to_exclude_legacy: String,
}

#[derive(Default, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct FindSearchCandidates {
    pub project_id: u64,
    pub query: SearchQuery,
    pub limit: u64,
}

#[derive(Default, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct FindSearchCandidatesResponse {
    pub buffer_ids: Vec<u64>,
}
