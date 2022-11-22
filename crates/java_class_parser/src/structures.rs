mod signatures;

use crate::attributes::Attribute;
pub use class::*;
pub use class_entries::*;
pub use signatures::*;

pub use fully_qualified_name::*;

pub mod attributes;
mod class;
mod class_entries;
mod fully_qualified_name;

/// Objects which implement this trait can be queried for their attributes.
pub trait HasAttributes {
    /// The iterator that attributes are returned in
    type Iter<'a>: Iterator<Item = Attribute<'a>>
    where
        Self: 'a;

    /// Gets the attributes associated with this value.
    fn attributes<'a>(&'a self) -> Self::Iter<'a>;

    /// Attempts to get an attribute by attribute name
    fn get_attribute(&self, name: &str) -> Option<Attribute> {
        self.attributes()
            .find(|att: &Attribute| att.attribute_name() == name)
    }
}
