//! Handles loading and storing of the metadata library as well as queries.

use model::LibraryEntry;
use std::error::Error;
use std::fmt;
use std::fs::File;
use std::ops::Drop;
use std::str::FromStr;

quick_error! {
    /// Used to indicate, that the library could not be correctly loaded or stored
    #[derive(Debug)]
    pub enum LibraryPersistenceError {
        /// Returned when an I/O error occurs while loading or storing library
        Io(err: std::io::Error) {
            description(err.description())
            display(self_) -> ("Saving or loading library failed; I/O error: {}",
                               self_.description())
            from()
        }
        /// Returned when Serialization or Deserialization of the library failed
        Serialization(err: serde_json::Error) {
            description(err.description())
            display(self_) -> ("Saving or loading library failed; (De)serialization error: {}",
                               self_.description())
            from()
        }
    }
}

/// An abstraction of a cargo crate version given as `major.minor.patch`.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
struct VersionSpec {
    major: u32,
    minor: u32,
    patch: u32,
}

#[derive(Debug, Serialize, Deserialize)]
struct LibraryFile {
    creation_version: VersionSpec,
    entries: Vec<LibraryEntry>,
}

#[derive(Debug)]
pub struct Library {
    content: LibraryFile,
    path: String,
    changed: bool,
}

impl FromStr for VersionSpec {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // Determine and validate the single version "digits" seperated by dots
        let digits: Vec<Option<u32>> = s.split('.').map(|v| u32::from_str(v).ok()).collect();
        if digits.iter().any(|v| v.is_none()) {
            return Err("Version digit ill-formatted.");
        }
        if digits.len() != 3 {
            return Err("Version has wrong amount of digits.");
        }

        // At this point we already know, that digits contains 3 entries which are Ok
        Ok(VersionSpec {
            major: digits[0].unwrap(),
            minor: digits[1].unwrap(),
            patch: digits[2].unwrap(),
        })
    }
}

impl fmt::Display for VersionSpec {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

impl Default for LibraryFile {
    fn default() -> LibraryFile {
        LibraryFile {
            // The crate version should be formatted correctly
            creation_version: VersionSpec::from_str(crate_version!()).unwrap(),
            entries: Vec::new(),
        }
    }
}

impl Drop for Library {
    fn drop(&mut self) {
        // store the new state of the library if it was changed
        if self.changed {
            self.store().unwrap()
        }
    }
}

impl Library {
    pub fn new(path: &str) -> Library {
        Library {
            content: LibraryFile::default(),
            path: String::from(path),
            changed: true,
        }
    }

    pub fn load(path: &str) -> Result<Library, LibraryPersistenceError> {
        // Open the library file and parse it
        let content = serde_json::from_reader(File::open(path)?)?;

        Ok(Library {
            content,
            path: String::from(path),
            changed: false,
        })
    }

    pub fn store(&self) -> Result<(), LibraryPersistenceError> {
        serde_json::to_writer(File::create(&self.path)?, &self.content)?;

        Ok(())
    }
}
