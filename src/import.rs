//! This module provides functions to import library entries from various file types.

use std::io;
use std::string;
use std::io::BufReader;
use std::io::copy;
use std::fs::File;
use std::fs;
use std::path::Path;
use std::error::Error;
use std::convert::From;
use std::vec::Vec;
use sha2::{Digest, Sha256};
use model::{LibraryEntryType, LibraryEntryMeta, LibraryEntry, Month, ParseMonthError};
use configuration::Configuration;
use configuration::util::assemble_name;

quick_error! {
    #[derive(Debug)]
    pub enum ImportError {
        /// Returned when an I/O error occurs while importing a file
        Io(err: io::Error) {
            description(err.description())
            display(self_) -> ("Import failed; I/O error: {}", self_.description())
            from()
        }
        /// Returned when the imported file could not be parsed correctly
        Parse(descr: String) {
            description(descr)
            display(self_) -> ("Import failed; Parsing failed: {}", self_.description())
            from(e: ParseMonthError) -> (format!("{}", e))
        }
        /// Returned when an entry key not present in the bibliography was specified
        NoBibliographyFound(descr: String) {
            description(descr)
            display(self_) -> ("Import failed; No fitting bibliography found: {}",
                               self_.description())
        }
        /// Returned when the imported file is not valid UTF-8
        Utf8(err: string::FromUtf8Error) {
            description(err.description())
            display(self_) -> ("Import failed; File not valid UTF-8: {}",
                               self_.description())
            from()
        }
        /// Returned when the file type could not be identified
        UnknownFile(descr: String) {
            description(descr)
            display(self_) -> ("Import failed; File type unkown: {}", self_.description())
        }
        /// Returned when a file's path is corrupt
        CorruptFilePath(descr: String) {
            description(descr)
            display(self_) -> ("Import failed; File path corrupt: {}", self_.description())
        }
    }
}

type ImportResultSet = Vec<LibraryEntryMeta>;
type ImportResult = Result<ImportResultSet, ImportError>;

pub fn import<P: AsRef<Path>>(file_path: P, resource_path: P, key: Option<&str>,
                              force_move: bool, force_copy: bool,
                              conf: &Configuration) -> Result<LibraryEntry, ImportError> {
    // Read file data as UTF-8 String
    let mut resource_reader = BufReader::new(File::open(&resource_path)?);
    let mut resource_bytes: Vec<u8> = Vec::new();
    copy(&mut resource_reader, &mut resource_bytes)?;
    let file_content = String::from_utf8(resource_bytes)?;

    // Use fitting import function to import the file
    let results = match resource_path.as_ref().extension() {
        Some(ext) => {
            if ext == "bib" { bib::import(file_content) }
            else {
                Err(ImportError::UnknownFile(
                        format!("File extension {} not known.", ext.to_string_lossy())))
            }
        },
        None => Err(ImportError::UnknownFile(String::from("File has no extension."))),
    }?;

    let known_keys = | | {
        (&results).into_iter()
        .map(| bib | bib.key())
        .collect::<Vec<&str>>()
    };

    let meta = match key {
        Some(k) => {
            results.iter()
                .find(| bib | k == bib.key())
                .cloned()
                .ok_or(
                    ImportError::NoBibliographyFound(
                        format!("Key {} unkown; known keys are: {:?}",
                                String::from(k), 
                                known_keys())))
        },
        None => {
            if results.len() == 1 {
                Ok(results.get(0).unwrap().clone())
            }
            else {
                Err(ImportError::NoBibliographyFound(
                        format!("Multiple bibliographies in file. Please specify a key. \
                                known keys are: {:?}", known_keys())))
            }
        }
    }?;

    // Decompose the file name
    let file_stem = file_path.as_ref()
        .file_stem()
        .ok_or(ImportError::CorruptFilePath(String::from("No file name specified.")))?
        .to_str()
        .ok_or(ImportError::CorruptFilePath(format!("File name {} not valid UTF-8"
                                                   , file_path.as_ref()
                                                   .to_string_lossy())))?;
    let file_ext = file_path.as_ref()
        .extension()
        .ok_or(ImportError::CorruptFilePath(String::from("No file name specified.")))?
        .to_str()
        .ok_or(ImportError::CorruptFilePath(format!("File extension of {} not valid UTF-8"
                                                   , file_path.as_ref()
                                                   .to_string_lossy())))?;

    // New lifetime to make sure the reader is closed before moving any file
    let digest = {
        // This can be done more elegantly (by not loading the entire file) but should suffice
        // for now
        let mut file_reader = BufReader::new(File::open(&file_path)?);
        let mut file_bytes: Vec<u8> = Vec::new();
        copy(&mut file_reader, &mut file_bytes)?;
        let mut hasher = Sha256::default();
        hasher.input(file_bytes.as_slice());

        hasher.result()
    };

    let name = format!("{}.{}", assemble_name(file_stem, &meta, conf), file_ext);
    let path = conf.variables().document_location().join(name);
    let path_str = match path.to_str() {
        Some(s) => String::from(s),
        None => return Err(ImportError::CorruptFilePath(
            format!("Path {} contains non UTF-8 characters", path.to_string_lossy())))
    };
    if force_move || (!force_copy && conf.variables().move_files()) {
        fs::rename(&file_path, &path)?;
    } else {
        fs::copy(&file_path, &path)?;
    }

    Ok(LibraryEntry::new(meta, path_str, digest))
}

