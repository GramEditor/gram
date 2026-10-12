use std::str::FromStr;

use anyhow::Context;

use crate::Oid;

#[derive(Debug)]
pub struct Submodule {
    pub name: String,
    pub oid: Oid,
    pub is_dirty: bool,
}

/// Parse the output of `git submodule status <path>` and `git -C <path> status --porcelain`.
pub fn parse_submodule(submodule_status: &[u8], status: &[u8]) -> anyhow::Result<Submodule> {
    let error_context = || {
        format!(
            "Malformed submodule_status output {:?}",
            String::from_utf8_lossy(submodule_status)
        )
    };

    let oid_end = 1 + submodule_status[1..]
        .iter()
        .position(|b| *b == b' ')
        .with_context(error_context)?;
    let oid_std = std::str::from_utf8(&submodule_status[1..oid_end])?;
    let oid = Oid::from_str(oid_std)?;

    let after_oid = &submodule_status[oid_end + 1..];
    let name_end = after_oid.iter().position(|b| *b == b' ').with_context(error_context)?;
    let name_str = std::str::from_utf8(&after_oid[..name_end])?;

    Ok(Submodule {
        name: name_str.to_owned(),
        oid,
        is_dirty: status != b"",
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_submodule() {
        let submodule_status = b"+da5a6c3cbf8349a5ff5c3f0ebf9dc455dd3cc00e Letterboxinator (da5a6c3)";
        let status = b"";
        let out = parse_submodule(submodule_status, status).unwrap();
        assert_eq!(out.name, "Letterboxinator".to_owned());
        assert_eq!(
            out.oid.to_string(),
            "da5a6c3cbf8349a5ff5c3f0ebf9dc455dd3cc00e".to_owned()
        );
        assert!(!out.is_dirty);

        let submodule_status = b" a1b2510191270969c5f1246635b14dc92380584f empc (heads/main)";
        let status = b" M .gitignore";
        let out = parse_submodule(submodule_status, status).unwrap();
        assert_eq!(out.name, "empc".to_owned());
        assert_eq!(
            out.oid.to_string(),
            "a1b2510191270969c5f1246635b14dc92380584f".to_owned()
        );
        assert!(out.is_dirty);
    }
}
