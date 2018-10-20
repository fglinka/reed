use std::vec::Vec;
use generic_array::GenericArray;
use sha2::Sha256;
use chrono::Date;

/// An enum specifying the type of a document. Currently containing all default BibTeX types.
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
    Thesis
    Misc,
    PHDThesis,
    Proceedings,
    Techreport,
    Unpublished
}

/// The type used for representing file digests, though not the actual type stored in the
/// database.
pub type FileDigest = GenericArray<u8, Sha256::OutputSize>;

/// A structure containing all information of an entry stored in the document database.
pub struct LibraryEntry {
    entry_type: LibraryEntryType,
    title: String,
    authors: Vec<String>,
    date: Date,
    filename: String,
    file_digest: FileDigest
}
