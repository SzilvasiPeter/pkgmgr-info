#![forbid(unsafe_code)]
use std::fs;
use std::io::{self, Error, ErrorKind};

const DISTRO_TO_PKG_MANAGER: [(&str, PackageManager); 10] = [
    ("debian", PackageManager::Apt),
    ("ubuntu", PackageManager::Apt),
    ("linuxmint", PackageManager::Apt),
    ("fedora", PackageManager::Dnf),
    ("redhat", PackageManager::Dnf),
    ("centos", PackageManager::Dnf),
    ("arch", PackageManager::Pacman),
    ("manjaro", PackageManager::Pacman),
    ("garuda", PackageManager::Pacman),
    ("alpine", PackageManager::Apk),
];

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum PackageManager {
    Apt,
    Dnf,
    Pacman,
    Apk,
}

impl PackageManager {
    #[must_use]
    pub const fn name(&self) -> &'static str {
        match self {
            Self::Apt => "apt",
            Self::Dnf => "dnf",
            Self::Pacman => "pacman",
            Self::Apk => "apk",
        }
    }
}

/// Detects the package manager by reading `/etc/os-release`.
///
/// # Errors
///
/// * `InvalidData` if the file does not contain an `ID=` entry.
/// * `InvalidInput` if the discovered ID is not in the supported list.
/// * Other `io::Error` variants propagated from `fs::read_to_string`.
pub fn detect_package_manager() -> io::Result<PackageManager> {
    let content = fs::read_to_string("/etc/os-release")?;
    detect_from_content(&content)
}

fn detect_from_content(content: &str) -> io::Result<PackageManager> {
    let id = content
        .lines()
        .find_map(|line| line.strip_prefix("ID=").map(|v| v.trim().trim_matches('"')))
        .ok_or_else(|| Error::new(ErrorKind::InvalidData, "missing /etc/os-release ID entry"))?;

    DISTRO_TO_PKG_MANAGER
        .iter()
        .find(|(candidate, _)| *candidate == id)
        .map(|(_, manager)| *manager)
        .ok_or_else(|| Error::new(ErrorKind::InvalidInput, "unknown package manager"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_pacakge_managers() {
        let cases = [
            ("ubuntu", "apt"),
            ("fedora", "dnf"),
            ("arch", "pacman"),
            ("alpine", "apk"),
        ];

        for (id, expected) in cases {
            let sample = format!("NAME=Foo\nID={id}\n");
            let pm = detect_from_content(&sample).expect("should match");
            assert_eq!(pm.name(), expected);
        }
    }

    #[test]
    fn rejects_unknown_id() {
        let sample = "ID=unknown\n";
        let err = detect_from_content(sample).unwrap_err();
        assert_eq!(err.kind(), ErrorKind::InvalidInput);
    }

    #[test]
    fn rejects_missing_id() {
        let sample = "NAME=Foo\n";
        let err = detect_from_content(sample).unwrap_err();
        assert_eq!(err.kind(), ErrorKind::InvalidData);
    }
}
