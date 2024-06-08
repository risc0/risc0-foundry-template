use std::os::unix::fs::symlink;
use std::path::PathBuf;

const LINK_PATH: &str = "../lib//risc0-ethereum";

fn main() {
    // Rerun this build script if and only if it changed.
    println!("cargo::rerun-if-changed=build.rs");
    println!("cargo::rerun-if-changed={}", LINK_PATH);

    let ctx = cargo::GlobalContext::default().unwrap();
    let workspace =
        cargo::core::Workspace::new(&PathBuf::from("./Cargo.toml").canonicalize().unwrap(), &ctx)
            .unwrap();
    let (packages, resolve) = cargo::ops::resolve_ws(&workspace).unwrap();
    let package_id = resolve
        .iter()
        .find(|id| id.name() == "risc0-ethereum-contracts")
        .unwrap();
    let package = packages.get_one(package_id).unwrap();
    let _checksum = resolve.checksums().get(&package_id).cloned();
    let package_root = package.root();

    let link_target = package_root.parent().unwrap().canonicalize().unwrap();
    match std::fs::read_link(LINK_PATH) {
        Ok(cur_target) => {
            if cur_target.canonicalize().unwrap() != link_target {
                std::fs::remove_file(LINK_PATH).unwrap();
                symlink(link_target, LINK_PATH).unwrap();
            }
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            symlink(link_target, LINK_PATH).unwrap()
        }
        Err(e) => panic!("failed to check for risc0-ethereum symlink: {e}"),
    }
}
