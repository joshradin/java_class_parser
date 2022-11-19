#![allow(warnings)]


use std::path::{Path, PathBuf};
use std::process::Command;
use fs_extra::dir::CopyOptions;

static JAVA_PATH: &str = "./java";

fn main() {
    // only compile for integration tests
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=java/");


    let ref out_dir = PathBuf::from(std::env::var("OUT_DIR").unwrap());
    let ref java_output_path = out_dir.join("java");
    std::fs::remove_dir_all(java_output_path).expect("couldn't remove dir");
    let mut options = CopyOptions::new();
    options.copy_inside = true;
    fs_extra::copy_items(
        &[Path::new(JAVA_PATH)],
        &java_output_path,
        &options
    ).expect("could not copy");

    // create jar
    let gradle_assemble = Command::new("./gradlew")
        .current_dir(&java_output_path)
        .arg("assemble")
        .spawn()
        .expect("could not run gradle wrapper")
        .wait()
        .expect("did not finish");
    if !gradle_assemble.success() {
        panic!("failed to run gradlew assemble")
    }

    std::fs::copy(
        java_output_path.join("build/libs/java.jar"),
        out_dir.join("java.jar")
    ).expect("couldn't copy");

}
