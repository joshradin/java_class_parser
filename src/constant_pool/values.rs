
use std::fmt;
use std::fmt::{Display, Formatter};

#[derive(Debug, Clone)]
pub struct Class {
    pub name_index: u16,
}
#[derive(Debug, Clone)]
pub struct FieldRef {
    pub class_index: u16,
    pub name_and_type_index: u16,
}
#[derive(Debug, Clone)]
pub struct MethodRef {
    pub class_index: u16,
    pub name_and_type_index: u16,
}
#[derive(Debug, Clone)]
pub struct InterfaceMethodRef {
    pub class_index: u16,
    pub name_and_type_index: u16,
}
#[derive(Debug, Clone)]
pub struct StringValue {
    pub string_index: u16,
}
#[derive(Debug, Clone)]
pub struct Integer {
    pub int: u32,
}
#[derive(Debug, Clone)]
pub struct Float {
    pub float: f32,
}
#[derive(Debug, Clone)]
pub struct Long {
    pub long: u64,
}
#[derive(Debug, Clone)]
pub struct Double {
    pub double: f64,
}
#[derive(Debug, Clone)]
pub struct NameAndType {
    pub name_index: u16,
    pub descriptor_index: u16,
}
#[derive(Debug, Clone)]
pub struct Utf8 {
    pub bytes: Box<[u8]>,
}

impl AsRef<str> for Utf8 {
    fn as_ref(&self) -> &str {
        std::str::from_utf8(&*self.bytes).expect("invalid utf8")
    }
}

impl Display for Utf8 {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let as_string = String::from_utf8_lossy(&*self.bytes);
        write!(f, "{}", as_string)
    }
}

#[derive(Debug, Clone)]
pub struct MethodHandle {
    pub reference_kind: u8,
    pub reference_index: u16,
}
#[derive(Debug, Clone)]
pub struct MethodType {
    pub descriptor_index: u16,
}
#[derive(Debug, Clone)]
pub struct InvokeDynamic {
    pub bootstrap_method_attr_index: u16,
    pub name_and_type_index: u16,
}
