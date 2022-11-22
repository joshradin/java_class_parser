use std::path::{Path, PathBuf};

/// Gets the generated jar file
pub fn jar_file() -> PathBuf {
    let out_dir = option_env!("OUT_DIR").expect("OUT_DIR not set");
    Path::new(&out_dir).join("java.jar")
}

#[cfg(test)]
mod tests {
    use crate::jar_file;

    #[test]
    fn jar_file_exists() {
        assert!(jar_file().exists(), "jar file at path {:?} doesn't exist", jar_file());
    }
}