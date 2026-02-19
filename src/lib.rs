#![forbid(unsafe_code)]
use std::fs;
use std::io::{self, Error, ErrorKind};

const LINUX_DISTROS: [(&str, PackageManager); 8] = [
    ("alpine", PackageManager::Apk),
    ("ubuntu", PackageManager::Apt),
    ("debian", PackageManager::Apt),
    ("fedora", PackageManager::Dnf),
    ("rhel", PackageManager::Dnf),
    ("arch", PackageManager::Pacman),
    ("gentoo", PackageManager::Portage),
    ("opensuse", PackageManager::Zypper),
];

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum PackageManager {
    Apk,
    Apt,
    Dnf,
    Pacman,
    Portage,
    Zypper,
}

impl PackageManager {
    #[must_use]
    pub const fn name(&self) -> &'static str {
        match self {
            Self::Apk => "apk",
            Self::Apt => "apt",
            Self::Dnf => "dnf",
            Self::Pacman => "pacman",
            Self::Portage => "portage",
            Self::Zypper => "zypper",
        }
    }
}

/// Detects the package manager by reading `/etc/os-release`.
///
/// # Errors
///
/// * `InvalidData` if the `/etc/os-release` does not contain an `ID=` and `ID_LIKE` entry.
/// * `InvalidInput` if the discovered ID is not in the supported list.
/// * Other `io::Error` variants propagated from `fs::read_to_string`.
pub fn detect_package_manager() -> io::Result<PackageManager> {
    let os_release = fs::read_to_string("/etc/os-release")?;
    detect_from_os_release(&os_release)
}

fn detect_from_os_release(os_release: &str) -> io::Result<PackageManager> {
    let id = read_key(os_release, "ID");
    let id_like = read_key(os_release, "ID_LIKE");
    if id.is_none() && id_like.is_none() {
        return Err(Error::new(ErrorKind::InvalidData, "missing ID and ID_LIKE"));
    }

    if let Some(distro) = id
        && let Some(manager) = lookup(distro)
    {
        return Ok(manager);
    }

    if let Some(distros) = id_like {
        for distro in distros.split_ascii_whitespace() {
            if let Some(manager) = lookup(distro) {
                return Ok(manager);
            }
        }
    }

    Err(Error::new(ErrorKind::InvalidInput, "unknown pkg manager"))
}

fn lookup(id: &str) -> Option<PackageManager> {
    LINUX_DISTROS
        .iter()
        .find(|(distro, _)| *distro == id)
        .map(|(_, manager)| *manager)
}

fn read_key<'a>(os: &'a str, prefix: &str) -> Option<&'a str> {
    os.lines()
        .filter_map(|line| line.trim_start().split_once('='))
        .find(|(key, _)| *key == prefix)
        .map(|(_, val)| val.trim_matches('"'))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn supported_distro_count_matches_expected() {
        assert_eq!(LINUX_DISTROS.len(), 8);
    }

    #[test]
    fn detects_pacakge_managers_from_id() {
        let cases = [
            ("debian", "apt"),
            ("fedora", "dnf"),
            ("arch", "pacman"),
            ("alpine", "apk"),
            ("gentoo", "portage"),
        ];

        for (id, expected) in cases {
            let sample = format!("NAME=Foo\nID={id}\n");
            let pm = detect_from_os_release(&sample).expect("should match");
            assert_eq!(pm.name(), expected);
        }
    }

    #[test]
    fn detects_pacakge_managers_from_id_like() {
        let cases = [
            ("almalinux", "rhel centos fedora", "dnf"),
            ("linuxmint", "ubuntu", "apt"),
            ("manjaro", "arch", "pacman"),
            ("opensuse-tumbleweed", "opensuse suse", "zypper"),
        ];

        for (id, id_like, expected) in cases {
            let sample = format!("NAME=Foo\nID={id}\nID_LIKE={id_like}\n");
            let pm = detect_from_os_release(&sample).expect("should match");
            assert_eq!(pm.name(), expected);
        }
    }

    #[test]
    fn prefers_id_over_id_like() {
        let sample = "NAME=Foo\nID=ubuntu\nID_LIKE=debian\n";
        let pm = detect_from_os_release(sample).expect("should match");
        assert_eq!(pm.name(), "apt");
    }

    #[test]
    fn rejects_missing_id_and_id_like() {
        let sample = "NAME=Foo\n";
        let err = detect_from_os_release(sample).unwrap_err();
        assert_eq!(err.kind(), ErrorKind::InvalidData);
    }

    #[test]
    fn rejects_unknown_id_like() {
        let sample = "ID_LIKE=unknown";
        let err = detect_from_os_release(sample).unwrap_err();
        assert_eq!(err.kind(), ErrorKind::InvalidInput);
    }

    #[test]
    fn rejects_unknown_id() {
        let sample = "ID=unknown\n";
        let err = detect_from_os_release(sample).unwrap_err();
        assert_eq!(err.kind(), ErrorKind::InvalidInput);
    }
}
