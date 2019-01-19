//! This module defines the content of the global configuration file. Additionally it takes
//! care of loading the static configuration instance

use directories::{ProjectDirs, UserDirs};
use model::LibraryEntryMeta;
use std::default::Default;
use std::error::Error;
use std::fs::{create_dir_all, File};
use std::io::BufReader;
use std::path::{Path, PathBuf};

quick_error! {
    #[derive(Debug)]
    pub enum ConfigurationPersistenceError {
        /// Returned when an IO error occured
        Io(err: std::io::Error) {
            description(err.description())
            display(self_) ->
                ("Saving or loading configuration failed; I/O error: {}",
                    self_.description())
            from()
        }
        /// Returned when Serialization or Deserialization failed
        Serialization(err: serde_yaml::Error) {
            description(err.description())
            display(self_) ->
                ("Saving or loading configuration failed; Serialization error: {}",
                    self_.description())
            from()
        }
    }
}

#[cfg(unix)]
fn get_config_paths() -> Vec<PathBuf> {
    let dirs = ProjectDirs::from("org", "fowlder", "fowlder")
        .expect("Failed to determine config file directories");
    vec![
        dirs.config_dir().join("config.yaml"),
        PathBuf::from("/etc/fowlder.yaml"),
    ]
}

#[cfg(not(unix))]
fn get_config_paths() -> Vec<PathBuf> {
    let dirs = ProjectDirs::from("fowlder", "fowlder", "org")
        .expect("Failed to determine config file directories");
    vec![dirs.config_dir().join("config.yaml")]
}

lazy_static! {
    static ref CONFIG_FILE_PATHS: Vec<PathBuf> = { get_config_paths() };
}

/// Stores the variables of the global configuration.
///
/// This is a seperate struct in order to not save the `modified` variable to disk.
#[derive(Debug, Serialize, Deserialize)]
pub struct ConfigurationVariables {
    document_location: PathBuf,
    library_location: PathBuf,
    // Expandos:
    // %F for original file name including capitalization
    // %f for original file name including capitalization
    // %K for specified citation key including capitalization
    // %k for specified citation key in lower case
    // %A for author name including captialization
    // %a for author name in lower case
    // %T for title including capitalization
    // %t for title in lowercase
    // %Y for the complete year
    // %y for the two digit year
    // %M for month including capitalization if present, else will be deleted
    // %m for month in lower case if present, else will be deleted
    name_pattern: String,
    max_author_names: u32,
    author_separator: String,
    move_files: bool,
}

/// Keeps the global configuration
#[derive(Debug)]
pub struct Configuration {
    variables: ConfigurationVariables,
    modified: bool,
}

impl Default for ConfigurationVariables {
    fn default() -> Self {
        let dirs = UserDirs::new().expect("Failed to determine default user directories");
        let default_doc_dir = dirs
            .document_dir()
            .expect("Failed to determine default document directory")
            .join("Papers");
        let default_library_path = (&default_doc_dir).join("library.json");

        ConfigurationVariables::new(
            default_doc_dir,
            default_library_path,
            String::from("%A-%y-%T"),
            2,
            String::from("_"),
            true,
        )
    }
}

impl ConfigurationVariables {
    pub fn document_location(&self) -> &Path {
        &self.document_location
    }

    pub fn library_location(&self) -> &Path {
        &self.library_location
    }

    pub fn name_pattern(&self) -> &str {
        &self.name_pattern
    }

    pub fn max_author_names(&self) -> u32 {
        self.max_author_names
    }

    pub fn author_separator(&self) -> &str {
        &self.author_separator
    }

    pub fn move_files(&self) -> bool {
        self.move_files
    }

    pub fn new(
        document_location: PathBuf,
        library_location: PathBuf,
        name_pattern: String,
        max_author_names: u32,
        author_separator: String,
        move_files: bool,
    ) -> ConfigurationVariables {
        ConfigurationVariables {
            document_location,
            library_location,
            max_author_names,
            author_separator,
            name_pattern,
            move_files,
        }
    }
}

impl Drop for Configuration {
    fn drop(&mut self) {
        if self.modified {
            if let Err(e) = self.save() {
                eprintln!("Failed to save configuration: {}", e);
            }
        }
    }
}

impl Default for Configuration {
    fn default() -> Self {
        Configuration {
            variables: ConfigurationVariables::default(),
            modified: true,
        }
    }
}

impl Configuration {
    pub fn new(variables: ConfigurationVariables) -> Configuration {
        Configuration {
            variables,
            modified: false,
        }
    }

    pub fn load() -> Result<Configuration, ConfigurationPersistenceError> {
        for path in CONFIG_FILE_PATHS.iter() {
            if let Ok(file) = File::open(path) {
                let mut reader = BufReader::new(file);
                return Ok(Configuration::new(serde_yaml::from_reader(reader)?));
            }
        }

        Ok(Configuration::default())
    }

    pub fn save(&mut self) -> Result<(), ConfigurationPersistenceError> {
        let path = CONFIG_FILE_PATHS
            .iter()
            .find(|&p| p.exists())
            .unwrap_or(&CONFIG_FILE_PATHS[0]);
        create_dir_all((path as &AsRef<Path>).as_ref().parent().unwrap())?;
        serde_yaml::to_writer(File::create(path)?, &self.variables())?;

        Ok(())
    }

    pub fn variables(&self) -> &ConfigurationVariables {
        &self.variables
    }
}

/// A module providing some helper functions for applying the configuration values
pub mod util {
    use super::*;

    fn get_last_name(author: &str) -> Option<&str> {
        if author.contains(',') {
            author.split(',').nth(0)
        } else {
            author.split(' ').last()
        }
    }

    /// Assembles a filename from metadata using the pattern specified in `name_pattern`
    pub fn assemble_name(
        original_name: &str,
        meta: &LibraryEntryMeta,
        conf: &Configuration,
    ) -> String {
        let (authors, authors_last_name) =
            if !meta.authors().is_empty() && conf.variables().max_author_names() != 0 {
                meta.authors()
                    .iter()
                    .take(conf.variables().max_author_names() as usize)
                    .map(|s| (s.clone(), String::from(get_last_name(s).unwrap_or(s))))
                    .enumerate()
                    .map(|(i, (s1, s2))| {
                        if i == 0 {
                            (s1, s2)
                        } else {
                            let sep = conf.variables().author_separator();
                            (format!("{}{}", sep, s1), format!("{}{}", sep, s2))
                        }
                    })
                    .unzip()
            } else {
                (String::from(""), String::from(""))
            };

        let month = match meta.month() {
            Some(m) => m.to_string(),
            None => String::from(""),
        };

        conf.variables()
            .name_pattern()
            .replace("%F", original_name)
            .replace("%f", &original_name.to_lowercase())
            .replace("%K", meta.key())
            .replace("%k", &meta.key().to_lowercase())
            .replace("%A", &authors)
            .replace("%a", &authors.to_lowercase())
            .replace("%L", &authors_last_name)
            .replace("%l", &authors_last_name.to_lowercase())
            .replace("%T", meta.title())
            .replace("%t", &meta.title().to_lowercase())
            .replace("%Y", &meta.year().to_string())
            .replace("%y", &(meta.year() % 100).to_string())
            .replace("%M", &month)
            .replace("%m", &month.to_lowercase())
    }
}
