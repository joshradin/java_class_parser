//! Allows for file system like access to java like classpaths
//!

use std::collections::{vec_deque, VecDeque};
use std::convert::Infallible;
use std::ffi::{OsStr, OsString};
use std::fmt::{Display, Formatter, Write};
use std::fs::File;
use std::io::{ErrorKind, Read};
use std::ops::{Add, AddAssign};
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::{io, vec};

use cfg_if::cfg_if;
use static_assertions::assert_impl_all;
use url::Url;
use zip::result::ZipError;
use zip::ZipArchive;

cfg_if! {
    if #[cfg(windows)] {
        /// The separator between different entries on the classpath. This is different depending on the os.
        /// In general, the separator on unix is `:`, while on windows it's `;`
        pub const CLASSPATH_SEPARATOR: char = ';';
    } else if #[cfg(unix)] {
        /// The separator between different entries on the classpath. This is different depending on the os.
        /// In general, the separator on unix is `:`, while on windows it's `;`
        pub const CLASSPATH_SEPARATOR: char = ':';
    }
}

/// A classpath in java
#[derive(Debug, PartialEq, Eq, Hash, Clone, Default)]
pub struct Classpath {
    paths: VecDeque<PathBuf>,
}

impl Classpath {
    /// Creates an empty classpath
    pub fn new() -> Self {
        Default::default()
    }

    /// Checks if the given classpath is empty
    pub fn is_empty(&self) -> bool {
        self.paths.is_empty()
    }

    /// Gets the number of entries in the classpath
    pub fn len(&self) -> usize {
        self.paths.len()
    }

    /// Converts this classpath into a usable classpath for java
    ///
    /// # Example
    /// ```
    /// # use std::ffi::OsString;
    /// # use java_classpaths::{Classpath, CLASSPATH_SEPARATOR};
    /// let cp = Classpath::from_iter(["file1", "file2", "file3"]);
    /// assert_eq!(cp.as_os_string(), OsString::from(format!("file1{0}file2{0}file3", CLASSPATH_SEPARATOR)));
    /// ```
    pub fn as_os_string(&self) -> OsString {
        self.paths.iter().fold(OsString::new(), |mut accum, path| {
            if accum.is_empty() {
                path.clone().into_os_string()
            } else {
                accum
                    .write_char(CLASSPATH_SEPARATOR)
                    .expect("couldn't add separator");
                accum.push(path);
                accum
            }
        })
    }

    /// Attempts to get a resource on the classpath.
    ///
    /// Paths will be interpreted with only `/` as a separator. A leading `/` is ignored.
    ///
    /// # Return
    /// Will return `None` is path is not on classpath. Otherwise, `Some(Result)` is returned
    /// where the resource exists. The result is `Ok` if the inner is actually readable.
    ///
    /// # Example
    /// ```no_run
    /// # use std::str::FromStr;
    /// # use java_classpaths::Classpath;
    /// let cp = Classpath::from_str("run.jar");
    /// let resource = cp.get("META-INF/MANIFEST").expect("manifest not found");
    /// ```
    pub fn get<P: AsRef<str>>(&self, path: P) -> Option<io::Result<Resource>> {
        let stripped = path.as_ref().trim_start_matches("/");
        for entry in self {
            if entry.is_dir() {
                if let Some(ret) = Self::get_in_dir(entry, stripped) {
                    return Some(ret);
                }
            } else {
                let ext = entry.extension();
                match ext.and_then(|os| os.to_str()) {
                    Some("jar") | Some("zip") => match Self::get_in_archive(entry, stripped) {
                        Ok(Some(resource)) => return Some(Ok(resource)),
                        Ok(None) => {}
                        Err(e) => return Some(Err(e)),
                    },
                    _ => {}
                }
            }
        }

        None
    }

    fn get_in_archive(archive_path: &Path, entry_path: &str) -> io::Result<Option<Resource>> {
        let archive_file = File::open(archive_path)?;
        let mut archive = ZipArchive::new(archive_file)
            .map_err(|e| io::Error::new(ErrorKind::InvalidData, e.to_string()))?;

        let out = match archive.by_name(entry_path) {
            Ok(mut entry) => {
                let mut buffer = vec![];
                entry.read_to_end(&mut buffer)?;
                Ok(Some(Resource {
                    kind: ResourceKind::ArchiveEntry(VecDeque::from(buffer)),
                    url: Url::parse(&format!(
                        "jar:file:{archive}!{entry_path}",
                        archive = archive_path.to_str().unwrap()
                    ))
                    .unwrap(),
                }))
            }
            Err(err) => match err {
                ZipError::FileNotFound => Ok(None),
                e => Err(io::Error::new(ErrorKind::InvalidData, e)),
            },
        };
        out
    }

    fn get_in_dir(dir: &Path, entry: &str) -> Option<io::Result<Resource>> {
        let full_path = dir.join(entry);
        if full_path.exists() {
            Some(
                File::open(&full_path)
                    .and_then(|file| {
                        Url::from_file_path(&full_path)
                            .map_err(|()| {
                                io::Error::new(
                                    io::ErrorKind::NotFound,
                                    format!("{:?} is not valid as a url", full_path),
                                )
                            })
                            .map(|url| (file, url))
                    })
                    .map(|(file, url)| Resource {
                        kind: ResourceKind::Real(file),
                        url,
                    }),
            )
        } else {
            None
        }
    }
}

