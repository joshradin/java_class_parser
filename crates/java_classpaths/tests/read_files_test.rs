use std::io::Read;
use java_classpaths::Classpath;
use std::path::{Path, PathBuf};
use itest_common::jar_file;

fn test_resources() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/resources")
        .canonicalize()
        .expect("could not get canonical path")
}

#[test]
fn read_file() {
    let cp = Classpath::from(test_resources());
    let mut text_file = cp.get("TEST_FILE.txt")
        .expect("should be on classpath")
        .expect("should be readable");

    let mut buffer = String::new();
    text_file.read_to_string(&mut buffer).expect("should be readable");
    let trimmed = buffer.trim();
    assert_eq!(trimmed, "Hello, World!")
}

#[test]
fn read_jar_file() {
    let cp = Classpath::from(jar_file());
    let mut text_file = cp.get("TEST_FILE.txt")
                          .expect("should be on classpath")
                          .expect("should be readable");

    let mut buffer = String::new();
    text_file.read_to_string(&mut buffer).expect("should be readable");
    let trimmed = buffer.trim();
    assert_eq!(trimmed, "Hello, World!")
}

