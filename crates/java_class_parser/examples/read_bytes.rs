use java_class_parser::parse_bytes;
use std::fs;

fn main() {
    let classes = itest_common::classes();
    println!("{classes:?}");
    let bytes = fs::read(classes.join("com/example/Square.class")).expect("couldn't read");

    let parsed = parse_bytes(&bytes[..]).expect("couldn't parse");

    println!("{:#}", parsed);
}
