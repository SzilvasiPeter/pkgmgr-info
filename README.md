# pkgmgr-info
![coverage](https://img.shields.io/endpoint?url=https://szilvasipeter.github.io/pkgmgr-info/coverage/badge.json)
Rust crate and CLI that reports the system package manager information.

## Supported package managers
- [apk](https://wiki.alpinelinux.org/wiki/Alpine_Package_Keeper)
- [apt](https://wiki.debian.org/Teams/Apt)
- [dnf](https://dnf.readthedocs.io/)
- [pacman](https://wiki.archlinux.org/title/Pacman)
- [portage](https://wiki.gentoo.org/wiki/Portage)
- [zypper](https://en.opensuse.org/Zypper)

## Usage
```rust
use pkgmgr_info::PackageManager;
use std::io;

fn main() -> io::Result<()> {
    let pkg_manager = PackageManager::detect()?;
    let pkg_name = pkg_manager.name();
    let pkg_count = pkg_manager.package_count()?;
    println!("{pkg_count} ({pkg_name})");
    Ok(())
}
```

## Running
```bash
cargo run --release --bin pkgmgrinfo
```
