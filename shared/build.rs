// SPDX-License-Identifier: Apache-2.0

// Copyright 2024 src_resources
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

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