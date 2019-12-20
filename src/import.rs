//! This module provides functions to import library entries from various file types.

use configuration::util::assemble_name;
use configuration::Configuration;
use model::{FileDigest, LibraryEntry, LibraryEntryMeta, LibraryEntryType, Month, ParseMonthError, TagMap};
use sha2::{Digest, Sha256};
use std::convert::From;
use std::error::Error;
use std::fs;
use std::fs::File;
use std::io;
use std::io::copy;
use std::io::BufReader;
use std::path::Path;
use std::string;
use std::vec::Vec;

quick_error! {
    #[derive(Debug)]
    pub enum ImportError {
        /// Returned when an I/O error occurs while importing a file
        Io(err: io::Error) {
            description(err.description())
            display(self_) -> ("I/O error: {}", self_.description())
            from()
        }
        /// Returned when the imported file could not be parsed correctly
        Parse(descr: String) {
            description(descr)
            display(self_) -> ("Parsing failed: {}", self_.description())
            from(e: ParseMonthError) -> (format!("{}", e))
        }
        /// Returned when an entry key not present in the bibliography was specified
        NoBibliographyFound(descr: String) {
            description(descr)
            display(self_) -> ("No fitting bibliography found: {}",
                               self_.description())
        }
        /// Returned when the imported file is not valid UTF-8
        Utf8(err: string::FromUtf8Error) {
            description(err.description())
            display(self_) -> ("File not valid UTF-8: {}",
                               self_.description())
            from()
        }
        /// Returned when the file type could not be identified
        UnknownFile(descr: String) {
            description(descr)
            display(self_) -> ("File type unkown: {}", self_.description())
        }
        /// Returned when a file's path is corrupt
        CorruptFilePath(descr: String) {
            description(descr)
            display(self_) -> ("File path corrupt: {}", self_.description())
        }
    }
}

type ImportResultSet = Vec<LibraryEntryMeta>;
type ImportResult = Result<ImportResultSet, ImportError>;

pub fn import<P: AsRef<Path>>(
    file_path: P,
    resource_path: P,
    key: Option<&str>,
    force_move: bool,
    force_copy: bool,
    tags: Vec<String>,
    conf: &Configuration,
) -> Result<LibraryEntry, ImportError> {
    // Read file data as UTF-8 String
    let mut resource_reader = BufReader::new(File::open(&resource_path)?);
    let mut resource_bytes: Vec<u8> = Vec::new();
    copy(&mut resource_reader, &mut resource_bytes)?;
    let file_content = String::from_utf8(resource_bytes)?;

    // Use fitting import function to import the file
    let results = match resource_path.as_ref().extension() {
        Some(ext) => {
            if ext == "bib" {
                bib::import(file_content)
            } else {
                Err(ImportError::UnknownFile(format!(
                    "File extension {} not known.",
                    ext.to_string_lossy()
                )))
            }
        }
        None => Err(ImportError::UnknownFile(String::from(
            "File has no extension.",
        ))),
    }?;

    let known_keys = || results.iter().map(|bib| bib.key()).collect::<Vec<&str>>();

    let meta = match key {
        Some(k) => results
            .iter()
            .find(|bib| k == bib.key())
            .cloned()
            .ok_or_else(|| {
                ImportError::NoBibliographyFound(format!(
                    "Key {} unkown; known keys are: {:?}",
                    String::from(k),
                    known_keys()
                ))
            }),
        None => {
            if results.len() == 1 {
                Ok((&results[0]).clone())
            } else {
                Err(ImportError::NoBibliographyFound(format!(
                    "Multiple bibliographies in file. Please specify a key. \
                     known keys are: {:?}",
                    known_keys()
                )))
            }
        }
    }?;

    // Decompose the file name
    let file_stem = file_path
        .as_ref()
        .file_stem()
        .ok_or_else(|| ImportError::CorruptFilePath(String::from("No file name specified.")))?
        .to_str()
        .ok_or_else(|| {
            ImportError::CorruptFilePath(format!(
                "File name {} not valid UTF-8",
                file_path.as_ref().to_string_lossy()
            ))
        })?;
    let file_ext = file_path
        .as_ref()
        .extension()
        .ok_or_else(|| ImportError::CorruptFilePath(String::from("No file name specified.")))?
        .to_str()
        .ok_or_else(|| {
            ImportError::CorruptFilePath(format!(
                "File extension of {} not valid UTF-8",
                file_path.as_ref().to_string_lossy()
            ))
        })?;

    // New lifetime to make sure the reader is closed before moving any file
    let digest = calculate_digest(&file_path)?;

    let name = format!("{}.{}", assemble_name(file_stem, &meta, conf), file_ext);
    let paths = if tags.is_empty() {
        vec![conf
            .variables()
            .document_location()
            .join(&name)
            .to_str()
            .map(String::from)
            .ok_or_else(|| ImportError::CorruptFilePath(String::from("Path is not valid UTF-8")))?]
    } else {
        (&tags)
            .iter()
            .map(|t| conf.variables().document_location().join(t).join(&name))
            .map(|p| p.to_str().map(String::from))
            .collect::<Option<Vec<String>>>()
            .ok_or_else(|| ImportError::CorruptFilePath(String::from("Path is not valid UTF-8")))?
    };

    for (i, p) in (&paths).iter().enumerate() {
        let dir = (p as &AsRef<Path>).as_ref().parent().unwrap();
        if !dir.exists() {
            fs::create_dir_all(dir)?;
        }
        if i == 0 {
            if force_move || (!force_copy && conf.variables().move_files()) {
                fs::rename(&file_path, p)?;
            } else {
                fs::copy(&file_path, p)?;
            }
        } else {
            fs::hard_link(&paths[0], p)?;
        }
    }

    Ok(LibraryEntry::new(meta, tags, paths, digest))
}

