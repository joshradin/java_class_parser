//! similar to paths

use std::borrow::Borrow;
use std::ffi::OsStr;
use std::fmt::{Debug, Display, Formatter};
use std::ops::Deref;
use std::path::{Path, PathBuf};

/// Gets an object as a fully qualified path
pub trait AsFullyQualifiedName {
    fn as_fcq(&self) -> &FQName;
}

impl AsFullyQualifiedName for str {
    fn as_fcq(&self) -> &FQName {
        FQName::new(self)
    }
}

/// A fully qualified name is a set of identifiers seperated by either `/` or `.`
#[derive(Eq, PartialEq, Hash)]
#[repr(transparent)]
pub struct FQName {
    fcq: str,
}

impl FQName {
    /// Create a new fully qualified name from a string
    pub fn new<'a, S: AsRef<str> + 'a + ?Sized>(fcq: &'a S) -> &'a Self {
        unsafe {
            let fcq = fcq.as_ref();
            let fcq_ptr = fcq as *const str;
            let output = Self::new_from_ptr(fcq_ptr);
            &*output
        }
    }

    unsafe fn new_from_ptr(ptr: *const str) -> *const FQName {
        ptr as *const FQName
    }

    /// Gets the fully qualified name as a path
    pub fn as_path(&self) -> &Path {
        Path::new(&self.fcq)
    }

    pub fn to_fqname_buf(&self) -> FQNameBuf {
        FQNameBuf {
            buf: self.fcq.to_string(),
        }
    }
}

impl PartialEq<str> for FQName {
    fn eq(&self, other: &str) -> bool {
        &self.fcq == other
    }
}

impl PartialEq<&str> for FQName {
    fn eq(&self, other: &&str) -> bool {
        &self.fcq == *other
    }
}

impl PartialEq<String> for FQName {
    fn eq(&self, other: &String) -> bool {
        &self.fcq == other
    }
}

impl AsRef<Path> for FQName {
    fn as_ref(&self) -> &Path {
        self.as_path()
    }
}

impl AsRef<FQName> for FQName {
    fn as_ref(&self) -> &FQName {
        self
    }
}

impl Debug for FQName {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(&self.fcq, f)
    }
}

impl Display for FQName {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.fcq, f)
    }
}

impl ToOwned for FQName {
    type Owned = FQNameBuf;

    fn to_owned(&self) -> Self::Owned {
        self.to_fqname_buf()
    }
}
impl AsFullyQualifiedName for FQName {
    fn as_fcq(&self) -> &FQName {
        self
    }
}

/// An owned version of a fully qualified name
#[derive(Eq, PartialEq, Hash, Clone)]
pub struct FQNameBuf {
    buf: String,
}
impl Debug for FQNameBuf {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(&self.buf, f)
    }
}

impl Display for FQNameBuf {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.buf, f)
    }
}
impl Deref for FQNameBuf {
    type Target = FQName;

    fn deref(&self) -> &Self::Target {
        FQName::new(&self.buf)
    }
}

impl AsRef<FQName> for FQNameBuf {
    fn as_ref(&self) -> &FQName {
        self.borrow()
    }
}

impl Borrow<FQName> for FQNameBuf {
    fn borrow(&self) -> &FQName {
        FQName::new(&self.buf)
    }
}

impl<T: ?Sized> PartialEq<T> for FQNameBuf
where
    FQName: PartialEq<T>,
{
    fn eq(&self, other: &T) -> bool {
        self.as_ref().eq(other)
    }
}

impl AsFullyQualifiedName for FQNameBuf {
    fn as_fcq(&self) -> &FQName {
        self.as_ref()
    }
}

#[cfg(test)]
mod tests {
    use crate::structures::FQName;
    use std::path::PathBuf;

    #[test]
    fn unsafe_conversion() {
        unsafe {
            let string = "java/lang/Object";
            let ptr = string as *const str;
            let fcq = FQName::new_from_ptr(ptr);
            assert_eq!(&*fcq, string);
            assert_eq!(
                (*fcq).as_path(),
                PathBuf::from_iter(["java", "lang", "Object"])
            );

            let mut string = string.to_string();
            let ptr = string.as_ref() as *const str;
            let fcq = FQName::new_from_ptr(ptr);
            assert_eq!(&*fcq, &string);
            assert_eq!(
                (*fcq).as_path(),
                PathBuf::from_iter(["java", "lang", "Object"])
            );

            string.as_bytes_mut()[0] = b'J';
            assert_eq!(&*fcq, &string); // should effect both
        }
    }

    #[test]
    fn fcq_ref_must_be_valid() {
        let fcq = {
            let string = String::from("Test");
            FQName::new(&string).to_fqname_buf()
        };
        assert_eq!(fcq, "Test");
    }

    #[test]
    fn safe_usage() {
        let string = "java/lang/Object";
        let fcq = FQName::new(string);
        assert_eq!(&*fcq, string);
        assert_eq!(
            (*fcq).as_path(),
            PathBuf::from_iter(["java", "lang", "Object"])
        );

        let mut string = string.to_string();
        let cloned = string.clone();
        let fcq = FQName::new(&cloned);
        assert_eq!(&*fcq, &string);
        assert_eq!(
            (*fcq).as_path(),
            PathBuf::from_iter(["java", "lang", "Object"])
        );

        string.push('j');
        assert_ne!(&*fcq, &string); // should no longer be equal
    }
}
