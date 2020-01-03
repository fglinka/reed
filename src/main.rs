#![warn(clippy::all)]

extern crate sha2;
#[macro_use]
extern crate quick_error;
#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate serde_json;
extern crate serde_yaml;
#[macro_use]
extern crate nom;
extern crate nom_bibtex;
#[macro_use]
extern crate lazy_static;
extern crate directories;
#[macro_use]
extern crate clap;
extern crate hex;
extern crate regex;

mod cli;
mod configuration;
mod import;
mod library;
mod model;

use cli::process_args;
use configuration::Configuration;

fn main() {
    // Load configuration
    let conf = Configuration::load().unwrap();
    let mut lib = match library::load_from_cfg(&conf) {
        Ok(l) => l,
        Err(e) => {
            eprintln!("Failed to load library: {}", e);
            return;
        }
    };
    process_args(&conf, &mut lib);
}