fn calculate_digest<P: AsRef<Path>>(path: P) -> Result<FileDigest, ImportError> {
    // This can be done more elegantly (by not loading the entire file) but should suffice
    // for now
    let mut file_reader = BufReader::new(File::open(path.as_ref())?);
    let mut file_bytes: Vec<u8> = Vec::new();
    copy(&mut file_reader, &mut file_bytes)?;
    let mut hasher = Sha256::default();
    hasher.input(file_bytes.as_slice());

    Ok(hasher.result())
}

mod bib {
    use super::*;
    use nom::IError;
    use nom_bibtex::error::BibtexError;
    use nom_bibtex::{Bibliography, Bibtex};

    impl From<BibtexError> for ImportError {
        fn from(err: BibtexError) -> ImportError {
            ImportError::Parse(String::from(err.description()))
        }
    }

    impl From<IError> for ImportError {
        fn from(err: IError) -> ImportError {
            match err {
                IError::Incomplete(_) => ImportError::Parse(String::from("Incomplete input")),
                IError::Error(e) => ImportError::Parse(String::from(e.description())),
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
            _ => Err(ImportError::Parse(format!("Entry type {} not known", name))),
        }
    }

    fn import_bib(b: &Bibliography) -> Result<LibraryEntryMeta, ImportError> {
        let tags: TagMap = b.tags().iter().cloned().collect();

        let find_tag = |tag: &str| {
            b.tags()
                .iter()
                .find(|&(ref name, _)| name.to_lowercase() == tag)
        };
        let find_tag_required = |tag: &str| {
            find_tag(tag).ok_or_else(|| ImportError::Parse(format!("Missing tag \"{}\"", tag)))
        };

        let entry_type = parse_entry_type(b.entry_type())?;
        let (_, title) = find_tag_required("title")?;
        let authors = parse_author_list(&find_tag_required("author")?.1);
        let year = match find_tag_required("year")?.1.parse::<u32>() {
            Ok(y) => y,
            Err(e) => return Err(ImportError::Parse(format!("Failed to parse year: {}", e))),
        };
        let month = match find_tag("month") {
            Some(&(_, ref s)) => Some(s.parse::<Month>()?),
            None => None,
        };

        Ok(LibraryEntryMeta::new(
            String::from(b.citation_key()),
            entry_type,
            title.clone(),
            authors,
            year,
            month,
            Some(tags),
        ))
    }

    pub fn import(file: String) -> ImportResult {
        let bibs = Bibtex::parse(&file)?;

        Ok(bibs
            .bibliographies()
            .iter()
            // map_or_else would make this more elegant but is not yet in stable
            // see #53268
            .filter_map(|bib| match import_bib(bib) {
                Ok(b) => Some(b),
                Err(e) => {
                    eprintln!(
                        "Warning: Failed to load entry {}: {}",
                        bib.citation_key(),
                        e
                    );
                    None
                }
            })
            .collect())
    }
}
