use crate::constant_pool::values::{Class, StringValue};
use crate::constant_pool::{ConstantPool, ConstantPoolInfo};
use crate::raw_java_class::RawJavaClass;
use std::collections::hash_map::Entry;
use std::collections::{HashMap, HashSet};
use std::fmt::{Display, Formatter};
use std::path::{Path, PathBuf};

mod constant_pool;
pub mod error;
pub(crate) mod raw_java_class;
mod structures;
pub(crate) mod utility;
use crate::utility::match_as;
use structures::class::JavaClass;
pub use structures::*;

/// Parses java classes from `.class` files. Produces a [`JavaClass`][crate::JavaClass] if successful.
#[derive(Debug, Default)]
pub struct JavaClassParser {
    class_path: HashSet<PathBuf>,
    cache: HashMap<PathBuf, JavaClass>,
}

impl JavaClassParser {
    /// Parses a java class by file type
    pub fn parse_file<P: AsRef<Path>>(path: P) -> Result<JavaClass, error::Error> {
        let bytes = std::fs::read(path)?;
        let raw_class = raw_java_class::parse_class_file_bytes(&bytes)?;
        Ok(JavaClass::new(raw_class))
    }

    /// Creates a new java class parser with a given classpath
    pub fn new<S: AsRef<str>>(classpath: S) -> Self {
        Self {
            class_path: classpath
                .as_ref()
                .split(";")
                .map(|s| PathBuf::from(s))
                .collect(),
            ..Default::default()
        }
    }

    /// Finds a class based on a fully qualified path.
    ///
    /// For example, if the given classpath contains some directory `output`
    /// ```text
    /// output/
    /// \-- com/
    ///     \-- example/
    ///         \-- Square.class
    ///
    /// ```
    /// Attempting to look up a class with fully qualified path `com/example/Square` would
    /// result in the `output/com/example/Square.class` file being parsed. This also works
    /// if a file on the classpath is a jar file.
    ///
    pub fn find<P: AsRef<Path>>(&mut self, path: P) -> Result<JavaClass, error::Error> {
        match self.cache.entry(path.as_ref().to_path_buf()) {
            Entry::Occupied(o) => Ok(o.get().clone()),
            Entry::Vacant(v) => {
                todo!()
            }
        }
    }
}

impl<P: AsRef<Path>> FromIterator<P> for JavaClassParser {
    fn from_iter<T: IntoIterator<Item = P>>(iter: T) -> Self {
        Self {
            class_path: iter.into_iter().map(|p| p.as_ref().to_path_buf()).collect(),
            ..Default::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{HasAttributes, JavaClassParser};
    use std::env;
    use std::path::PathBuf;

    #[test]
    fn parse_class() {
        let file = PathBuf::new()
            .join(env::var("OUT_DIR").unwrap())
            .join("com/example/Square.class");

        let class = JavaClassParser::parse_file(file).unwrap();
        assert_eq!(
            class.this(),
            "com/example/Square",
            "couldn't get name from {:#?}",
            class
        );
        assert_eq!(
            class.super_name(),
            "com/example/Rectangle",
            "couldn't get name from {:#?}",
            class
        );

        println!("{:#}", class);

        println!("methods = {:#?}", class.methods());
        println!("attributes = {:#?}", class.attributes().collect::<Vec<_>>())
    }
}
