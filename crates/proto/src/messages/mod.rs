use serde::{Serialize, Deserialize};

mod app;
mod buffer;
mod call;
mod core;
mod debugger;
mod git;
mod image;
mod lsp;
mod notification;
mod task;
mod toolchain;
mod worktree;

pub use app::*;
pub use buffer::*;
pub use call::*;
pub use core::*;
pub use debugger::*;
pub use git::*;
pub use image::*;
pub use lsp::*;
pub use notification::*;
pub use task::*;
pub use toolchain::*;
pub use worktree::*;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Envelope {
    pub id: u32,
    pub responding_to: Option<u32>,
    pub original_sender_id: Option<PeerId>,
    pub ack_id: Option<u32>,
    pub payload: Payload,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Hello {
    peer_id: PeerId
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Ping {}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Ack {}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Error {
    pub message: String,
    pub code: ErrorCode,
    pub tags: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Copy)]
pub enum ErrorCode {
    Internal,
    Disconnected,
    SignedOut,
    UpgradeRequired,
    Forbidden,
    NeedsCla,
    BadPublicNesting,
    CircularNesting,
    WrongMoveTarget,
    UnsharedItem,
    NoSuchProject,
    DevServerProjectPathDoesNotExist,
    RemoteUpgradeRequired,
    RateLimitExceeded,
    CommitFailed,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct EndStream {}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Test {
    pub id: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct FlushBufferedMessages {}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct FlushBufferedMessagesResponse {}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct RemoteStarted {}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum Payload {
    Hello(Hello),
    Ack(Ack),
    Error(Error),
    Ping(Ping),
    Test(Test),
    EndStream(EndStream),

    CreateRoom(CreateRoom),
    CreateRoomResponse(CreateRoomResponse),
    JoinRoom(JoinRoom),
    JoinRoomResponse(JoinRoomResponse),
    RejoinRoom(RejoinRoom),
    RejoinRoomResponse(RejoinRoomResponse),
    LeaveRoom(LeaveRoom),
    UpdateParticipantLocation(UpdateParticipantLocation),
    RoomUpdated(RoomUpdated),

    ShareProject(ShareProject),
    ShareProjectResponse(ShareProjectResponse),
    UnshareProject(UnshareProject),
    JoinProject(JoinProject),
    JoinProjectResponse(JoinProjectResponse),
    LeaveProject(LeaveProject),

    GetDefinition(GetDefinition),
    GetDefinitionResponse(GetDefinitionResponse),
    GetDeclaration(GetDeclaration),
    GetDeclarationResponse(GetDeclarationResponse),
    GetTypeDefinition(GetTypeDefinition),
    GetTypeDefinitionResponse(GetTypeDefinitionResponse),

    GetReferences(GetReferences),
    GetReferencesResponse(GetReferencesResponse),
    GetDocumentHighlights(GetDocumentHighlights),
    GetDocumentHighlightsResponse(GetDocumentHighlightsResponse),
    GetProjectSymbols(GetProjectSymbols),
    GetProjectSymbolsResponse(GetProjectSymbolsResponse),
    OpenBufferForSymbol(OpenBufferForSymbol),
    OpenBufferForSymbolResponse(OpenBufferForSymbolResponse),

    UpdateProject(UpdateProject),
    UpdateWorktree(UpdateWorktree),

    CreateProjectEntry(CreateProjectEntry),
    RenameProjectEntry(RenameProjectEntry),
    CopyProjectEntry(CopyProjectEntry),
    DeleteProjectEntry(DeleteProjectEntry),
    ProjectEntryResponse(ProjectEntryResponse),
    ExpandProjectEntry(ExpandProjectEntry),
    ExpandProjectEntryResponse(ExpandProjectEntryResponse),
    ExpandAllForProjectEntry(ExpandAllForProjectEntry),
    ExpandAllForProjectEntryResponse(ExpandAllForProjectEntryResponse),
    UpdateDiagnosticSummary(UpdateDiagnosticSummary),
    StartLanguageServer(StartLanguageServer),
    UpdateLanguageServer(UpdateLanguageServer),

    OpenBufferById(OpenBufferById),
    OpenBufferByPath(OpenBufferByPath),
    OpenBufferResponse(OpenBufferResponse),
    CreateBufferForPeer(CreateBufferForPeer),
    UpdateBuffer(UpdateBuffer),
    UpdateBufferFile(UpdateBufferFile),
    SaveBuffer(SaveBuffer),
    BufferSaved(BufferSaved),
    BufferReloaded(BufferReloaded),
    ReloadBuffers(ReloadBuffers),
    ReloadBuffersResponse(ReloadBuffersResponse),
    SynchronizeBuffers(SynchronizeBuffers),
    SynchronizeBuffersResponse(SynchronizeBuffersResponse),
    FormatBuffers(FormatBuffers),
    FormatBuffersResponse(FormatBuffersResponse),
    GetCompletions(GetCompletions),
    GetCompletionsResponse(GetCompletionsResponse),
    ResolveCompletionDocumentation(ResolveCompletionDocumentation),
    ResolveCompletionDocumentationResponse(ResolveCompletionDocumentationResponse),
    ApplyCompletionAdditionalEdits(ApplyCompletionAdditionalEdits),
    ApplyCompletionAdditionalEditsResponse(ApplyCompletionAdditionalEditsResponse),
    GetCodeActions(GetCodeActions),
    GetCodeActionsResponse(GetCodeActionsResponse),
    GetHover(GetHover),
    GetHoverResponse(GetHoverResponse),
    ApplyCodeAction(ApplyCodeAction),
    ApplyCodeActionResponse(ApplyCodeActionResponse),
    PrepareRename(PrepareRename),
    PrepareRenameResponse(PrepareRenameResponse),
    PerformRename(PerformRename),
    PerformRenameResponse(PerformRenameResponse),

    UpdateInviteInfo(UpdateInviteInfo),

    GetUsers(GetUsers),
    FuzzySearchUsers(FuzzySearchUsers),
    UsersResponse(UsersResponse),

    Follow(Follow),
    FollowResponse(FollowResponse),
    Unfollow(Unfollow),
    UpdateDiffBases(UpdateDiffBases),

    OnTypeFormatting(OnTypeFormatting),
    OnTypeFormattingResponse(OnTypeFormattingResponse),

    UpdateWorktreeSettings(UpdateWorktreeSettings),

    InlayHints(InlayHints),
    InlayHintsResponse(InlayHintsResponse),
    ResolveInlayHint(ResolveInlayHint),
    ResolveInlayHintResponse(ResolveInlayHintResponse),
    RefreshInlayHints(RefreshInlayHints),

    AddNotification(AddNotification),
    GetNotifications(GetNotifications),
    GetNotificationsResponse(GetNotificationsResponse),
    DeleteNotification(DeleteNotification),
    MarkNotificationRead(MarkNotificationRead),
    LspExtExpandMacro(LspExtExpandMacro),
    LspExtExpandMacroResponse(LspExtExpandMacroResponse),

    GetImplementation(GetImplementation),
    GetImplementationResponse(GetImplementationResponse),

    BlameBuffer(BlameBuffer),
    BlameBufferResponse(BlameBufferResponse),

    UpdateNotification(UpdateNotification),

    RestartLanguageServers(RestartLanguageServers),

    RejoinRemoteProjects(RejoinRemoteProjects),
    RejoinRemoteProjectsResponse(RejoinRemoteProjectsResponse),

    OpenNewBuffer(OpenNewBuffer),

    TaskContextForLocation(TaskContextForLocation),
    TaskContext(TaskContext),

    LinkedEditingRange(LinkedEditingRange),
    LinkedEditingRangeResponse(LinkedEditingRangeResponse),

    GetSignatureHelp(GetSignatureHelp),
    GetSignatureHelpResponse(GetSignatureHelpResponse),

    ListRemoteDirectory(ListRemoteDirectory),
    ListRemoteDirectoryResponse(ListRemoteDirectoryResponse),
    AddWorktree(AddWorktree),
    AddWorktreeResponse(AddWorktreeResponse),

    LspExtSwitchSourceHeader(LspExtSwitchSourceHeader),
    LspExtSwitchSourceHeaderResponse(LspExtSwitchSourceHeaderResponse),

    FindSearchCandidates(FindSearchCandidates),
    FindSearchCandidatesResponse(FindSearchCandidatesResponse),

    CloseBuffer(CloseBuffer),

    ShutdownRemoteServer(ShutdownRemoteServer),

    RemoveWorktree(RemoveWorktree),

    LanguageServerLog(LanguageServerLog),

    Toast(Toast),
    HideToast(HideToast),

    OpenServerSettings(OpenServerSettings),

    GetPermalinkToLine(GetPermalinkToLine),
    GetPermalinkToLineResponse(GetPermalinkToLineResponse),

    FlushBufferedMessages(FlushBufferedMessages),

    LanguageServerPromptRequest(LanguageServerPromptRequest),
    LanguageServerPromptResponse(LanguageServerPromptResponse),

    GitBranchesResponse(GitBranchesResponse),

    UpdateGitBranch(UpdateGitBranch),

    ListToolchains(ListToolchains),
    ListToolchainsResponse(ListToolchainsResponse),
    ActivateToolchain(ActivateToolchain),
    ActiveToolchain(ActiveToolchain),
    ActiveToolchainResponse(ActiveToolchainResponse),

    GetPathMetadata(GetPathMetadata),
    GetPathMetadataResponse(GetPathMetadataResponse),

    CancelLanguageServerWork(CancelLanguageServerWork),

    LspExtOpenDocs(LspExtOpenDocs),
    LspExtOpenDocsResponse(LspExtOpenDocsResponse),

    SyncExtensions(SyncExtensions),
    SyncExtensionsResponse(SyncExtensionsResponse),
    InstallExtension(InstallExtension),

    OpenUnstagedDiff(OpenUnstagedDiff),
    OpenUnstagedDiffResponse(OpenUnstagedDiffResponse),

    RegisterBufferWithLanguageServers(RegisterBufferWithLanguageServers),

    Stage(Stage),
    Unstage(Unstage),
    Commit(Commit),
    OpenCommitMessageBuffer(OpenCommitMessageBuffer),

    OpenUncommittedDiff(OpenUncommittedDiff),
    OpenUncommittedDiffResponse(OpenUncommittedDiffResponse),

    SetIndexText(SetIndexText),

    GitShow(GitShow),
    GitReset(GitReset),
    GitCommitDetails(GitCommitDetails),
    GitCheckoutFiles(GitCheckoutFiles),

    Push(Push),
    Fetch(Fetch),
    GetRemotes(GetRemotes),
    GetRemotesResponse(GetRemotesResponse),
    Pull(Pull),

    ApplyCodeActionKind(ApplyCodeActionKind),
    ApplyCodeActionKindResponse(ApplyCodeActionKindResponse),

    RemoteMessageResponse(RemoteMessageResponse),

    GitGetBranches(GitGetBranches),
    GitCreateBranch(GitCreateBranch),
    GitChangeBranch(GitChangeBranch),

    CheckForPushedCommits(CheckForPushedCommits),
    CheckForPushedCommitsResponse(CheckForPushedCommitsResponse),

    AskPassRequest(AskPassRequest),
    AskPassResponse(AskPassResponse),

    GitDiff(GitDiff),
    GitDiffResponse(GitDiffResponse),
    GitInit(GitInit),

    CodeLens(CodeLens),
    GetCodeLens(GetCodeLens),
    GetCodeLensResponse(GetCodeLensResponse),
    RefreshCodeLens(RefreshCodeLens),

    ToggleBreakpoint(ToggleBreakpoint),
    BreakpointsForFile(BreakpointsForFile),

    UpdateRepository(UpdateRepository),
    RemoveRepository(RemoveRepository),

    GetDocumentSymbols(GetDocumentSymbols),
    GetDocumentSymbolsResponse(GetDocumentSymbolsResponse),

    LoadCommitDiff(LoadCommitDiff),
    LoadCommitDiffResponse(LoadCommitDiffResponse),

    StopLanguageServers(StopLanguageServers),

    LspExtRunnables(LspExtRunnables),
    LspExtRunnablesResponse(LspExtRunnablesResponse),

    GetDebugAdapterBinary(GetDebugAdapterBinary),
    DebugAdapterBinary(DebugAdapterBinary),
    RunDebugLocators(RunDebugLocators),
    DebugRequest(DebugRequest),

    LspExtGoToParentModule(LspExtGoToParentModule),
    LspExtGoToParentModuleResponse(LspExtGoToParentModuleResponse),
    LspExtCancelFlycheck(LspExtCancelFlycheck),
    LspExtRunFlycheck(LspExtRunFlycheck),
    LspExtClearFlycheck(LspExtClearFlycheck),

    LogToDebugConsole(LogToDebugConsole),

    GetDocumentDiagnostics(GetDocumentDiagnostics),
    GetDocumentDiagnosticsResponse(GetDocumentDiagnosticsResponse),
    PullWorkspaceDiagnostics(PullWorkspaceDiagnostics),

    GetDocumentColor(GetDocumentColor),
    GetDocumentColorResponse(GetDocumentColorResponse),
    GetColorPresentation(GetColorPresentation),
    GetColorPresentationResponse(GetColorPresentationResponse),

    Stash(Stash),
    StashPop(StashPop),

    GetDefaultBranch(GetDefaultBranch),
    GetDefaultBranchResponse(GetDefaultBranchResponse),

    GetCrashFiles(GetCrashFiles),
    GetCrashFilesResponse(GetCrashFilesResponse),

    GitClone(GitClone),
    GitCloneResponse(GitCloneResponse),

    LspQuery(LspQuery),
    LspQueryResponse(LspQueryResponse),
    ToggleLspLogs(ToggleLspLogs),

    UpdateUserSettings(UpdateUserSettings),

    GetProcesses(GetProcesses),
    GetProcessesResponse(GetProcessesResponse),

    ResolveToolchain(ResolveToolchain),
    ResolveToolchainResponse(ResolveToolchainResponse),

    StashDrop(StashDrop),
    StashApply(StashApply),

    GitRenameBranch(GitRenameBranch),

    RemoteStarted(RemoteStarted),

    GetDirectoryEnvironment(GetDirectoryEnvironment),
    DirectoryEnvironment(DirectoryEnvironment),

    GetTreeDiff(GetTreeDiff),
    GetTreeDiffResponse(GetTreeDiffResponse),

    GetBlobContent(GetBlobContent),
    GetBlobContentResponse(GetBlobContentResponse),

    GitWorktreesResponse(GitWorktreesResponse),
    GitGetWorktrees(GitGetWorktrees),
    GitCreateWorktree(GitCreateWorktree),

    OpenImageByPath(OpenImageByPath),
    OpenImageResponse(OpenImageResponse),
    CreateImageForPeer(CreateImageForPeer),


    GitFileHistory(GitFileHistory),
    GitFileHistoryResponse(GitFileHistoryResponse),

    RunGitHook(RunGitHook),

    GitDeleteBranch(GitDeleteBranch),

    GitCreateRemote(GitCreateRemote),
    GitRemoveRemote(GitRemoveRemote),
    RescanDirectory(RescanDirectory),
    RescanDirectoryResponse(RescanDirectoryResponse),

    GitCommitHistory(GitCommitHistory),
    GitCommitHistoryResponse(GitCommitHistoryResponse),
}
