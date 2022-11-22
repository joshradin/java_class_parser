use std::path::Path;
use java_class_parser::inheritance::inspect;
use java_class_parser::JavaClassParser;

#[test]
fn parse_jar() {
    let parser = JavaClassParser::from(itest_common::jar_file());
    let class = parser
        .find("com/example/Square")
        .expect("couldn't get square");
    let super_class = parser
        .find_super(&class)
        .expect("Square should have a super class that's on the classpath");
    assert_eq!(super_class.this(), "com/example/Rectangle");
    assert!(
        matches!(parser.find_super(&super_class), Err(_)),
        "Rectangle should have no available super class"
    );

    let inheritance = inspect(&class, &parser).expect("couldn't create graph");
    let parents = inheritance
        .inherits(class.this())
        .expect("couldn't get parents")
        .into_iter()
        .map(|(class, _)| class.this().to_fqname_buf())
        .collect::<Vec<_>>()
        ;
    assert_eq!(parents, ["com/example/Rectangle", "com/example/Shape"]);
}

#[test]
fn get_java_install() {
    let java_home = java_locator::locate_java_home().unwrap();
    println!("java_home: {:?}", Path::new(&java_home));
}