/// Classpath manipulation methods
impl Classpath {
    /// Pushes a new entry to this classpath, at the front.
    pub fn push_front<P: AsRef<Path>>(&mut self, path: P) {
        self.paths.push_front(path.as_ref().to_path_buf());
    }

    /// Pushes a new entry to this classpath, at the back.
    pub fn push_back<P: AsRef<Path>>(&mut self, path: P) {
        self.paths.push_back(path.as_ref().to_path_buf());
    }

    /// Joins two classpaths together, with the `self` classpath being at the front and the `other`
    /// classpath at the back.
    pub fn join(self, other: Self) -> Self {
        let mut paths = self.paths;
        paths.extend(other.paths);
        Self { paths }
    }
}

impl Display for Classpath {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.as_os_string())
    }
}

impl<P> FromIterator<P> for Classpath
where
    P: AsRef<Path>,
{
    fn from_iter<T: IntoIterator<Item = P>>(iter: T) -> Self {
        Self {
            paths: iter.into_iter().map(|p| p.as_ref().to_path_buf()).collect(),
        }
    }
}

impl From<Vec<PathBuf>> for Classpath {
    fn from(vec: Vec<PathBuf>) -> Self {
        Self::from_iter(vec)
    }
}

impl From<&Path> for Classpath {
    /// Tries to create a classpath where the given path is a *single* entry.
    fn from(path: &Path) -> Self {
        let mut output = Self::new();
        output.push_front(path);
        output
    }
}

impl From<PathBuf> for Classpath {
    /// Tries to create a classpath where the given path is a *single* entry.
    fn from(path: PathBuf) -> Self {
        let mut output = Self::new();
        output.push_front(path);
        output
    }
}

impl From<&str> for Classpath {
    /// Tries to create a classpath where the given path is a *single* entry.
    fn from(path: &str) -> Self {
        let mut output = Self::new();
        output.push_front(path);
        output
    }
}

impl From<&OsStr> for Classpath {
    /// Tries to create a classpath where the given path is a *single* entry.
    fn from(path: &OsStr) -> Self {
        let mut output = Self::new();
        output.push_front(path);
        output
    }
}

impl FromStr for Classpath {
    type Err = Infallible;

    /// Attempts to parse a classpath, with entries seperated by the Os's classpath separator
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self::from_iter(s.split(CLASSPATH_SEPARATOR)))
    }
}

impl IntoIterator for Classpath {
    type Item = PathBuf;
    type IntoIter = vec_deque::IntoIter<PathBuf>;

    fn into_iter(self) -> Self::IntoIter {
        self.paths.into_iter()
    }
}

impl<'a> IntoIterator for &'a Classpath {
    type Item = &'a Path;
    type IntoIter = vec::IntoIter<&'a Path>;

    fn into_iter(self) -> Self::IntoIter {
        self.paths
            .iter()
            .map(|s| s.as_path())
            .collect::<Vec<_>>()
            .into_iter()
    }
}

impl<P: AsRef<Path>> Extend<P> for Classpath {
    fn extend<T: IntoIterator<Item = P>>(&mut self, iter: T) {
        self.paths
            .extend(iter.into_iter().map(|p| p.as_ref().to_path_buf()))
    }
}

impl Add for Classpath {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        self.join(rhs)
    }
}

impl AddAssign for Classpath {
    fn add_assign(&mut self, rhs: Self) {
        self.extend(rhs)
    }
}

/// A classpath resource. This is some readable entry available on the classpath
#[derive(Debug)]
pub struct Resource {
    kind: ResourceKind,
    url: Url,
}

impl Resource {
    /// Gets the url of the resource as it would appear in java.
    pub fn url(&self) -> &Url {
        &self.url
    }
}

assert_impl_all!(Resource: io::Read);

impl io::Read for Resource {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.kind.read(buf)
    }
}

#[derive(Debug)]
enum ResourceKind {
    Real(File),
    ArchiveEntry(VecDeque<u8>),
}

assert_impl_all!(ResourceKind: io::Read);

impl io::Read for ResourceKind {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        match self {
            ResourceKind::Real(file) => file.read(buf),
            ResourceKind::ArchiveEntry(old_buf) => old_buf.read(buf),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::ffi::OsString;

    use crate::{Classpath, CLASSPATH_SEPARATOR};

    #[test]
    fn as_path() {
        let cp = Classpath::from_iter(["path1", "path2"]);
        let as_path = cp.as_os_string();
        assert_eq!(
            as_path,
            OsString::from(format!("path1{}path2", CLASSPATH_SEPARATOR))
        );
    }

    #[test]
    fn join() {
        let cp1 = Classpath::from("path1");
        let cp2 = Classpath::from("path2");
        assert_eq!(cp1.join(cp2), Classpath::from_iter(["path1", "path2"]));
    }

    #[test]
    fn add_classpaths() {
        let mut cp = Classpath::new();
        cp = cp + Classpath::from("path1");
        cp += Classpath::from_iter(["path2", "path3"]);
        assert_eq!(cp, Classpath::from_iter(["path1", "path2", "path3"]));
    }

    #[test]
    fn from_string() {
        let classpath: Classpath = format!("path1{}path2", CLASSPATH_SEPARATOR)
            .parse()
            .unwrap();
        assert_eq!(classpath, Classpath::from_iter(["path1", "path2"]))
    }
}
