#![forbid(unsafe_code)]
use pkgmgr_info::detect_package_manager;
use std::io;

fn main() -> io::Result<()> {
    let package_manager = detect_package_manager()?;
    println!("{}", package_manager.name());
    Ok(())
}
