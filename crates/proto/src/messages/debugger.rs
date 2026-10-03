use std::collections::HashMap;
use serde::{Serialize, Deserialize};

use crate::messages::task::SpawnInTerminal;
use crate::messages::buffer::Anchor;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Copy)]
pub enum BreakpointState {
    Enabled,
    Disabled,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Breakpoint {
    pub position: Anchor,
    pub state: BreakpointState,
    pub message: Option<String>,
    pub condition: Option<String>,
    pub hit_condition: Option<String>,
    pub session_state: HashMap<u64, BreakpointSessionState>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct BreakpointSessionState {
    pub id: u64,
    pub verified: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct BreakpointsForFile {
    pub project_id: u64,
    pub path: String,
    pub breakpoints: Vec<Breakpoint>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ToggleBreakpoint {
    pub project_id: u64,
    pub path: String,
    pub breakpoint: Breakpoint,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Copy)]
pub enum DapThreadStatus {
    Running,
    Stopped,
    Exited,
    Ended,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Copy)]
pub enum VariablesArgumentsFilter {
    Indexed,
    Named,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ValueFormat {
    pub hex: Option<bool>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct VariablesRequest {
    pub project_id: u64,
    pub client_id: u64,
    pub variables_reference: u64,
    pub filter: Option<VariablesArgumentsFilter>,
    pub start: Option<u64>,
    pub count: Option<u64>,
    pub format: Option<ValueFormat>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Copy)]
pub enum SteppingGranularity {
    Statement,
    Line,
    Instruction,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct DapLocationsRequest {
    pub project_id: u64,
    pub session_id: u64,
    pub location_reference: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct DapLocationsResponse {
    pub source: DapSource,
    pub line: u64,
    pub column: Option<u64>,
    pub end_line: Option<u64>,
    pub end_column: Option<u64>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Copy)]
pub enum DapEvaluateContext {
    Repl,
    Watch,
    Hover,
    Clipboard,
    EvaluateVariables,
    EvaluateUnknown,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct DapEvaluateRequest {
    pub project_id: u64,
    pub client_id: u64,
    pub expression: String,
    pub frame_id: Option<u64>,
    pub context: Option<DapEvaluateContext>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct DapEvaluateResponse {
    pub result: String,
    pub evaluate_type: Option<String>,
    pub variable_reference: u64,
    pub named_variables: Option<u64>,
    pub indexed_variables: Option<u64>,
    pub memory_reference: Option<String>,
}


#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct DapCompletionRequest {
    pub project_id: u64,
    pub client_id: u64,
    pub query: String,
    pub frame_id: Option<u64>,
    pub line: Option<u64>,
    pub column: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Copy)]
pub enum DapCompletionItemType {
    Method,
    Function,
    Constructor,
    Field,
    Variable,
    Class,
    Interface,
    Module,
    Property,
    Unit,
    Value,
    Enum,
    Keyword,
    Snippet,
    Text,
    Color,
    CompletionItemFile,
    Reference,
    Customcolor,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct DapCompletionItem {
    pub label: String,
    pub text: Option<String>,
    pub sort_text: Option<String>,
    pub detail: Option<String>,
    pub typ: Option<DapCompletionItemType>,
    pub start: Option<u64>,
    pub length: Option<u64>,
    pub selection_start: Option<u64>,
    pub selection_length: Option<u64>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct DapCompletionResponse {
    pub client_id: u64,
    pub completions: Vec<DapCompletionItem>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct DapScopesRequest {
    pub project_id: u64,
    pub client_id: u64,
    pub stack_frame_id: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct DapScopesResponse {
    pub scopes: Vec<DapScope>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct DapSetVariableValueRequest {
    pub project_id: u64,
    pub client_id: u64,
    pub name: String,
    pub value: String,
    pub variables_reference: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct DapSetVariableValueResponse {
    pub client_id: u64,
    pub value: String,
    pub variable_type: Option<String>,
    pub variables_reference: Option<u64>,
    pub named_variables: Option<u64>,
    pub indexed_variables: Option<u64>,
    pub memory_reference: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct DapPauseRequest {
    pub project_id: u64,
    pub client_id: u64,
    pub thread_id: i64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct DapDisconnectRequest {
    pub project_id: u64,
    pub client_id: u64,
    pub restart: Option<bool>,
    pub terminate_debuggee: Option<bool>,
    pub suspend_debuggee: Option<bool>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct DapTerminateThreadsRequest {
    pub project_id: u64,
    pub client_id: u64,
    pub thread_ids: Vec<i64>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct DapThreadsRequest {
    pub project_id: u64,
    pub client_id: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct DapThreadsResponse {
    pub threads: Vec<DapThread>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct DapTerminateRequest {
    pub project_id: u64,
    pub client_id: u64,
    pub restart: Option<bool>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct DapRestartRequest {
    pub project_id: u64,
    pub client_id: u64,
    pub raw_args: Vec<u8>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct DapRestartStackFrameRequest {
    pub project_id: u64,
    pub client_id: u64,
    pub stack_frame_id: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ToggleIgnoreBreakpoints {
    pub project_id: u64,
    pub session_id: u32,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct IgnoreBreakpointState {
    pub project_id: u64,
    pub session_id: u64,
    pub ignore: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct DapNextRequest {
    pub project_id: u64,
    pub client_id: u64,
    pub thread_id: i64,
    pub single_thread: Option<bool>,
    pub granularity: Option<SteppingGranularity>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct DapStepInRequest {
    pub project_id: u64,
    pub client_id: u64,
    pub thread_id: i64,
    pub target_id: Option<u64>,
    pub single_thread: Option<bool>,
    pub granularity: Option<SteppingGranularity>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct DapStepOutRequest {
    pub project_id: u64,
    pub client_id: u64,
    pub thread_id: i64,
    pub single_thread: Option<bool>,
    pub granularity: Option<SteppingGranularity>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct DapStepBackRequest {
    pub project_id: u64,
    pub client_id: u64,
    pub thread_id: i64,
    pub single_thread: Option<bool>,
    pub granularity: Option<SteppingGranularity>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct DapContinueRequest {
    pub project_id: u64,
    pub client_id: u64,
    pub thread_id: i64,
    pub single_thread: Option<bool>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct DapContinueResponse {
    pub client_id: u64,
    pub all_threads_continued: Option<bool>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct DapModulesRequest {
    pub project_id: u64,
    pub client_id: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct DapModulesResponse {
    pub client_id: u64,
    pub modules: Vec<DapModule>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct DapLoadedSourcesRequest {
    pub project_id: u64,
    pub client_id: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct DapLoadedSourcesResponse {
    pub client_id: u64,
    pub sources: Vec<DapSource>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct DapStackTraceRequest {
    pub project_id: u64,
    pub client_id: u64,
    pub thread_id: i64,
    pub start_frame: Option<u64>,
    pub stack_trace_levels: Option<u64>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct DapStackTraceResponse {
    pub frames: Vec<DapStackFrame>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct DapStackFrame {
    pub id: u64,
    pub name: String,
    pub source: Option<DapSource>,
    pub line: u64,
    pub column: u64,
    pub end_line: Option<u64>,
    pub end_column: Option<u64>,
    pub can_restart: Option<bool>,
    pub instruction_pointer_reference: Option<String>,
    pub module_id: Option<DapModuleId>,
    pub presentation_hint: Option<DapStackPresentationHint>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct DebuggerLoadedSourceList {
    pub client_id: u64,
    pub sources: Vec<DapSource>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct DapVariables {
    pub client_id: u64,
    pub variables: Vec<DapVariable>,
}

// Remote Debugging: Dap Types
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct DapVariable {
    pub name: String,
    pub value: String,
    pub dap_type: Option<String>,
    pub evaluate_name: Option<String>,
    pub variables_reference: u64,
    pub named_variables: Option<u64>,
    pub indexed_variables: Option<u64>,
    pub memory_reference: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct DapThread {
    pub id: i64,
    pub name: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct DapScope {
    pub name: String,
    pub presentation_hint: Option<DapScopePresentationHint>,
    pub variables_reference: u64,
    pub named_variables: Option<u64>,
    pub indexed_variables: Option<u64>,
    pub expensive: bool,
    pub source: Option<DapSource>,
    pub line: Option<u64>,
    pub column: Option<u64>,
    pub end_line: Option<u64>,
    pub end_column: Option<u64>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct DapSource {
    pub name: Option<String>,
    pub path: Option<String>,
    pub source_reference: Option<u64>,
    pub presentation_hint: Option<DapSourcePresentationHint>,
    pub origin: Option<String>,
    pub sources: Vec<DapSource>,
    pub adapter_data: Option<Vec<u8>>,
    pub checksums: Vec<DapChecksum>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Copy)]
pub enum DapOutputCategory {
    ConsoleOutput,
    Important,
    Stdout,
    Stderr,
    Unknown,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Copy)]
pub enum DapOutputEventGroup {
    Start,
    StartCollapsed,
    End,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct DapOutputEvent {
    pub output: String,
    pub category: Option<DapOutputCategory>,
    pub variables_reference: Option<u64>,
    pub group: Option<DapOutputEventGroup>,
    pub source: Option<DapSource>,
    pub line: Option<u32>,
    pub column: Option<u32>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Copy)]
pub enum DapChecksumAlgorithm {
    Unspecified,
    Md5,
    Sha1,
    Sha256,
    Timestamp,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct DapChecksum {
    pub algorithm: DapChecksumAlgorithm,
    pub checksum: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Copy)]
pub enum DapScopePresentationHint {
    Arguments,
    Locals,
    Registers,
    ReturnValue,
    ScopeUnknown,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Copy)]
pub enum DapSourcePresentationHint {
    SourceNormal,
    Emphasize,
    Deemphasize,
    SourceUnknown,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Copy)]
pub enum DapStackPresentationHint {
    StackNormal,
    Label,
    Subtle,
    StackUnknown,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct DapModule {
    pub id: DapModuleId,
    pub name: String,
    pub path: Option<String>,
    pub is_optimized: Option<bool>,
    pub is_user_code: Option<bool>,
    pub version: Option<String>,
    pub symbol_status: Option<String>,
    pub symbol_file_path: Option<String>,
    pub date_time_stamp: Option<String>,
    pub address_range: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct DebugTaskDefinition {
    pub adapter: String,
    pub label: String,
    pub config: String,
    pub tcp_connection: Option<TcpHost>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct TcpHost {
    pub port: Option<u32>,
    pub host: Option<String>,
    pub timeout: Option<u64>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct DebugLaunchRequest {
    pub program: String,
    pub cwd: Option<String>,
    pub args: Vec<String>,
    pub env: HashMap<String, String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct DebugAttachRequest {
    pub process_id: u32,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum DapModuleId {
    Number(u32),
    String(String),
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct GetDebugAdapterBinary {
    pub project_id: u64,
    pub session_id: u64,
    pub definition: DebugTaskDefinition,
    pub worktree_id: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Copy)]
pub enum DebugAdapterBinaryLaunchType {
    Attach,
    Launch,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct DebugAdapterBinary {
    pub command: Option<String>,
    pub arguments: Vec<String>,
    pub envs: HashMap<String, String>,
    pub cwd: Option<String>,
    pub connection: Option<TcpHost>,
    pub configuration: String,
    pub launch_type: DebugAdapterBinaryLaunchType,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct RunDebugLocators {
    pub project_id: u64,
    pub build_command: SpawnInTerminal,
    pub locator: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum DebugRequest {
    Launch(DebugLaunchRequest),
    Attach(DebugAttachRequest),
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct DebugScenario {
    pub label: String,
    pub adapter: String,
    pub request: DebugRequest,
    pub connection: Option<TcpHost>,
    pub stop_on_entry: Option<bool>,
    pub configuration: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct LogToDebugConsole {
    pub project_id: u64,
    pub session_id: u64,
    pub message: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct GetProcesses {
    pub project_id: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct GetProcessesResponse {
    pub processes: Vec<ProcessInfo>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ProcessInfo {
    pub pid: u32,
    pub name: String,
    pub command: Vec<String>,
}
