//! Handles loading and storing of the metadata library as well as queries.

use model::LibraryEntry;
use std::str::FromStr;
use std::fmt;

/// An abstraction of a cargo crate version given as `major.minor.patch`.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
struct VersionSpec {
    major: u32,
    minor: u32,
    patch: u32
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
    changed: bool
}

impl FromStr for VersionSpec {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // Determine and validate the single version "digits" seperated by dots
        let digits: Vec<Option<u32>> = s.split('.')
            .map(| v | u32::from_str(v).ok())
            .collect();
        if (&digits).into_iter().any(| v | v.is_none()) {
            return Err("Version digit ill-formatted.");
        }
        if digits.len() != 3 {
            return Err("Version has wrong amount of digits.");
        }

        // At this point we already know, that digits contains 3 entries which are Ok
        Ok(VersionSpec {
            major: digits[0].unwrap(),
            minor: digits[1].unwrap(),
            patch: digits[2].unwrap()
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
            entries: Vec::new()
        }
    }
}

impl Library {
    fn new(path: &str) -> Library {
        Library {
            content: LibraryFile::default(),
            path: String::from(path),
            changed: true
        }
    }
}
