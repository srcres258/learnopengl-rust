use std::{env, fs};
use std::path::Path;
use std::process::Command;

fn main() {
    // Generate the root path of the git repository
    let output = Command::new("git")
        .args(&["rev-parse", "--show-toplevel"])
        .output()
        .expect("Failed to execute git.");
    assert!(output.status.success(), "Process git didn't finish successfully.");
    let mut root_dir = String::from_utf8(output.stdout).unwrap();
    if root_dir.ends_with('\n') {
        let root_dir_old = root_dir.clone();
        root_dir = String::new();
        let chars: Vec<_> = root_dir_old.chars().collect();
        for (i, c) in chars.iter().enumerate() {
            if i < chars.len() - 1 {
                root_dir.push(c.clone());
            }
        }
    }
    let out_dir = env::var_os("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("git_repo_root_path.txt");
    fs::write(&dest_path, root_dir).unwrap();

    println!("cargo:rerun-if-changed=build.rs");
}