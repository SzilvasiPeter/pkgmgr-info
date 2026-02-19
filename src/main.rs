#![forbid(unsafe_code)]
use pkgmgr_info::detect_package_manager;
use std::io;

fn main() -> io::Result<()> {
    let pkg_manager = detect_package_manager()?;
    let pkg_count = pkg_manager.package_count()?;
    println!("{pkg_count} ({})", pkg_manager.name());
    Ok(())
}
