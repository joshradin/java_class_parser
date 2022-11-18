use std::env;
use std::ffi::OsStr;
use std::fs::{DirEntry, FileType};
use std::process::Command;
use walkdir::WalkDir;

fn main() {
    /// only compile for integration tests
    println!("cargo:rerun-if-changed=src/java");
    println!("cargo:rerun-if-changed=build.rs");

    let dir = WalkDir::new("src/java");

    let mut java_files = vec![];

    for entry in dir {
        let entry = entry.unwrap();
        println!("cargo:warn=entry={:?}", entry);
        if entry.file_type().is_file() && entry.path().extension() == Some(OsStr::new("java")) {
            java_files.push(entry.path().to_path_buf());
        }
    }

    let mut cmd = Command::new("javac");
    println!("cargo:warn=compiling {:?}", java_files);
    let res = cmd
        .args(["-d", &std::env::var("OUT_DIR").unwrap()])
        .args(java_files)
        .output()
        .expect("couldn't successfully run javac");
    if !res.status.success() {
        println!("cargo:warn={:#?} did not successfully pass", cmd);
        panic!("javac failed: {}", String::from_utf8_lossy(&res.stderr))
    }
}
