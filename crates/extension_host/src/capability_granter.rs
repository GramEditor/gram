use std::sync::Arc;

use anyhow::{Result, bail};
use dap::settings::DapSettings;
use extension::{ExtensionCapability, ExtensionManifest};
use lsp::LanguageServerBinaryOptions;
use parking_lot::RwLock;
use url::Url;

#[derive(Default, Debug, Clone)]
pub struct BinaryOptions {
    pub allow_path_lookup: bool,
    pub allow_binary_download: bool,
    pub enable_auto_updates: bool,
}

impl From<&LanguageServerBinaryOptions> for BinaryOptions {
    fn from(item: &LanguageServerBinaryOptions) -> Self {
        Self {
            allow_path_lookup: item.allow_path_lookup,
            allow_binary_download: item.allow_binary_download,
            enable_auto_updates: item.enable_auto_updates,
        }
    }
}

impl From<&DapSettings> for BinaryOptions {
    fn from(item: &DapSettings) -> Self {
        Self {
            allow_path_lookup: !item.ignore_system_version,
            allow_binary_download: item.allow_binary_download,
            enable_auto_updates: item.enable_auto_updates,
        }
    }
}

pub struct CapabilityGranter {
    granted_capabilities: Vec<ExtensionCapability>,
    manifest: Arc<ExtensionManifest>,
    binary_options: RwLock<BinaryOptions>,
}

impl CapabilityGranter {
    pub fn new(granted_capabilities: Vec<ExtensionCapability>, manifest: Arc<ExtensionManifest>) -> Self {
        Self {
            granted_capabilities,
            manifest,
            binary_options: RwLock::new(BinaryOptions::default()),
        }
    }

    pub fn set_binary_options(&self, binary_options: &BinaryOptions) {
        let mut w = self.binary_options.write();
        *w = binary_options.clone();
    }

    pub fn grant_exec(&self, desired_command: &str, desired_args: &[impl AsRef<str> + std::fmt::Debug]) -> Result<()> {
        if !self.binary_options.read().allow_path_lookup {
            bail!("path lookup not allowed for {desired_command} {desired_args:?}");
        }

        self.manifest.allow_exec(desired_command, desired_args)?;

        let is_allowed = self.granted_capabilities.iter().any(|capability| match capability {
            ExtensionCapability::ProcessExec(capability) => capability.allows(desired_command, desired_args),
            _ => false,
        });

        if !is_allowed {
            bail!(
                "capability for process:exec {desired_command} {desired_args:?} is not granted by the extension host",
            );
        }

        Ok(())
    }

    pub fn grant_download_file(&self, desired_url: &Url) -> Result<()> {
        if !self.binary_options.read().allow_binary_download {
            bail!("binary download not allowed for {desired_url}");
        }
        if !self.binary_options.read().enable_auto_updates {
            bail!("auto updates not allowed for {desired_url}");
        }

        let is_allowed = self.granted_capabilities.iter().any(|capability| match capability {
            ExtensionCapability::DownloadFile(capability) => capability.allows(desired_url),
            _ => false,
        });

        if !is_allowed {
            bail!("capability for download_file {desired_url} is not granted by the extension host",);
        }

        Ok(())
    }

    pub fn grant_npm_install_package(&self, package_name: &str) -> Result<()> {
        if !self.binary_options.read().allow_binary_download {
            bail!("binary download not allowed for {package_name}");
        }
        if !self.binary_options.read().enable_auto_updates {
            bail!("auto updates not allowed for {package_name}");
        }

        let is_allowed = self.granted_capabilities.iter().any(|capability| match capability {
            ExtensionCapability::NpmInstallPackage(capability) => capability.allows(package_name),
            _ => false,
        });

        if !is_allowed {
            bail!("capability for npm:install {package_name} is not granted by the extension host",);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use extension::{ProcessExecCapability, SchemaVersion};

    use super::*;

    fn extension_manifest() -> ExtensionManifest {
        ExtensionManifest {
            id: "test".into(),
            name: "Test".to_string(),
            version: "1.0.0".into(),
            schema_version: SchemaVersion::ZERO,
            description: None,
            repository: None,
            authors: vec![],
            lib: Default::default(),
            themes: vec![],
            icon_themes: vec![],
            languages: vec![],
            grammars: BTreeMap::default(),
            language_servers: BTreeMap::default(),
            snippets: None,
            capabilities: vec![],
            debug_adapters: Default::default(),
            debug_locators: Default::default(),
        }
    }

    #[test]
    fn test_grant_exec() {
        let manifest = Arc::new(ExtensionManifest {
            capabilities: vec![ExtensionCapability::ProcessExec(ProcessExecCapability {
                command: "ls".to_string(),
                args: vec!["-la".to_string()],
            })],
            ..extension_manifest()
        });

        // It returns an error when the extension host has no granted capabilities.
        let granter = CapabilityGranter::new(Vec::new(), manifest.clone());
        assert!(granter.grant_exec("ls", &["-la"]).is_err());

        // It succeeds when the extension host has the exact capability.
        let granter = CapabilityGranter::new(
            vec![ExtensionCapability::ProcessExec(ProcessExecCapability {
                command: "ls".to_string(),
                args: vec!["-la".to_string()],
            })],
            manifest.clone(),
        );
        assert!(granter.grant_exec("ls", &["-la"]).is_ok());

        // It succeeds when the extension host has a wildcard capability.
        let granter = CapabilityGranter::new(
            vec![ExtensionCapability::ProcessExec(ProcessExecCapability {
                command: "*".to_string(),
                args: vec!["**".to_string()],
            })],
            manifest,
        );
        assert!(granter.grant_exec("ls", &["-la"]).is_ok());
    }
}
