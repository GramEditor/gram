/// LSP support for typst
/// based on https://github.com/zed-extensions/typst
use anyhow::{Result, anyhow};
use async_trait::async_trait;
use gpui::AsyncApp;
use http_client::github::{AssetKind, GitHubLspBinaryVersion, latest_github_release};
use http_client::github_download::download_server_binary;
use language::{LanguageServerName, LspAdapter, LspAdapterDelegate, LspInstaller, Toolchain};
use lsp::LanguageServerBinary;
use std::path::PathBuf;
use util::fs::{make_file_executable, remove_matching};

use crate::helpers::{find_cached_server_binary, verify_metadata, with_exe, write_metadata};

pub struct TypstLspAdapter;

#[cfg(target_os = "macos")]
impl TypstLspAdapter {
    const GITHUB_ASSET_KIND: AssetKind = AssetKind::TarGz;
    const OS_NAME: &str = "apple-darwin";
}

#[cfg(target_os = "linux")]
impl TypstLspAdapter {
    const GITHUB_ASSET_KIND: AssetKind = AssetKind::TarGz;
    const OS_NAME: &str = "unknown-linux-musl";
}

#[cfg(target_os = "windows")]
impl TypstLspAdapter {
    const GITHUB_ASSET_KIND: AssetKind = AssetKind::Zip;
    const OS_NAME: &str = "pc-windows-msvc";
}

impl TypstLspAdapter {
    const SERVER_NAME: LanguageServerName = LanguageServerName::new_static("tinymist");

    fn build_asset_base_name() -> Result<String> {
        let arch = match std::env::consts::ARCH {
            "aarch64" => "aarch64",
            "x86_64" => "x86_64",
            "x86" => "x86",
            other => return Err(anyhow!("unsupported architecture: {}", other)),
        };

        let asset_name = format!("tinymist-{}-{}", arch, Self::OS_NAME);

        Ok(asset_name)
    }
}

impl LspInstaller for TypstLspAdapter {
    type BinaryVersion = GitHubLspBinaryVersion;

    async fn check_if_user_installed(
        &self,
        delegate: &dyn LspAdapterDelegate,
        _: Option<Toolchain>,
        _: &AsyncApp,
    ) -> Option<LanguageServerBinary> {
        let path = delegate.which(with_exe("tinymist").as_ref()).await?;
        Some(LanguageServerBinary {
            path,
            arguments: vec!["lsp".into()],
            env: None,
        })
    }

    async fn fetch_latest_server_version(
        &self,
        delegate: &dyn LspAdapterDelegate,
        pre_release: bool,
        _cx: &mut AsyncApp,
    ) -> Result<GitHubLspBinaryVersion> {
        let release =
            latest_github_release("Myriad-Dreamin/tinymist", true, pre_release, delegate.http_client()).await?;

        let asset_name = format!(
            "{}.{}",
            Self::build_asset_base_name()?,
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
            name: release.tag_name.clone(),
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
        log::info!(
            "fetch_server_binary: version={:?} dir={:?}",
            version.name,
            container_dir,
        );

        let GitHubLspBinaryVersion {
            name: version_name,
            url,
            digest: expected_digest,
        } = version;

        let asset_basename = Self::build_asset_base_name()?;
        let destination_path = container_dir.join(format!("tinymist-{version_name}"));
        let server_path = destination_path.join(asset_basename).join(with_exe("tinymist"));

        log::info!("server_path={:?}", server_path);

        let binary = LanguageServerBinary {
            path: server_path.clone(),
            env: None,
            arguments: vec!["lsp".into()],
        };

        if verify_metadata(&destination_path, &server_path, &expected_digest, delegate).await {
            return Ok(binary);
        }

        download_server_binary(
            &*delegate.http_client(),
            &url,
            expected_digest.as_deref(),
            &destination_path,
            Self::GITHUB_ASSET_KIND,
        )
        .await?;

        make_file_executable(&server_path).await?;
        remove_matching(&container_dir, |path| path != destination_path).await;
        write_metadata(&destination_path, expected_digest).await?;

        Ok(binary)
    }

    async fn cached_server_binary(
        &self,
        container_dir: PathBuf,
        _: &dyn LspAdapterDelegate,
    ) -> Option<LanguageServerBinary> {
        let asset_basename = Self::build_asset_base_name().ok()?;

        match find_cached_server_binary(&container_dir, Some("tinymist-"), async |path| {
            Some(path.join(&asset_basename).join(with_exe("tinymist")))
        })
        .await
        {
            Some(path) => Some(LanguageServerBinary {
                path,
                arguments: vec!["lsp".into()],
                env: None,
            }),
            None => None,
        }
    }
}

#[async_trait(?Send)]
impl LspAdapter for TypstLspAdapter {
    fn name(&self) -> LanguageServerName {
        Self::SERVER_NAME
    }
}
