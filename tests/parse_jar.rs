use java_class_parser::JavaClassParser;

mod common;

#[test]
fn parse_jar() {
    let parser = JavaClassParser::from(common::jar_file());
    let class = parser
        .find("com/example/Square")
        .expect("couldn't get square");
    let super_class = parser
        .find_super(&class)
        .expect("Square should have a super class")
        .expect("Should be in classpath");
    assert_eq!(super_class.this(), "com/example/Rectangle");
    assert!(matches!(parser.find_super(&super_class), Ok(None)), "Rectangle should have no super class");
}
