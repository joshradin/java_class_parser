use std::env;
use std::path::Path;

#[test]
fn parse_class() {
    let file_path = Path::new(&env::var("CARGO_TARGET_TMPDIR").unwrap());
}
