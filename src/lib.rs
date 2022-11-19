//! Provides mechanisms to parse and then inspect java class files. Java classes have very specific
//! formats that can be parsed. However, because the format relies on constantly referring back to
//! a constant pool, information can be hard to actually parse. This library provides easier mechanisms
//! for digesting this info.
//!
//! There are three main entrance points to the api: [`parse_file`][0], [`parse_bytes`][1], and
//! [`JavaClassParser`][2]
//!
//! [0]: parse_file
//! [1]: parse_bytes
//! [2]: JavaClassParser
//!
//! # Example
//! If you want to inspect many classes, it may be better to create the parser using a classpath,
//! then finding classes by their fully qualified path.
//! ```no_run
//! # use java_class_parser::JavaClassParser;
//! let mut parser = JavaClassParser::new("classes.jar");
//! let class1 = parser.find("com.example.TestClass").expect("couldn't find class");
//! let class2 = parser.find("com.example.OtherTestClass").expect("couldn't find class");
//!
//! ```

#![deny(rustdoc::broken_intra_doc_links)]
#![deny(unused)]
#![deny(missing_docs)]

use crate::constant_pool::ConstantPoolInfo;

use std::collections::{HashMap, HashSet};
use std::collections::hash_map::Entry;
use std::fs::File;

use std::io::Read;
use std::path::{Path, PathBuf};
use zip::result::{ZipError};
use zip::ZipArchive;

mod constant_pool;
pub mod error;
pub(crate) mod raw_java_class;
mod structures;
pub(crate) mod utility;

use crate::error::{Error, ErrorKind};
pub use structures::*;

/// Parses java classes from `.class` files. Produces a [`JavaClass`][crate::JavaClass] if successful.
#[derive(Debug, Default)]
pub struct JavaClassParser {
    class_path: HashSet<PathBuf>,
    cache: HashMap<PathBuf, JavaClass>,
    open_zips: HashMap<PathBuf, ZipArchive<File>>
}

impl JavaClassParser {
    /// Parses a java class by file type
    pub fn parse_file<P: AsRef<Path>>(path: P) -> Result<JavaClass, Error> {
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
    ///         \-- Square.java
    ///
    /// ```
    /// Attempting to look up a class with fully qualified path `com/example/Square` would
    /// result in the `output/com/example/Square.java` file being parsed. This also works
    /// if a file on the classpath is a jar file.
    ///
    pub fn find<P: AsRef<Path>>(&mut self, path: P) -> Result<&JavaClass, Error> {
        if !self.cache.contains_key(path.as_ref()) {
            let class = self.find_class(path.as_ref())?;
            self.cache.insert(path.as_ref().to_path_buf(), class);
        }
        Ok(&self.cache[path.as_ref()])
    }

    /// Gets the classpath of the parser
    pub fn classpath(&self) -> impl Iterator<Item=&Path> {
        self.class_path.iter().map(|p| p.as_ref())
    }

    /// scans through the classpath to find a file. In terms of complexity,
    /// directories are easiest.
    fn find_class(&mut self, path: &Path) -> Result<JavaClass, Error> {
        let cp = self.classpath().into_iter().map(|s| s.to_owned()).collect::<Vec<_>>();
        for entry in cp {
            if let Some(found) = self.find_class_in_entry(&entry, path)? {
                return Ok(found);
            }
        }
        Err(Error::from(ErrorKind::NoClassFound(path.to_path_buf())))
    }

    /// find a file in a classpath entry. Returns Ok(Some()) if found, Ok(None) if not, and an error
    /// if an error occurred.
    fn find_class_in_entry(&mut self, entry: &Path, path: &Path) -> Result<Option<JavaClass>, Error> {
        if entry.is_file() {
            match entry.extension().and_then(|s| s.to_str()) {
                Some("class") => {
                    let parsed = parse_file(entry)?;
                    if parsed.this() == FullyQualifiedName::from(path) {
                        Ok(Some(parsed))
                    } else {
                        Ok(None)
                    }
                }
                Some("jar") => {
                    let zip_archive =
                        match self.open_zips.entry(entry.to_path_buf()) {
                            Entry::Occupied(o) => {o.into_mut()}
                            Entry::Vacant(vac) => {
                                let file = File::open(entry)?;
                                vac.insert(ZipArchive::new(file)?)
                            }
                        };

                    println!("working with jar with files: {:?}", zip_archive.file_names().collect::<Vec<_>>());
                    let name = path.with_extension("class");
                    match zip_archive.by_name(&name.to_string_lossy()) {
                        Ok(archived) => {
                            parse_bytes(archived).map(|class| Some(class))
                        }
                        Err(ZipError::FileNotFound) => {
                            Ok(None)
                        }
                        Err(e) => {
                            Err(e.into())
                        }
                    }

                }
                _ => Err(ErrorKind::UnsupportedEntry(entry.to_path_buf()).into())
            }
        } else if entry.is_dir() {
            let full_path = entry.join(path).with_extension("class");
            if full_path.exists() {
                let read = parse_bytes(File::open(full_path)?)?;
                Ok(Some(read))
            } else {
                Ok(None)
            }
        } else {
            Err(ErrorKind::UnsupportedEntry(entry.to_path_buf()).into())
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

/// Parse bytes into a java class.
///
/// # Error
/// Will return an error if the byte stream does not resolve to a valid java class
pub fn parse_bytes<R: Read>(mut read: R) -> Result<JavaClass, Error> {
    let mut buffer = vec![];
    read.read_to_end(&mut buffer)?;

    raw_java_class::parse_class_file_bytes(&buffer[..]).map(JavaClass::new)
}

/// Parses the contents of a file into a java class
///
/// # Error
/// Will return an error if the file does not exist, or the contents of the file doesn't resolve
/// to a valid java class.
///
/// > This is a wrapper over the [`JavaClassParser::parse_file`](JavaClassParser::parse_file) method.
pub fn parse_file<P: AsRef<Path>>(path: P) -> Result<JavaClass, Error> {
    JavaClassParser::parse_file(path)
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
            .join("java.jar");

        let mut parser = JavaClassParser::from_iter([file]);
        let class = parser.find("com/example/Square").unwrap();
        assert_eq!(
            class.this(),
            "com/example/Square",
            "invalid name {:?} from {:#?}",
            class.this(),
            class
        );
        assert_eq!(
            class.super_name(),
            "com/example/Rectangle",
            "wrong super name from {:#?}",
            class
        );

        println!("{:#}", class);

        println!("methods = {:#?}", class.methods());
        println!("attributes = {:#?}", class.attributes().collect::<Vec<_>>())
    }
}
