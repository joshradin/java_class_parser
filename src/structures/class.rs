use crate::attributes::{Attribute, ResolveAttributeError};
use crate::constant_pool::values::{Class, StringValue};
use crate::constant_pool::{ConstantPool, ConstantPoolInfo};
use crate::raw_java_class::RawJavaClass;
use crate::utility::match_as;
use crate::{Field, FullyQualifiedName, HasAttributes, Method, Signature};
use nom::bytes;
use std::collections::HashMap;
use std::fmt::{Display, Formatter};

/// A java class
#[derive(Debug, Clone)]
pub struct JavaClass(RawJavaClass);

impl JavaClass {
    pub(crate) fn new(class: RawJavaClass) -> Self {
        Self(class)
    }

    pub(crate) fn raw_constant_pool(&self) -> &ConstantPool {
        &self.0.constant_pool
    }

    /// gets the info at a given constant pool location
    pub(crate) fn get_at_index(&self, index: u16) -> Option<&ConstantPoolInfo> {
        self.raw_constant_pool().get(index)
    }

    /// Gets a string at an index, or if possible follow indexes
    pub(crate) fn get_string(&self, index: u16) -> Option<&str> {
        match self.raw_constant_pool().get(index)? {
            ConstantPoolInfo::String(StringValue { string_index }) => {
                self.get_string(*string_index)
            }
            ConstantPoolInfo::Utf8(s) => Some(s.as_ref()),
            _ => None,
        }
    }

    pub(crate) fn get_class_info(&self, index: u16) -> Option<&Class> {
        if let Some(ConstantPoolInfo::Class(class)) = self.get_at_index(index) {
            Some(class)
        } else {
            None
        }
    }

    /// get a descriptor at an index
    pub(crate) fn get_descriptor(&self, index: u16) -> Option<Signature> {
        self.get_at_index(index)
            .and_then(|info| match_as!(utf; ConstantPoolInfo::Utf8(utf) = info))
            .map(|s| {
                Signature::from_str(s.as_ref())
                    .unwrap_or_else(|e| panic!("{} is invalid as signature: {}", s, e))
            })
    }

    pub(crate) fn create_attribute<'a>(
        &'a self,
        name_index: u16,
        info: &'a [u8],
    ) -> Result<Attribute<'a>, ResolveAttributeError> {
        self.get_string(name_index)
            .ok_or(ResolveAttributeError::new("<unknown>"))
            .and_then(|name| Attribute::new(self, name, info))
    }

    /// Gets this class's name
    pub fn this(&self) -> FullyQualifiedName {
        self.get_class_info(self.0.this_class)
            .and_then(|Class { name_index }| self.get_string(*name_index))
            .map(|s| FullyQualifiedName::new(s))
            .unwrap_or_else(|| {
                let info = self.get_at_index(self.0.this_class);
                panic!("{:?} could not be treated as a string", info);
            })
    }

    /// Gets the super class's name of this class
    pub fn super_name(&self) -> FullyQualifiedName {
        self.get_class_info(self.0.super_class)
            .and_then(|Class { name_index }| self.get_string(*name_index))
            .map(|s| FullyQualifiedName::new(s))
            .unwrap_or_else(|| {
                let info = self.get_at_index(self.0.this_class);
                panic!("{:?} could not be treated as a string", info);
            })
    }

    /// Gets the names of this interfaces that this class implements
    pub fn interfaces(&self) -> Vec<FullyQualifiedName> {
        self.0
            .interfaces
            .iter()
            .map(|index| {
                let Class { name_index } =
                    self.get_class_info(*index).expect("no class info found");
                self.get_string(*name_index).expect("couldn't get string")
            })
            .map(|s| FullyQualifiedName::new(s))
            .collect()
    }

    /// Gets the fields declared in this class.
    pub fn fields(&self) -> Vec<Field> {
        self.0.fields.iter().map(|f| Field::new(f, &self)).collect()
    }

    /// Gets the methods declared in this class.
    pub fn methods(&self) -> Vec<Method> {
        self.0
            .methods
            .iter()
            .map(|f| Method::new(f, &self))
            .collect()
    }
}

impl Display for JavaClass {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let attributes: Vec<_> = self.attributes().collect();
        f.debug_struct("JavaClass")
            .field("this", &self.this())
            .field("super", &self.super_name())
            .field("interfaces", &self.interfaces())
            .field(
                "attributes",
                &attributes
                    .iter()
                    .map(|att| (att.attribute_name(), att.kind()))
                    .collect::<HashMap<_, _>>(),
            )
            .finish()
    }
}

impl HasAttributes for JavaClass {
    type Iter<'a>  = <Vec<Attribute<'a>> as IntoIterator>::IntoIter where Self: 'a;

    fn attributes<'a>(&'a self) -> Self::Iter<'a> {
        let mut output = vec![];
        for raw_info in self.0.attributes.iter() {
            let bytes = &*raw_info.info;
            output.extend(self.create_attribute(raw_info.attribute_name_index, bytes));
        }
        output.into_iter()
    }
}