mod bib {
    use super::*;
    use nom_bibtex::{Bibtex, Bibliography};
    use nom_bibtex::error::BibtexError;
    use nom::{IError};

    impl From<BibtexError> for ImportError {
        fn from(err: BibtexError) -> ImportError {
            ImportError::Parse(String::from(err.description()))
        }
    }

    impl From<IError> for ImportError {
        fn from(err: IError) -> ImportError {
            match err {
                IError::Incomplete(_) =>
                    ImportError::Parse(String::from("Incomplete input")),
                IError::Error(e) => ImportError::Parse(String::from(e.description()))
            }
        }
    }

    //named!(parse_author<&str, &str>, delimited!(space, tag!("and"), space));

    //named!(parse_author_list<&str, Vec<String>>,
           //separated_nonempty_list!(tag!(" and "), map!(non_empty, String::from)));

    fn parse_author_list(authors: &str) -> Vec<String> {
        authors.split(" and ").map(String::from).collect()
    }

    fn parse_entry_type(name: &str) -> Result<LibraryEntryType, ImportError> {
        match name.to_lowercase().as_str() {
            "article" => Ok(LibraryEntryType::Article),
            "book" => Ok(LibraryEntryType::Book),
            "booklet" => Ok(LibraryEntryType::Booklet),
            "conference" => Ok(LibraryEntryType::Conference),
            "inbook" => Ok(LibraryEntryType::InBook),
            "incollection" => Ok(LibraryEntryType::InCollection),
            "inproceedings" => Ok(LibraryEntryType::InProceedings),
            "manual" => Ok(LibraryEntryType::Manual),
            "masterthesis" => Ok(LibraryEntryType::MasterThesis),
            "thesis" => Ok(LibraryEntryType::Thesis),
            "misc" => Ok(LibraryEntryType::Misc),
            "phdthesis" => Ok(LibraryEntryType::PHDThesis),
            "proceedings" => Ok(LibraryEntryType::Proceedings),
            "techreport" => Ok(LibraryEntryType::Techreport),
            "unpublished" => Ok(LibraryEntryType::Unpublished),
            _ => Err(ImportError::Parse(format!("Entry type {} not known", name)))
        }
    }

    fn import_bib(b: &Bibliography, file: &str) -> Result<LibraryEntryMeta, ImportError> {
        let find_tag = | tag: &str | {
            b.tags().into_iter()
                .find(| &(ref name, _) | name.to_lowercase() == tag)
        };
        let find_tag_required = | tag: &str | {
            find_tag(tag).ok_or(ImportError::Parse(format!("Missing tag \"{}\"", tag)))
        };

        let entry_type = parse_entry_type(b.entry_type())?;
        let (_, title) = find_tag_required("title")?;
        let authors = parse_author_list(&find_tag_required("author")?.1);
        let year = match find_tag_required("year")?.1.parse::<u32>() {
            Ok(y) => y,
            Err(e) => return Err(ImportError::Parse(format!("Failed to parse year: {}", e)))
        };
        let month = match find_tag("month") {
            Some(&(_, ref s)) => Some(s.parse::<Month>()?),
            None => None
        };
        
        Ok(LibraryEntryMeta::new(String::from(b.citation_key()),
                                 entry_type,
                                 title.clone(),
                                 authors,
                                 year,
                                 month,
                                 Some(String::from(file.clone()))))
    }

    pub fn import(file: String) -> ImportResult {
        let bibs = Bibtex::parse(&file)?;

        Ok(bibs.bibliographies()
            .into_iter()
            // map_or_else would make this more elegant but is not yet in stable
            // see #53268
            .filter_map(| bib | match import_bib(bib, &file) {
                Ok(b) => Some(b),
                Err(e) => {
                    eprintln!("Warning: Failed to load entry {}: {}", bib.citation_key(), e);
                    None
                }
            })
            .collect())
    }
}
