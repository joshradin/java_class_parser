use crate::raw_java_class::{RawFieldInfo, RawMethodInfo};
use crate::utility::match_as;
use crate::{ConstantPoolInfo, JavaClass, Signature};

/// A field in a class
#[derive(Debug)]
pub struct Field<'a> {
    name: &'a str,
    signature: Signature<'a>,
}

impl<'a> Field<'a> {
    pub(crate) fn new(field_info: &'a RawFieldInfo, java_class: &'a JavaClass) -> Self {
        let name = match_as!(name; Some(ConstantPoolInfo::Utf8(name)) = java_class.get_at_index(field_info.name_index)).expect("invalid").as_ref();
        let signature = java_class.get_descriptor(
            field_info.descriptor_index
        ).expect("should be a valid descriptor");
        Self {
            name,
            signature
        }
    }
}

/// A field in a class
#[derive(Debug)]
pub struct Method<'a> {
    name: &'a str,
    signature: Signature<'a>,
}

impl<'a> Method<'a> {
    pub(crate) fn new(method_info: &'a RawMethodInfo, java_class: &'a JavaClass) -> Self {
        let name = match_as!(name; Some(ConstantPoolInfo::Utf8(name)) = java_class.get_at_index(method_info.name_index)).expect("invalid").as_ref();
        let signature = java_class.get_descriptor(
            method_info.descriptor_index
        ).expect("should be a valid descriptor");
        Self {
            name,
            signature
        }
    }
}
