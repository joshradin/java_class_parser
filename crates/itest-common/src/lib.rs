use std::path::{Path, PathBuf};

/// Gets the generated jar file
pub fn jar_file() -> PathBuf {
    let out_dir = option_env!("OUT_DIR").expect("OUT_DIR not set");
    Path::new(&out_dir).join("java.jar")
}

/// Gets the path to the generated classes directory
pub fn classes() -> PathBuf {
    let out_dir = option_env!("OUT_DIR").expect("OUT_DIR not set");
    Path::new(&out_dir).join("classes")
}

#[cfg(test)]
mod tests {
    use crate::{classes, jar_file};

    #[test]
    fn jar_file_exists() {
        assert!(
            jar_file().exists(),
            "jar file at path {:?} doesn't exist",
            jar_file()
        );
    }

    #[test]
    fn classes_is_dir() {
        assert!(
            classes().exists(),
            "classes directory at path {:?} doesn't exist",
            classes()
        );
        assert!(
            classes().is_dir(),
            "classes path {:?} doesn't point to a directory",
            classes()
        );
    }
}
