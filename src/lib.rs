#![forbid(unsafe_code)]
use std::fs;
use std::io::{self, Error, ErrorKind};
use std::process::{Command, Output};

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

    /// Returns the installed package count for the manager.
    ///
    /// # Errors
    ///
    /// Returns an error if the command fails, output is invalid UTF-8,
    /// output is empty, or the count cannot be parsed as an integer.
    pub fn package_count(&self) -> io::Result<u64> {
        self.package_count_with(run_count)
    }

    fn package_count_with(self, run: fn(&str) -> io::Result<u64>) -> io::Result<u64> {
        #[allow(clippy::literal_string_with_formatting_args)]
        match self {
            Self::Apk => run("apk info | wc -l"),
            Self::Apt => run("dpkg-query -f '${binary:Package}\\n' -W | wc -l"),
            Self::Dnf | Self::Zypper => run("rpm -qa | wc -l"),
            Self::Pacman => run("pacman -Q | wc -l"),
            Self::Portage => run("qlist -I | wc -l"),
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

fn run_cmd(cmd: &str) -> io::Result<Output> {
    Command::new("sh").arg("-c").arg(cmd).output()
}

fn run_count(cmd: &str) -> io::Result<u64> {
    run_count_with(cmd, run_cmd)
}

fn run_count_with(cmd: &str, run: fn(&str) -> io::Result<Output>) -> io::Result<u64> {
    let output = run(cmd)?;
    if !output.status.success() {
        return Err(Error::other("command failed"));
    }

    let text = std::str::from_utf8(&output.stdout)
        .map_err(|_| Error::new(ErrorKind::InvalidData, "non-utf8 output"))?;
    parse_count(text)
}

fn parse_count(text: &str) -> io::Result<u64> {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return Err(Error::new(ErrorKind::InvalidData, "empty output"));
    }

    trimmed
        .parse::<u64>()
        .map_err(|_| Error::new(ErrorKind::InvalidData, "invalid count"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::os::unix::process::ExitStatusExt;
    use std::process::ExitStatus;

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

    fn fake_run(cmd: &str) -> io::Result<u64> {
        match cmd {
            "apk info | wc -l" => Ok(10),
            "dpkg-query -f '${binary:Package}\\n' -W | wc -l" => Ok(20),
            "rpm -qa | wc -l" => Ok(30),
            "pacman -Q | wc -l" => Ok(40),
            "qlist -I | wc -l" => Ok(50),
            _ => Err(Error::new(ErrorKind::InvalidInput, "unknown cmd")),
        }
    }

    #[test]
    fn package_count_uses_expected_commands() {
        let cases = [
            (PackageManager::Apk, 10),
            (PackageManager::Apt, 20),
            (PackageManager::Dnf, 30),
            (PackageManager::Pacman, 40),
            (PackageManager::Portage, 50),
            (PackageManager::Zypper, 30),
        ];

        for (pm, expected) in cases {
            let count = pm.package_count_with(fake_run).expect("count ok");
            assert_eq!(count, expected);
        }
    }

    #[test]
    fn fake_run_rejects_unknown_command() {
        let err = fake_run("nope").unwrap_err();
        assert_eq!(err.kind(), ErrorKind::InvalidInput);
    }

    #[allow(clippy::unnecessary_wraps)]
    fn fake_output_ok(_cmd: &str) -> io::Result<Output> {
        Ok(Output {
            status: ExitStatus::from_raw(0),
            stdout: b"42\n".to_vec(),
            stderr: Vec::new(),
        })
    }

    #[allow(clippy::unnecessary_wraps)]
    fn fake_output_bad(_cmd: &str) -> io::Result<Output> {
        Ok(Output {
            status: ExitStatus::from_raw(1),
            stdout: Vec::new(),
            stderr: Vec::new(),
        })
    }

    fn fake_output_err(_cmd: &str) -> io::Result<Output> {
        Err(Error::new(ErrorKind::NotFound, "missing cmd"))
    }

    #[allow(clippy::unnecessary_wraps)]
    fn fake_output_non_utf8(_cmd: &str) -> io::Result<Output> {
        Ok(Output {
            status: ExitStatus::from_raw(0),
            stdout: vec![0xff, 0xfe, 0xfd],
            stderr: Vec::new(),
        })
    }

    #[test]
    fn run_count_with_parses_stdout() {
        let count = run_count_with("ignored", fake_output_ok).expect("count ok");
        assert_eq!(count, 42);
    }

    #[test]
    fn run_count_with_fails_on_status() {
        let err = run_count_with("ignored", fake_output_bad).unwrap_err();
        assert_eq!(err.kind(), ErrorKind::Other);
    }

    #[test]
    fn run_count_with_rejects_non_utf8() {
        let err = run_count_with("ignored", fake_output_non_utf8).unwrap_err();
        assert_eq!(err.kind(), ErrorKind::InvalidData);
    }

    #[test]
    fn run_count_with_propagates_runner_error() {
        let err = run_count_with("ignored", fake_output_err).unwrap_err();
        assert_eq!(err.kind(), ErrorKind::NotFound);
    }

    #[test]
    fn run_count_reports_missing_command_failure() {
        let err = run_count("cmd-that-should-not-exist").unwrap_err();
        assert_eq!(err.kind(), ErrorKind::Other);
    }

    #[test]
    fn parse_count_rejects_empty() {
        let err = parse_count("   ").unwrap_err();
        assert_eq!(err.kind(), ErrorKind::InvalidData);
    }

    #[test]
    fn parse_count_rejects_invalid() {
        let err = parse_count("nope").unwrap_err();
        assert_eq!(err.kind(), ErrorKind::InvalidData);
    }

    #[test]
    fn parse_count_accepts_valid() {
        let count = parse_count(" 123 ").expect("count ok");
        assert_eq!(count, 123);
    }
}
