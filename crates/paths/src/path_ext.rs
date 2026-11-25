use crate::locations::home_dir;
use itertools::Itertools;
use std::path::{Path, PathBuf};

pub trait PathExt {
    /// Compacts a given file path by replacing the user's home directory
    /// prefix with a tilde (`~`).
    ///
    /// # Returns
    ///
    /// * A `PathBuf` containing the compacted file path. If the input path
    ///   does not have the user's home directory prefix, or if we are not on
    ///   Linux or macOS, the original path is returned unchanged.
    fn compact(&self) -> PathBuf;

    /// Returns a file's extension or, if the file is hidden, its name without the leading dot
    fn extension_or_hidden_file_name(&self) -> Option<&str>;

    fn try_from_bytes<'a>(bytes: &'a [u8]) -> anyhow::Result<Self>
    where
        Self: From<&'a Path>,
    {
        cfg_select! {
            unix => {
                use std::{ffi::OsStr, os::unix::prelude::OsStrExt};
                Ok(Self::from(Path::new(OsStr::from_bytes(bytes))))
            }

            windows => {
                use anyhow::Context;
                use tendril::fmt::{Format, WTF8};
                use std::ffi::OsStr;
                WTF8::validate(bytes)
                    .then(|| {
                        // Safety: bytes are valid WTF-8 sequence.
                        Self::from(Path::new(unsafe { OsStr::from_encoded_bytes_unchecked(bytes) }))
                    })
                    .with_context(|| format!("Invalid WTF-8 sequence: {bytes:?}"))
            }
        }
    }

    /// Converts a local path to one that can be used inside of WSL.
    /// Returns `None` if the path cannot be converted into a WSL one (network share).
    fn local_to_wsl(&self) -> Option<PathBuf>;

    /// Returns a file's "full" joined collection of extensions, in the case where a file does not
    /// just have a singular extension but instead has multiple (e.g File.tar.gz, Component.stories.tsx)
    ///
    /// Will provide back the extensions joined together such as tar.gz or stories.tsx
    fn multiple_extensions(&self) -> Option<String>;
}

impl<T: AsRef<Path>> PathExt for T {
    fn compact(&self) -> PathBuf {
        if cfg!(any(target_os = "linux", target_os = "freebsd")) || cfg!(target_os = "macos") {
            match self.as_ref().strip_prefix(home_dir()) {
                Ok(relative_path) => {
                    let mut shortened_path = PathBuf::new();
                    shortened_path.push("~");
                    shortened_path.push(relative_path);
                    shortened_path
                }
                Err(_) => self.as_ref().to_path_buf(),
            }
        } else {
            self.as_ref().to_path_buf()
        }
    }

    fn extension_or_hidden_file_name(&self) -> Option<&str> {
        let path = self.as_ref();
        let file_name = path.file_name()?.to_str()?;
        if file_name.starts_with('.') {
            return file_name.strip_prefix('.');
        }

        path.extension()
            .and_then(|e| e.to_str())
            .or_else(|| path.file_stem()?.to_str())
    }

    fn local_to_wsl(&self) -> Option<PathBuf> {
        // quite sketchy to convert this back to path at the end, but a lot of functions only accept paths
        // todo: ideally rework them..?
        let mut new_path = std::ffi::OsString::new();
        for component in self.as_ref().components() {
            match component {
                std::path::Component::Prefix(prefix) => {
                    let drive_letter = prefix.as_os_str().to_string_lossy().to_lowercase();
                    let drive_letter = drive_letter.strip_suffix(':')?;

                    new_path.push(format!("/mnt/{}", drive_letter));
                }
                std::path::Component::RootDir => {}
                std::path::Component::CurDir => {
                    new_path.push("/.");
                }
                std::path::Component::ParentDir => {
                    new_path.push("/..");
                }
                std::path::Component::Normal(os_str) => {
                    new_path.push("/");
                    new_path.push(os_str);
                }
            }
        }

        Some(new_path.into())
    }

    fn multiple_extensions(&self) -> Option<String> {
        let path = self.as_ref();
        let file_name = path.file_name()?.to_str()?;

        let parts: Vec<&str> = file_name
            .split('.')
            // Skip the part with the file name extension
            .skip(1)
            .collect();

        if parts.len() < 2 {
            return None;
        }

        Some(parts.into_iter().join("."))
    }
}
