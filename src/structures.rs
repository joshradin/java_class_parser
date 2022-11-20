mod signatures;

use crate::attributes::Attribute;
pub use class::*;
pub use class_entries::*;
pub use signatures::*;
use std::fmt::{Debug, Display, Formatter};
use std::path::{Path, PathBuf};

pub mod attributes;
mod class;
mod class_entries;

/// A fully qualified name is a set of identifiers seperated by either `/` or `.`
#[derive(Default, Eq, PartialEq, Hash, Clone)]
pub struct FullyQualifiedName<'a> {
    fcq: &'a str,
}

impl<'a> FullyQualifiedName<'a> {
    /// Gets the fully qualified name as a path
    pub fn as_path(&self) -> PathBuf {
        PathBuf::from_iter(
            self.fcq.split("/")
        )
    }
}

impl<'a> From<&'a Path> for FullyQualifiedName<'a> {
    fn from(path: &'a Path) -> Self {
        FullyQualifiedName {
            fcq: path.to_str().unwrap(),
        }
    }
}

impl<S: AsRef<str>> PartialEq<S> for FullyQualifiedName<'_> {
    fn eq(&self, other: &S) -> bool {
        self.fcq == other.as_ref()
    }
}

impl<'a> FullyQualifiedName<'a> {
    /// Create a new fully qualified name from a string
    pub fn new(fcq: &'a str) -> Self {
        Self { fcq }
    }
}

impl Debug for FullyQualifiedName<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(self.fcq, f)
    }
}

impl Display for FullyQualifiedName<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(self.fcq, f)
    }
}

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
