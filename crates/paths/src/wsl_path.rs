use std::path::Path;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WslPath {
    pub distro: String,

    // the reason this is an OsString and not any of the path types is that it needs to
    // represent a unix path (with '/' separators) on windows. `from_path` does this by
    // manually constructing it from the path components of a given windows path.
    pub path: std::ffi::OsString,
}

impl WslPath {
    pub fn from_path<P: AsRef<Path>>(path: P) -> Option<WslPath> {
        if cfg!(not(target_os = "windows")) {
            return None;
        }
        use std::{
            ffi::OsString,
            path::{Component, Prefix},
        };

        let mut components = path.as_ref().components();
        let Some(Component::Prefix(prefix)) = components.next() else {
            return None;
        };
        let (server, distro) = match prefix.kind() {
            Prefix::UNC(server, distro) => (server, distro),
            Prefix::VerbatimUNC(server, distro) => (server, distro),
            _ => return None,
        };
        let Some(Component::RootDir) = components.next() else {
            return None;
        };

        let server_str = server.to_string_lossy();
        if server_str == "wsl.localhost" || server_str == "wsl$" {
            let mut result = OsString::from("");
            for c in components {
                use Component::*;
                match c {
                    Prefix(p) => unreachable!("got {p:?}, but already stripped prefix"),
                    RootDir => unreachable!("got root dir, but already stripped root"),
                    CurDir => continue,
                    ParentDir => result.push("/.."),
                    Normal(s) => {
                        result.push("/");
                        result.push(s);
                    }
                }
            }
            if result.is_empty() {
                result.push("/");
            }
            Some(WslPath {
                distro: distro.to_string_lossy().to_string(),
                path: result,
            })
        } else {
            None
        }
    }
}
