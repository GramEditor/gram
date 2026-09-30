use collections::HashMap;
use settings::DapSettingsContent;

#[derive(Default, Debug, Clone)]
pub struct DapSettings {
    pub binary: DapBinary,
    pub args: Option<Vec<String>>,
    pub env: Option<HashMap<String, String>>,
    pub ignore_system_version: bool,
    pub allow_binary_download: bool,
    pub enable_auto_updates: bool,
}

impl From<DapSettingsContent> for DapSettings {
    fn from(content: DapSettingsContent) -> Self {
        DapSettings {
            binary: content.binary.map_or_else(|| DapBinary::Default, DapBinary::Custom),
            args: content.args,
            env: content.env,
            ignore_system_version: content.ignore_system_version.unwrap_or(false),
            allow_binary_download: content.allow_binary_download.unwrap_or(false),
            enable_auto_updates: content.enable_auto_updates.unwrap_or(false),
        }
    }
}

#[derive(Default, Debug, Clone)]
pub enum DapBinary {
    #[default]
    Default,
    Custom(String),
}
