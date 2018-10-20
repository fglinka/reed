//! Defines structures used to handle and store library entries.

use std::vec::Vec;
use sha2::digest::{generic_array::GenericArray, FixedOutput};
use sha2::Sha256;
use chrono::Date;
use chrono::offset::Utc;

/// An enum specifying the type of a document. Currently containing all default BibTeX types.
#[derive(Debug, Clone)]
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
    Unpublished
}

/// The type used for representing file digests, though not the actual type stored in the
/// database.
pub type FileDigest = GenericArray<u8, <Sha256 as FixedOutput>::OutputSize>;

/// A structure containing all information of an entry stored in the document database.
#[derive(Debug, Clone)]
pub struct LibraryEntry {
    entry_type: LibraryEntryType,
    title: String,
    authors: Vec<String>,
    date: Date<Utc>,
    filename: String,
    file_digest: FileDigest
}
