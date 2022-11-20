use std::path::{Path, PathBuf};

/// Gets the generated jar file
pub fn jar_file() -> PathBuf {
    let out_dir = std::env::var("OUT_DIR").expect("OUT_DIR not set");
    Path::new(&out_dir).join("java.jar")
}
