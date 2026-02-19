# pkgmgr-info
Rust crate and CLI that reports the system package manager information.

## Usage
```rust
use pkgmgr_info::detect_package_manager;

fn main() -> std::io::Result<()> {
    let pm = detect_package_manager()?;
    println!("{}", pm.name());
    let count = pm.package_count()?;
    println!("{count}");
    Ok(())
}
```

## Running
```bash
cargo run --release --bin pkgmgrinfo
```
