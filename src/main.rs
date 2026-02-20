#![forbid(unsafe_code)]
use pkgmgr_info::PackageManager;
use std::io;

fn main() -> io::Result<()> {
    let pkg_manager = PackageManager::detect()?;
    let pkg_name = pkg_manager.name();
    let pkg_count = pkg_manager.package_count()?;
    println!("{pkg_count} ({pkg_name})");
    Ok(())
}
