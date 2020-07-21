//! Defines structures used to handle and store library entries.

use serde::{de, Deserialize, Deserializer, Serializer};
use sha2::digest::{generic_array::GenericArray, FixedOutput};
use sha2::Sha256;
use std::collections::HashMap;
use std::fmt;
use std::str::FromStr;
use std::vec::Vec;

quick_error! {
    #[derive(Debug)]
    pub enum ParseMonthError {
        /// Returned when the month expressed as a string is not known
        Unkown(descr: String) {
            display(self_) -> ("Failed to parse month: {}", descr)
        }
        /// Returned when the month was specified numerically but is out of bounds
        OutOfBounds(descr: String) {
            display(self_) -> ("Failed to parse month: {}", descr)
        }
    }
}

/// An enum specifying the type of a document. Currently containing all default BibTeX types.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum LibraryEntryType {
    Article,
    Book,
    Booklet,
    Conference,
    InBook,
    InCollection,
    InProceedings,
    Manual,
    MasterThesis,
    Thesis,
    Misc,
    PHDThesis,
    Proceedings,
    Techreport,
    Unpublished,
}

/// An enum expressing a month and providing various conversion functions
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum Month {
    Jan,
    Feb,
    Mar,
    Apr,
    May,
    Jun,
    Jul,
    Aug,
    Sep,
    Oct,
    Nov,
    Dec,
}

/// The type used for representing file digests, though not the actual type stored in the
/// database.
pub type FileDigest = GenericArray<u8, <Sha256 as FixedOutput>::OutputSize>;

/// The structure used to store tags contained in the original metadata file used during the import.
pub type TagMap = HashMap<String, String>;

/// A structure containing all metadata information of an entry stored in the document
/// database
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LibraryEntryMeta {
    key: String,
    entry_type: LibraryEntryType,
    title: String,
    authors: Vec<String>,
    year: u32,
    month: Option<Month>,
    original_tags: Option<TagMap>,
}

/// A structure containing the metadata and file information of an entry stored in the
/// document database.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LibraryEntry {
    meta: LibraryEntryMeta,
    tags: Vec<String>,
    file_paths: Vec<String>,
    #[serde(serialize_with = "as_hex", deserialize_with = "from_hex")]
    digest: FileDigest,
}

impl fmt::Display for Month {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(match self {
            Month::Jan => "January",
            Month::Feb => "February",
            Month::Mar => "March",
            Month::Apr => "April",
            Month::May => "May",
            Month::Jun => "June",
            Month::Jul => "July",
            Month::Aug => "August",
            Month::Sep => "September",
            Month::Oct => "October",
            Month::Nov => "November",
            Month::Dec => "December",
        })
    }
}

impl FromStr for Month {
    type Err = ParseMonthError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Ok(num) = s.parse::<u32>() {
            return Month::from_number(num);
        }
        let init = s.chars().take(3).collect::<String>().to_lowercase();
        match init.as_str() {
            "jan" => Ok(Month::Jan),
            "feb" => Ok(Month::Feb),
            "mar" => Ok(Month::Mar),
            "apr" => Ok(Month::Apr),
            "may" => Ok(Month::May),
            "jun" => Ok(Month::Jun),
            "jul" => Ok(Month::Jul),
            "aug" => Ok(Month::Aug),
            "sep" => Ok(Month::Sep),
            "oct" => Ok(Month::Oct),
            "nov" => Ok(Month::Nov),
            "dec" => Ok(Month::Dec),
            _ => Err(ParseMonthError::Unkown(format!("month {} unkown", init))),
        }
    }
}

/// Displaying a LibraryEntryType will yield the same result as the derived Debug trait.
/// This function is based on https://stackoverflow.com/a/32712140
impl fmt::Display for LibraryEntryType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl LibraryEntryMeta {
    pub fn new(
        key: String,
        entry_type: LibraryEntryType,
        title: String,
        authors: Vec<String>,
        year: u32,
        month: Option<Month>,
        original_tags: Option<TagMap>,
    ) -> LibraryEntryMeta {
        LibraryEntryMeta {
            key,
            entry_type,
            title,
            authors,
            year,
            month,
            original_tags,
        }
    }
    pub fn key(&self) -> &str {
        &self.key
    }

    pub fn entry_type(&self) -> LibraryEntryType {
        self.entry_type
    }

    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn authors(&self) -> &Vec<String> {
        &self.authors
    }

    pub fn year(&self) -> u32 {
        self.year
    }

    pub fn month(&self) -> Option<Month> {
        self.month
    }

    pub fn original_tags(&self) -> Option<&TagMap> {
        self.original_tags.as_ref()
    }
}

impl LibraryEntry {
    pub fn new(
        meta: LibraryEntryMeta,
        tags: Vec<String>,
        file_paths: Vec<String>,
        digest: FileDigest,
    ) -> LibraryEntry {
        LibraryEntry {
            meta,
            tags,
            file_paths,
            digest,
        }
    }

    pub fn meta(&self) -> &LibraryEntryMeta {
        &self.meta
    }

    pub fn tags(&self) -> &[String] {
        self.tags.as_slice()
    }

    pub fn file_paths(&self) -> &[String] {
        self.file_paths.as_slice()
    }

    pub fn digest(&self) -> &FileDigest {
        &self.digest
    }
}

impl Month {
    pub fn from_number(num: u32) -> Result<Month, ParseMonthError> {
        match num {
            1 => Ok(Month::Jan),
            2 => Ok(Month::Feb),
            3 => Ok(Month::Mar),
            4 => Ok(Month::Apr),
            5 => Ok(Month::May),
            6 => Ok(Month::Jun),
            7 => Ok(Month::Jul),
            8 => Ok(Month::Aug),
            9 => Ok(Month::Sep),
            10 => Ok(Month::Oct),
            11 => Ok(Month::Nov),
            12 => Ok(Month::Dec),
            _ => Err(ParseMonthError::OutOfBounds(format!(
                "month {} out of bounds",
                num
            ))),
        }
    }
}

fn as_hex<S: Serializer, T: AsRef<[u8]>>(arr: T, serializer: S) -> Result<S::Ok, S::Error> {
    serializer.serialize_str(&hex::encode(arr))
}

fn from_hex<'a, D: Deserializer<'a>>(deserializer: D) -> Result<FileDigest, D::Error> {
    match hex::decode(String::deserialize(deserializer)?) {
        Ok(v) => Ok(FileDigest::clone_from_slice(&v)),
        Err(e) => Err(de::Error::custom(e)),
    }
}
