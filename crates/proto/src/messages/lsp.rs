use serde::{Serialize, Deserialize};

use crate::messages::buffer::{Anchor, AnchorRange, Location, ProjectTransaction, Transaction, VectorClockEntry};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct GetDefinition {
    pub project_id: u64,
    pub buffer_id: u64,
    pub position: Anchor,
    pub version: Vec<VectorClockEntry>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct GetDefinitionResponse {
    pub links: Vec<LocationLink>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct GetDeclaration {
    pub project_id: u64,
    pub buffer_id: u64,
    pub position: Anchor,
    pub version: Vec<VectorClockEntry>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct GetDeclarationResponse {
    pub links: Vec<LocationLink>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct GetTypeDefinition {
    pub project_id: u64,
    pub buffer_id: u64,
    pub position: Anchor,
    pub version: Vec<VectorClockEntry>,
 }

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct GetTypeDefinitionResponse {
    pub links: Vec<LocationLink>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct GetImplementation {
    pub project_id: u64,
    pub buffer_id: u64,
    pub position: Anchor,
    pub version: Vec<VectorClockEntry>,
 }

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct GetImplementationResponse {
    pub links: Vec<LocationLink>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct GetReferences {
    pub project_id: u64,
    pub buffer_id: u64,
    pub position: Anchor,
    pub version: Vec<VectorClockEntry>,
 }

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct GetReferencesResponse {
    pub locations: Vec<Location>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct GetDocumentHighlights {
    pub project_id: u64,
    pub buffer_id: u64,
    pub position:  Anchor,
    pub version: Vec<VectorClockEntry>,
 }

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct GetDocumentHighlightsResponse {
    pub highlights: Vec<DocumentHighlight>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct LocationLink {
    pub origin: Option<Location>,
    pub target: Location,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Copy)]
pub enum DocumentHighlightKind {
    Text,
    Read,
    Write,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct DocumentHighlight {
    pub kind: DocumentHighlightKind,
    pub start: Anchor,
    pub end: Anchor,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct GetProjectSymbols {
    pub project_id: u64,
    pub query: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct GetProjectSymbolsResponse {
    pub symbols: Vec<Symbol>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Symbol {
    pub source_worktree_id: u64,
    pub worktree_id: u64,
    pub language_server_name: String,
    pub name: String,
    pub kind: i32,
    pub path: String,
    // Cannot use generate anchors for unopened files,
    // so we are forced to use point coords instead
    pub start: PointUtf16,
    pub end: PointUtf16,
    pub signature: Vec<u8>,
    pub language_server_id: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct GetDocumentSymbols {
    pub project_id: u64,
    pub buffer_id: u64,
    pub version: Vec<VectorClockEntry>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct GetDocumentSymbolsResponse {
    pub symbols: Vec<DocumentSymbol>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct DocumentSymbol {
    pub name: String,
    pub kind: i32,
    // Cannot use generate anchors for unopened files,
    // so we are forced to use point coords instead
    pub start: PointUtf16,
    pub end: PointUtf16,
    pub selection_start: PointUtf16,
    pub selection_end: PointUtf16,
    pub children: Vec<DocumentSymbol>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct InlayHints {
    pub project_id: u64,
    pub buffer_id: u64,
    pub start: Anchor,
    pub end: Anchor,
    pub version: Vec<VectorClockEntry>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct InlayHintsResponse {
    pub hints: Vec<InlayHint>,
    pub version: Vec<VectorClockEntry>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct PointUtf16 {
    pub row: u32,
    pub column: u32,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct LspExtExpandMacro {
    pub project_id: u64,
    pub buffer_id: u64,
    pub position: Anchor,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct LspExtExpandMacroResponse {
    pub name: String,
    pub expansion: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct LspExtOpenDocs {
    pub project_id: u64,
    pub buffer_id: u64,
    pub position: Anchor,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct LspExtOpenDocsResponse {
    pub web: Option<String>,
    pub local: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct LspExtSwitchSourceHeader {
    pub project_id: u64,
    pub buffer_id: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct LspExtSwitchSourceHeaderResponse {
    pub target_file: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct LspExtGoToParentModule {
    pub project_id: u64,
    pub buffer_id: u64,
    pub position: Anchor,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct LspExtGoToParentModuleResponse {
    pub links: Vec<LocationLink>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct GetCompletionsResponse {
    pub completions: Vec<Completion>,
    pub version: Vec<VectorClockEntry>,
    // `!is_complete`, inverted for a default of `is_complete = true`
    pub can_reuse: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ApplyCompletionAdditionalEdits {
    pub project_id: u64,
    pub buffer_id: u64,
    pub completion: Completion,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ApplyCompletionAdditionalEditsResponse {
    pub transaction: Transaction,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Copy)]
pub enum CompletionSource {
    Lsp,
    Custom,
    BufferWord,
    Dap,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Completion {
    pub old_replace_start: Anchor,
    pub old_replace_end: Anchor,
    pub new_text: String,
    pub server_id: u64,
    pub lsp_completion: Vec<u8>,
    pub resolved: bool,
    pub source: CompletionSource,
    pub lsp_defaults: Option<Vec<u8>>,
    pub buffer_word_start: Option<Anchor>,
    pub buffer_word_end: Option<Anchor>,
    pub old_insert_start: Anchor,
    pub old_insert_end: Anchor,
    pub sort_text: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct GetCodeActions {
    pub project_id: u64,
    pub buffer_id: u64,
    pub start: Anchor,
    pub end: Anchor,
    pub version: Vec<VectorClockEntry>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct GetCodeActionsResponse {
    pub actions: Vec<CodeAction>,
    pub version: Vec<VectorClockEntry>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct GetSignatureHelp {
    pub project_id: u64,
    pub buffer_id: u64,
    pub position: Anchor,
    pub version: Vec<VectorClockEntry>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct GetSignatureHelpResponse {
    pub signature_help: Option<SignatureHelp>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct SignatureHelp {
    pub signatures: Vec<SignatureInformation>,
    pub active_signature: Option<u32>,
    pub active_parameter: Option<u32>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct SignatureInformation {
    pub label: String,
    pub documentation: Option<Documentation>,
    pub parameters: Vec<ParameterInformation>,
    pub active_parameter: Option<u32>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum Documentation {
    Value(String),
    MarkupContent(MarkupContent),
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Copy)]
pub enum MarkupKind {
    PlainText,
    Markdown,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum ParameterInformationLabel {
    Simple(String),
    Offsets(LabelOffsets),
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ParameterInformation {
    pub label: ParameterInformationLabel,
    pub documentation: Option<Documentation>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct LabelOffsets {
    pub start: u32,
    pub end: u32,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct GetHover {
    pub project_id: u64,
    pub buffer_id: u64,
    pub position: Anchor,
    pub version: Vec<VectorClockEntry>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct GetHoverResponse {
    pub start: Option<Anchor>,
    pub end: Option<Anchor>,
    pub contents: Vec<HoverBlock>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct HoverBlock {
    pub text: String,
    pub language: Option<String>,
    pub is_markdown: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ApplyCodeAction {
    pub project_id: u64,
    pub buffer_id: u64,
    pub action: CodeAction,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ApplyCodeActionResponse {
    pub transaction: ProjectTransaction,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct PrepareRename {
    pub project_id: u64,
    pub buffer_id: u64,
    pub position: Anchor,
    pub version: Vec<VectorClockEntry>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct PrepareRenameResponse {
    pub can_rename: bool,
    pub start: Anchor,
    pub end: Anchor,
    pub version: Vec<VectorClockEntry>,
    pub only_unprepared_rename_supported: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct PerformRename {
    pub project_id: u64,
    pub buffer_id: u64,
    pub position: Anchor,
    pub new_name: String,
    pub version: Vec<VectorClockEntry>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct OnTypeFormatting {
    pub project_id: u64,
    pub buffer_id: u64,
    pub position: Anchor,
    pub trigger: String,
    pub version: Vec<VectorClockEntry>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct OnTypeFormattingResponse {
    pub transaction: Transaction,
}


#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct LinkedEditingRange {
    pub project_id: u64,
    pub buffer_id: u64,
    pub position: Anchor,
    pub version: Vec<VectorClockEntry>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct LinkedEditingRangeResponse {
    pub items: Vec<AnchorRange>,
    pub version: Vec<VectorClockEntry>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct InlayHint {
    pub position: Anchor,
    pub label: InlayHintLabel,
    pub kind: Option<String>,
    pub padding_left: bool,
    pub padding_right: bool,
    pub tooltip: InlayHintTooltip,
    pub resolve_state: ResolveState,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum InlayHintLabel {
    Value(String),
    LabelParts(InlayHintLabelParts),
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct InlayHintLabelParts {
    pub parts: Vec<InlayHintLabelPart>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct InlayHintLabelPart {
    pub value: String,
    pub tooltip: InlayHintLabelPartTooltip,
    pub location_url: Option<String>,
    pub location_range_start: PointUtf16,
    pub location_range_end: PointUtf16,
    pub language_server_id: Option<u64>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum InlayHintTooltip {
    Value(String),
    MarkupContent(MarkupContent),
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum InlayHintLabelPartTooltip {
    Value(String),
    MarkupContent(MarkupContent),
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Copy)]
pub enum LspResolveState {
    Resolved,
    CanResolve,
    Resolving,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ResolveState {
    pub state: LspResolveState,
    pub lsp_value: Option<String>,
    pub lsp_server_id: u64,
}

// This type is used to resolve more than just
// the documentation, but for backwards-compatibility
// reasons we can't rename the type.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ResolveCompletionDocumentation {
    pub project_id: u64,
    pub language_server_id: u64,
    pub lsp_completion: Vec<u8>,
    pub buffer_id: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ResolveCompletionDocumentationResponse {
    pub documentation: String,
    pub documentation_is_markdown: bool,
    pub old_replace_start: Anchor,
    pub old_replace_end: Anchor,
    pub new_text: String,
    pub lsp_completion: Vec<u8>,
    pub old_insert_start: Anchor,
    pub old_insert_end: Anchor,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ResolveInlayHint {
    pub project_id: u64,
    pub buffer_id: u64,
    pub language_server_id: u64,
    pub hint: InlayHint,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ResolveInlayHintResponse {
    pub hint: InlayHint,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct RefreshInlayHints {
    pub project_id: u64,
    pub server_id: u64,
    pub request_id: Option<u64>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct CodeLens {
    pub lsp_lens: Vec<u8>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct GetCodeLens {
    pub project_id: u64,
    pub buffer_id: u64,
    pub version: Vec<VectorClockEntry>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct GetCodeLensResponse {
    pub lens_actions: Vec<CodeAction>,
    pub version: Vec<VectorClockEntry>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct RefreshCodeLens {
    pub project_id: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct MarkupContent {
    pub is_markdown: bool,
    pub value: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct PerformRenameResponse {
    pub transaction: ProjectTransaction,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Copy)]
pub enum CodeActionKind {
    Action,
    Command,
    CodeLens,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct CodeAction {
    pub server_id: u64,
    pub start: Anchor,
    pub end: Anchor,
    pub lsp_action: Vec<u8>,
    pub kind: CodeActionKind,
    pub resolved: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct LanguageServer {
    pub id: u64,
    pub name: String,
    pub worktree_id: Option<u64>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct StartLanguageServer {
    pub project_id: u64,
    pub server: LanguageServer,
    pub capabilities: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct UpdateDiagnosticSummary {
    pub project_id: u64,
    pub worktree_id: u64,
    pub summary: DiagnosticSummary,
    pub more_summaries: Vec<DiagnosticSummary>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct DiagnosticSummary {
    pub path: String,
    pub language_server_id: u64,
    pub error_count: u32,
    pub warning_count: u32,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum UpdateLanguageServerVariant {
    WorkStart(LspWorkStart),
    WorkProgress(LspWorkProgress),
    WorkEnd(LspWorkEnd),
    DiskBasedDiagnosticsUpdating(LspDiskBasedDiagnosticsUpdating),
    DiskBasedDiagnosticsUpdated(LspDiskBasedDiagnosticsUpdated),
    StatusUpdate(StatusUpdate),
    RegisteredForBuffer(RegisteredForBuffer),
    MetadataUpdated(ServerMetadataUpdated),
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct UpdateLanguageServer {
    pub project_id: u64,
    pub language_server_id: u64,
    pub server_name: Option<String>,
    pub variant: UpdateLanguageServerVariant,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum ProgressToken {
    Number(i32),
    String(String),
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct LspWorkStart {
    pub title: Option<String>,
    pub message: Option<String>,
    pub percentage: Option<u32>,
    pub is_cancellable: Option<bool>,
    pub token: ProgressToken,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct LspWorkProgress {
    pub message: Option<String>,
    pub percentage: Option<u32>,
    pub is_cancellable: Option<bool>,
    pub token: ProgressToken,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct LspWorkEnd {
    pub token: ProgressToken,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct LspDiskBasedDiagnosticsUpdating {}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct LspDiskBasedDiagnosticsUpdated {}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum StatusUpdateVariant {
    Binary(ServerBinaryStatus),
    Health(ServerHealth),
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct StatusUpdate {
    pub message: Option<String>,
    pub variant: StatusUpdateVariant,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Copy)]
pub enum ServerHealth {
    Ok,
    Warning,
    Error,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Copy)]
pub enum ServerBinaryStatus {
    None,
    CheckingForUpdate,
    Downloading,
    Starting,
    Stopping,
    Stopped,
    Failed,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct RegisteredForBuffer {
    pub buffer_abs_path: String,
    pub buffer_id: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct LanguageServerBinaryInfo {
    pub path: String,
    pub arguments: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ServerMetadataUpdated {
    pub capabilities: Option<String>,
    pub binary: Option<LanguageServerBinaryInfo>,
    pub configuration: Option<String>,
    pub workspace_folders: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum LanguageServerLogType {
    Log(LogMessage),
    Trace(TraceMessage),
    Rpc(RpcMessage),
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct LanguageServerLog {
    pub project_id: u64,
    pub language_server_id: u64,
    pub message: String,
    pub log_type: LanguageServerLogType,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Copy)]
pub enum LogMessageLevel {
    Log,
    Info,
    Warning,
    Error,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct LogMessage {
    pub level: LogMessageLevel,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct TraceMessage {
    pub verbose_info: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Copy)]
pub enum RpcMessage {
    Received,
    Sent,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct LspLogTrace {
    pub message: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ApplyCodeActionKind {
    pub project_id: u64,
    pub kind: String,
    pub buffer_ids: Vec<u64>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ApplyCodeActionKindResponse {
    pub transaction: ProjectTransaction,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct RegisterBufferWithLanguageServers {
    pub project_id: u64,
    pub buffer_id: u64,
    pub only_servers: Vec<LanguageServerSelector>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Copy)]
pub enum FormatTrigger {
    Save,
    Manual,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct OpenBufferForSymbol {
    pub project_id: u64,
    pub symbol: Symbol,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct OpenBufferForSymbolResponse {
    pub buffer_id: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct BufferFormatRanges {
    pub buffer_id: u64,
    pub ranges: Vec<AnchorRange>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct FormatBuffers {
    pub project_id: u64,
    pub trigger: FormatTrigger,
    pub buffer_ids: Vec<u64>,
    pub buffer_ranges: Vec<BufferFormatRanges>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct FormatBuffersResponse {
    pub transaction: ProjectTransaction,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct GetCompletions {
    pub project_id: u64,
    pub buffer_id: u64,
    pub position: Anchor,
    pub version: Vec<VectorClockEntry>,
    pub server_id: Option<u64>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum CancelLanguageServerWorkVariant {
    Buffer {
        buffer_ids: Vec<u64>,
    },
    Work {
        language_server_id: u64,
        token: Option<ProgressToken>,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct CancelLanguageServerWork {
    pub project_id: u64,
    pub variant: CancelLanguageServerWorkVariant,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Copy)]
pub enum LanguageServerPromptRequestLevel {
    Info,
    Warning,
    Critical,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct LanguageServerPromptRequest {
    pub project_id: u64,

    pub level: LanguageServerPromptRequestLevel,
    pub message: String,
    pub actions: Vec<String>,
    pub lsp_name: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct LanguageServerPromptResponse {
    pub action_response: Option<u64>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct GetDocumentColor {
    pub project_id: u64,
    pub buffer_id: u64,
    pub version: Vec<VectorClockEntry>,

}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct GetDocumentColorResponse {
    pub colors: Vec<ColorInformation>,
    pub version: Vec<VectorClockEntry>,

}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ColorInformation {
    pub lsp_range_start: PointUtf16,
    pub lsp_range_end: PointUtf16,
    pub red: f32,
    pub green: f32,
    pub blue: f32,
    pub alpha: f32,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct GetColorPresentation {
    pub project_id: u64,
    pub buffer_id: u64,
    pub color: ColorInformation,
    pub server_id: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct GetColorPresentationResponse {
    pub presentations: Vec<ColorPresentation>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ColorPresentation {
    pub label: String,
    pub text_edit: Option<TextEdit>,
    pub additional_text_edits: Vec<TextEdit>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct TextEdit {
    pub new_text: String,
    pub lsp_range_start: PointUtf16,
    pub lsp_range_end: PointUtf16,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum LspQueryRequest {
    GetReferences(GetReferences),
    GetDocumentColor(GetDocumentColor),
    GetHover(GetHover),
    GetCodeActions(GetCodeActions),
    GetSignatureHelp(GetSignatureHelp),
    GetCodeLens(GetCodeLens),
    GetDocumentDiagnostics(GetDocumentDiagnostics),
    GetDefinition(GetDefinition),
    GetDeclaration(GetDeclaration),
    GetTypeDefinition(GetTypeDefinition),
    GetImplementation(GetImplementation),
    InlayHints(InlayHints),
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct LspQuery {
    pub project_id: u64,
    pub lsp_request_id: u64,
    pub server_id: Option<u64>,
    pub request: LspQueryRequest,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct LspQueryResponse {
    pub project_id: u64,
    pub lsp_request_id: u64,
    pub responses: Vec<LspResponse>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum LspResponseVariant {
    GetHoverResponse(GetHoverResponse),
    GetCodeActionsResponse(GetCodeActionsResponse),
    GetSignatureHelpResponse(GetSignatureHelpResponse),
    GetCodeLensResponse(GetCodeLensResponse),
    GetDocumentDiagnosticsResponse(GetDocumentDiagnosticsResponse),
    GetDocumentColorResponse(GetDocumentColorResponse),
    GetDefinitionResponse(GetDefinitionResponse),
    GetDeclarationResponse(GetDeclarationResponse),
    GetTypeDefinitionResponse(GetTypeDefinitionResponse),
    GetImplementationResponse(GetImplementationResponse),
    GetReferencesResponse(GetReferencesResponse),
    InlayHintsResponse(InlayHintsResponse),
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct LspResponse {
    pub server_id: u64,
    pub variant: LspResponseVariant,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct AllLanguageServers {}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Copy)]
pub enum LanguageServerSelector {
    ServerId,
    Name,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct RestartLanguageServers {
    pub project_id: u64,
    pub buffer_ids: Vec<u64>,
    pub only_servers: Vec<LanguageServerSelector>,
    pub all: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct StopLanguageServers {
    pub project_id: u64,
    pub buffer_ids: Vec<u64>,
    pub also_servers: Vec<LanguageServerSelector>,
    pub all: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct LspExtRunnables {
    pub project_id: u64,
    pub buffer_id: u64,
    pub position: Option<Anchor>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct LspExtRunnablesResponse {
    pub runnables: Vec<LspRunnable>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct LspRunnable {
    pub task_template: Vec<u8>,
    pub location: Option<LocationLink>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct LspExtCancelFlycheck {
    pub project_id: u64,
    pub language_server_id: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct LspExtRunFlycheck {
    pub project_id: u64,
    pub buffer_id: Option<u64>,
    pub language_server_id: u64,
    pub current_file_only: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct LspExtClearFlycheck {
    pub project_id: u64,
    pub language_server_id: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct LspDiagnosticRelatedInformation {
    pub location_url: Option<String>,
    pub location_range_start: PointUtf16,
    pub location_range_end: PointUtf16,
    pub message: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Copy)]
pub enum LspDiagnosticTag {
    None,
    Unnecessary,
    Deprecated,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Copy)]
pub enum LspDiagnosticSeverity {
    None,
    Error,
    Warning,
    Information,
    Hint,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct LspDiagnostic {
    pub start: PointUtf16,
    pub end: PointUtf16,
    pub severity: LspDiagnosticSeverity,
    pub code: Option<String>,
    pub code_description: Option<String>,
    pub source: Option<String>,
    pub message: String,
    pub related_information: Vec<LspDiagnosticRelatedInformation>,
    pub tags: Vec<LspDiagnosticTag>,
    pub data: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct GetDocumentDiagnostics {
    pub project_id: u64,
    pub buffer_id: u64,
    pub version: Vec<VectorClockEntry>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct GetDocumentDiagnosticsResponse {
    pub pulled_diagnostics: Vec<PulledDiagnostics>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct PulledDiagnostics {
    pub server_id: u64,
    pub uri: String,
    pub result_id: Option<String>,
    pub changed: bool,
    pub diagnostics: Vec<LspDiagnostic>,
    pub registration_id: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct PullWorkspaceDiagnostics {
    pub project_id: u64,
    pub server_id: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Copy)]
pub enum ToggleLspLogsType {
    Log,
    Trace,
    Rpc,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ToggleLspLogs {
    pub project_id: u64,
    pub log_type: ToggleLspLogsType,
    pub server_id: u64,
    pub enabled: bool,

}
