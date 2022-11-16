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
        let signature = java_class
            .get_descriptor(field_info.descriptor_index)
            .expect("should be a valid descriptor");
        Self { name, signature }
    }

    /// Gets the name of the field
    pub fn name(&self) -> &'a str {
        self.name
    }

    /// Gets the signature (AKA the type) of the field.
    pub fn signature(&self) -> &Signature<'a> {
        &self.signature
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
        let signature = java_class
            .get_descriptor(method_info.descriptor_index)
            .expect("should be a valid descriptor");
        Self { name, signature }
    }

    /// Gets the name of the method
    pub fn name(&self) -> &'a str {
        self.name
    }

    /// Gets the signature of the method. This includes the arguments and return type.
    pub fn signature(&self) -> MethodSignature {
        let Signature::Method {
            args, ret_type
        } = &self.signature else {
            panic!("Signature of a method should only ever be a Method");
        };

        MethodSignature {
            ret_type: &**ret_type,
            args: &**args
        }
    }
}

/// A method signature contains both the signature of the return type and of it's argument types.
#[derive(Debug)]
pub struct MethodSignature<'a> {
    /// The return type of the method
    pub ret_type: &'a Signature<'a>,
    /// The args of the method
    pub args: &'a [Signature<'a>]
}