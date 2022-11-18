use crate::attributes::Attribute;
use crate::raw_java_class::{RawAttributeInfo, RawFieldInfo, RawMethodInfo};
use crate::structures::class::JavaClass;
use crate::utility::match_as;
use crate::{ConstantPoolInfo, Signature};

/// A field in a class
#[derive(Debug)]
pub struct Field<'a> {
    entry: Entry<'a>,
}

impl<'a> Field<'a> {
    pub(crate) fn new(field_info: &'a RawFieldInfo, java_class: &'a JavaClass) -> Self {
        Self {
            entry: Entry::new(
                java_class,
                field_info.name_index,
                field_info.descriptor_index,
                &field_info.attributes,
            ),
        }
    }
}

/// A field in a class
#[derive(Debug)]
pub struct Method<'a> {
    entry: Entry<'a>,
}

impl<'a> Method<'a> {
    pub(crate) fn new(method_info: &'a RawMethodInfo, java_class: &'a JavaClass) -> Self {
        Self {
            entry: Entry::new(
                java_class,
                method_info.name_index,
                method_info.descriptor_index,
                &method_info.attributes,
            ),
        }
    }
}

#[derive(Debug)]
struct Entry<'a> {
    name: &'a str,
    signature: Signature<'a>,
    attributes: Vec<Attribute<'a>>,
}

impl<'a> Entry<'a> {
    fn new(
        java_class: &'a JavaClass,
        name_index: u16,
        descriptor_index: u16,
        attributes: &'a [RawAttributeInfo],
    ) -> Self {
        let name = match_as!(name; Some(ConstantPoolInfo::Utf8(name)) = java_class.get_at_index(name_index)).expect("invalid").as_ref();
        let signature = java_class
            .get_descriptor(descriptor_index)
            .expect("should be a valid descriptor");

        let attributes = attributes
            .iter()
            .map(|s| {
                java_class
                    .create_attribute(s.attribute_name_index, &s.info)
                    .expect("couldn't create attribute")
            })
            .collect::<Vec<_>>();

        Self {
            name,
            signature,
            attributes,
        }
    }
}
