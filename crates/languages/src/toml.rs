use anyhow::{Result, anyhow};
use async_trait::async_trait;
use gpui::AsyncApp;
use http_client::github::{AssetKind, GitHubLspBinaryVersion, latest_github_release};
use http_client::github_download::download_server_binary;
use language::{LanguageServerName, LspAdapter, LspAdapterDelegate, LspInstaller, Toolchain};
use lsp::LanguageServerBinary;
use std::path::PathBuf;
use util::fs::make_file_executable;

use crate::helpers::{find_cached_server_binary, verify_metadata, write_metadata};

pub struct TomlLspAdapter;

#[cfg(target_os = "macos")]
impl TomlLspAdapter {
    const GITHUB_ASSET_KIND: AssetKind = AssetKind::TarGz;
    const OS_NAME: &str = "apple-darwin";
}

#[cfg(target_os = "linux")]
impl TomlLspAdapter {
    const GITHUB_ASSET_KIND: AssetKind = AssetKind::TarGz;
    const OS_NAME: &str = "unknown-linux-musl";
}

#[cfg(target_os = "windows")]
impl TomlLspAdapter {
    const GITHUB_ASSET_KIND: AssetKind = AssetKind::Zip;
    const OS_NAME: &str = "pc-windows-msvc";
}

impl TomlLspAdapter {
    const SERVER_NAME: LanguageServerName = LanguageServerName::new_static("tombi");
}

impl LspInstaller for TomlLspAdapter {
    type BinaryVersion = GitHubLspBinaryVersion;

    async fn check_if_user_installed(
        &self,
        delegate: &dyn LspAdapterDelegate,
        _: Option<Toolchain>,
        _: &AsyncApp,
    ) -> Option<LanguageServerBinary> {
        let path = delegate.which("tombi".as_ref()).await?;
        Some(LanguageServerBinary {
            path,
            arguments: vec!["lsp".into(), "stdio".into()],
            env: None,
        })
    }

    async fn fetch_latest_server_version(
        &self,
        delegate: &dyn LspAdapterDelegate,
        _pre_release: bool,
        _cx: &mut AsyncApp,
    ) -> Result<GitHubLspBinaryVersion> {
        let release = latest_github_release("tombi-toml/tombi", false, false, delegate.http_client()).await?;

        let binary_version = release
            .tag_name
            .strip_prefix("v")
            .unwrap_or(&release.tag_name)
            .to_owned();

        let arch = match std::env::consts::ARCH {
            "aarch64" => "aarch64",
            "x86" => "x86",
            "x86_64" => "x86_64",
            other => return Err(anyhow!("unsupported architecture: {}", other)),
        };

        let asset_name = format!(
            "tombi-cli-{}-{}-{}.{}",
            binary_version,
            arch,
            Self::OS_NAME,
            match Self::GITHUB_ASSET_KIND {
                AssetKind::TarGz => "tar.gz",
                AssetKind::Zip => "zip",
                _ => unreachable!(),
            }
        );

        let asset = release
            .assets
            .iter()
            .find(|a| a.name == asset_name)
            .ok_or_else(|| anyhow!("no matching asset found for {}", asset_name))?;

        Ok(GitHubLspBinaryVersion {
            name: binary_version,
            url: asset.browser_download_url.clone(),
            digest: None,
        })
    }

    async fn fetch_server_binary(
        &self,
        version: GitHubLspBinaryVersion,
        container_dir: PathBuf,
        delegate: &dyn LspAdapterDelegate,
    ) -> Result<LanguageServerBinary> {
        let GitHubLspBinaryVersion {
            name: version_name,
            url,
            digest: expected_digest,
        } = version;

        let arch = match std::env::consts::ARCH {
            "aarch64" => "aarch64",
            "x86" => "x86",
            "x86_64" => "x86_64",
            other => return Err(anyhow!("unsupported architecture: {}", other)),
        };

        // The archive contains a top-level directory with this name.
        let asset_dir = format!("tombi-cli-{version_name}-{}-{}", arch, Self::OS_NAME);
        let path = container_dir.join(&asset_dir).join("tombi");

        let binary = LanguageServerBinary {
            path: path.clone(),
            env: None,
            arguments: vec!["lsp".into(), "stdio".into()],
        };

        if verify_metadata(&path, &path, &expected_digest, delegate).await {
            return Ok(binary);
        }

        download_server_binary(
            &*delegate.http_client(),
            &url,
            expected_digest.as_deref(),
            &container_dir, // extract into container_dir
            Self::GITHUB_ASSET_KIND,
        )
        .await?;

        make_file_executable(&path).await?;
        write_metadata(&path, expected_digest).await?;

        Ok(binary)
    }

    async fn cached_server_binary(
        &self,
        container_dir: PathBuf,
        _: &dyn LspAdapterDelegate,
    ) -> Option<LanguageServerBinary> {
        find_cached_server_binary(&container_dir, Some("tombi-cli-"), async |path| {
            let binary = path.join("tombi");
            if binary.is_file() { Some(binary) } else { None }
        })
        .await
        .map(|path| LanguageServerBinary {
            path,
            arguments: vec!["lsp".into()],
            env: None,
        })
    }
}

#[async_trait(?Send)]
impl LspAdapter for TomlLspAdapter {
    fn name(&self) -> LanguageServerName {
        Self::SERVER_NAME
    }
}
