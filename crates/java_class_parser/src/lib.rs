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

#![cfg_attr(feature = "strict", strict_mode)]
#![cfg_attr(strict_mode, deny(unused))]
#![deny(rustdoc::broken_intra_doc_links)]
#![deny(missing_docs)]

use crate::constant_pool::ConstantPoolInfo;
use std::cell::RefCell;

use std::collections::hash_map::Entry;
use std::collections::{HashMap, HashSet};
use std::fs::File;

use std::io::Read;
use std::path::{Path, PathBuf};
use zip::result::ZipError;
use zip::ZipArchive;

mod constant_pool;
pub mod error;
pub mod inheritance;
pub(crate) mod raw_java_class;
mod structures;
pub(crate) mod utility;

use crate::error::{Error, ErrorKind};
pub use structures::*;
use structures::{AsFullyQualifiedName, FQName};

/// Parses java classes from `.class` files. Produces a [`JavaClass`][crate::JavaClass] if successful.
#[derive(Debug, Default)]
pub struct JavaClassParser {
    class_path: HashSet<PathBuf>,
    cache: RefCell<HashMap<FQNameBuf, JavaClass>>,
    open_zips: RefCell<HashMap<PathBuf, ZipArchive<File>>>,
}

impl JavaClassParser {
    /// Parses a java class by file type
    pub fn parse_file<P: AsRef<Path>>(path: P) -> Result<JavaClass, Error> {
        let bytes = std::fs::read(path)?;
        let raw_class = raw_java_class::parse_class_file_bytes(&bytes)?;
        Ok(JavaClass::new(raw_class))
    }

    /// Creates a new java class parser with a given classpath.
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
    pub fn find<P: AsFullyQualifiedName + ?Sized>(&self, path: &P) -> Result<JavaClass, Error> {
        let fcq = path.as_fcq();
        if !self.cache.borrow().contains_key(fcq) {
            let class = self.find_class(fcq)?;
            self.cache.borrow_mut().insert(fcq.to_fqname_buf(), class);
        }
        Ok(self.cache.borrow()[fcq].clone())
    }

    /// Tries to find the super class of a java class on the classpath
    pub fn find_super(&self, class: &JavaClass) -> Result<JavaClass, Error> {
        let super_class = class.super_name();
        self.find(super_class)
    }

    /// Finds a list of interfaces that are available on the classpath
    pub fn find_interfaces(&self, class: &JavaClass) -> Result<Vec<JavaClass>, Error> {
        class
            .interfaces()
            .into_iter()
            .filter_map(|class| match self.find(class) {
                o @ Ok(_) => Some(o),
                Err(e) => match e.kind() {
                    ErrorKind::NoClassFound(_) => None,
                    _ => Some(Err(e)),
                },
            })
            .collect()
    }

    /// Gets the classpath of the parser
    pub fn classpath(&self) -> impl Iterator<Item = &Path> {
        self.class_path.iter().map(|p| p.as_ref())
    }

    /// scans through the classpath to find a file. In terms of complexity,
    /// directories are easiest.
    fn find_class(&self, path: &FQName) -> Result<JavaClass, Error> {
        let cp = self
            .classpath()
            .into_iter()
            .map(|s| s.to_owned())
            .collect::<Vec<_>>();
        for entry in cp {
            if let Some(found) = self.find_class_in_entry(&entry, path)? {
                return Ok(found);
            }
        }
        Err(Error::from(ErrorKind::NoClassFound(path.to_fqname_buf())))
    }

    /// find a file in a classpath entry. Returns Ok(Some()) if found, Ok(None) if not, and an error
    /// if an error occurred.
    fn find_class_in_entry(
        &self,
        entry: &Path,
        fq_name: &FQName,
    ) -> Result<Option<JavaClass>, Error> {
        if entry.is_file() {
            match entry.extension().and_then(|s| s.to_str()) {
                Some("class") => {
                    let parsed = parse_file(entry)?;
                    if parsed.this() == fq_name {
                        Ok(Some(parsed))
                    } else {
                        Ok(None)
                    }
                }
                Some("jar") => {
                    let mut open_zips = self.open_zips.borrow_mut();
                    let zip_archive = match open_zips.entry(entry.to_path_buf()) {
                        Entry::Occupied(o) => o.into_mut(),
                        Entry::Vacant(vac) => {
                            let file = File::open(entry)?;
                            vac.insert(ZipArchive::new(file)?)
                        }
                    };
                    let name = fq_name.as_path().with_extension("class");
                    let result = match zip_archive.by_name(&name.to_string_lossy()) {
                        Ok(archived) => parse_bytes(archived).map(|class| Some(class)),
                        Err(ZipError::FileNotFound) => Ok(None),
                        Err(e) => Err(e.into()),
                    };
                    result
                }
                _ => Err(ErrorKind::UnsupportedEntry(entry.to_path_buf()).into()),
            }
        } else if entry.is_dir() {
            let full_path = entry.join(fq_name).with_extension("class");
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

impl<P: AsRef<Path>> From<P> for JavaClassParser {
    fn from(p: P) -> Self {
        Self::from_iter([p])
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
