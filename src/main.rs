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

mod model;
mod import;
mod configuration;
mod cli;

use configuration::Configuration;
use cli::process_args;

fn main() {
    // Load configuration
    let conf = Configuration::load().unwrap();
    process_args(&conf);
}
