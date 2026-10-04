use serde::{Deserialize, Serialize};

#[derive(Default, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct UpdateInviteInfo {
    pub url: String,
    pub count: u32,
}

#[derive(Default, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ShutdownRemoteServer {}

#[derive(Default, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Toast {
    pub project_id: u64,
    pub notification_id: String,
    pub message: String,
}

#[derive(Default, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct HideToast {
    pub project_id: u64,
    pub notification_id: String,
}

#[derive(Default, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct OpenServerSettings {
    pub project_id: u64,
}

#[derive(Default, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct GetCrashFiles {}

#[derive(Default, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct GetCrashFilesResponse {
    pub crashes: Vec<CrashReport>,
}

#[derive(Default, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct CrashReport {
    pub metadata: String,
    pub minidump_contents: Vec<u8>,
}

#[derive(Default, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Extension {
    pub id: String,
    pub version: String,
    pub dev: bool,
}

#[derive(Default, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct SyncExtensions {
    pub extensions: Vec<Extension>,
}

#[derive(Default, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct SyncExtensionsResponse {
    pub tmp_dir: String,
    pub missing_extensions: Vec<Extension>,
}

#[derive(Default, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct InstallExtension {
    pub extension: Extension,
    pub tmp_dir: String,
}

#[derive(Default, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct AskPassRequest {
    pub project_id: u64,
    pub repository_id: u64,
    pub askpass_id: u64,
    pub prompt: String,
}

#[derive(Default, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct AskPassResponse {
    pub response: String,
}
