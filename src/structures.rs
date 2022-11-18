mod signatures;

pub use signatures::*;
use std::fmt::{Debug, Display, Formatter};
mod class_entries;
use crate::attributes::Attribute;
pub use class_entries::*;

pub mod attributes;
pub mod class;

/// A fully qualified name is a set of identifiers seperated by either `/` or `.`
#[derive(Default, Eq, PartialEq, Hash)]
pub struct FullyQualifiedName<'a> {
    fcq: &'a str,
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
    type Iter<'a>: Iterator<Item = Attribute<'a>>
    where
        Self: 'a;

    /// Gets the attributes associated with this value.
    fn attributes<'a>(&'a self) -> Self::Iter<'a>;
}
