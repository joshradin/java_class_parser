//! # Java Class Parser
//! Provides an easy to use interface to inspect the bytecode contents of a compiled `.class` file
//! intended for use in the JVM.
//!
//! There are two main ways to parse java files
//! 1. Directly via the [`parse_file`](crate::parse_file) and [`parse_bytes`](crate::parse_bytes) functions
//! 2. Indirectly via the [`JavaClassParser`]. Using the parser, one can load an entire java classpath
//! then request classes via relative path. The java class that is used would follow the same result
//! that a classloader would find.
//!
//! This library provides a safe way of accessing constants within the a java class's constant pool.
//!
//! **In no way does this library provide mechanisms to write, compile, or modify java class files**

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
pub use structures::*;
pub mod attributes;

/// Parses java classes from `.class` files. Produces a [`JavaClass`][crate::JavaClass] if successful.
///
/// Can created with a given classpath in order to refer to classes by their fully qualified names,
/// which corresponds to their relative path within the classpath.
///
/// Supported classpath members:
/// - Direct classes (ie `../com/example/Test.class`)
/// - Directories
/// - Jar files
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
        Ok(JavaClass(raw_class))
    }

    /// Creates a new java class parser with a given classpath.
    ///
    /// # Warning
    /// The classpath always uses a `;` marker for separation. This library doesn't provide any
    /// mechanisms for checking for a valid classpath. If you want to ensure safety over paths,
    /// a java class parser can also be built over an iterator of paths.
    ///
    /// # Example
    /// ```
    /// # use java_class_parser::JavaClassParser;
    /// let parser = JavaClassParser::new("jar1.jar;jar2.jar");
    /// ```
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

    /// Finds a class based on a fully qualified path within the classpath that
    /// this parser was create with.
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

/// Parses a java class at a given path.
///
/// This is a convenience function over [`JavaClassParser::parse_file`](JavaClassParser::parse_file).
pub fn parse_file<P: AsRef<Path>>(path: &P) -> Result<JavaClass, error::Error> {
    JavaClassParser::parse_file(path)
}

/// Directly parses a buffer of bytes to produce a class.
///
/// # Error
/// Will return an error if the given byte array is not a valid java class
///
/// # Example
/// ```
/// let buffer = b"asdf"; // invalid
/// assert!(matches!(java_class_parser::parse_bytes(buffer), Err(_)));
/// ```
pub fn parse_bytes(bytes: &[u8]) -> Result<JavaClass, error::Error> {
    let raw_class = raw_java_class::parse_class_file_bytes(&bytes)?;
    Ok(JavaClass(raw_class))
}

/// A java class
#[derive(Debug, Clone)]
pub struct JavaClass(RawJavaClass);

impl JavaClass {
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

    fn get_class_info(&self, index: u16) -> Option<&Class> {
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
                Signature::new(s.as_ref())
                    .unwrap_or_else(|e| panic!("{} is invalid as signature: {}", s, e))
            })
    }

    /// Gets this class's name
    pub fn this(&self) -> &str {
        self.get_class_info(self.0.this_class)
            .and_then(|Class { name_index }| self.get_string(*name_index))
            .unwrap_or_else(|| {
                let info = self.get_at_index(self.0.this_class);
                panic!("{:?} could not be treated as a string", info);
            })
    }

    /// Gets the super class's name of this class
    pub fn super_name(&self) -> &str {
        self.get_class_info(self.0.super_class)
            .and_then(|Class { name_index }| self.get_string(*name_index))
            .unwrap_or_else(|| {
                let info = self.get_at_index(self.0.this_class);
                panic!("{:?} could not be treated as a string", info);
            })
    }

    /// Gets the names of this interfaces that this class implements
    pub fn interfaces(&self) -> Vec<&str> {
        self.0
            .interfaces
            .iter()
            .map(|index| {
                let Class { name_index } =
                    self.get_class_info(*index).expect("no class info found");
                self.get_string(*name_index).expect("couldn't get string")
            })
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
        f.debug_struct("JavaClass")
            .field("this", &self.this())
            .field("super", &self.super_name())
            .field("interfaces", &self.interfaces())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use crate::{JavaClassParser, MethodSignature, Signature};
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
        let methods = class.methods();
        let init = methods.iter().find(|i| i.name() == "<init>").expect("all classes should have <init>");
        let MethodSignature { ret_type, args } = init.signature();
        assert_eq!(ret_type, &Signature::Void, "constructors always have void return type");
        assert_eq!(args, &[Signature::Double]);
    }
}
