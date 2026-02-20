#![forbid(unsafe_code)]
use pkgmgr_info::PackageManager;
use std::io;

fn main() -> io::Result<()> {
    let pm = PackageManager::detect()?;
    let name = pm.name();
    let count = pm.package_count()?;
    println!("{count} ({name})");
    Ok(())
}
