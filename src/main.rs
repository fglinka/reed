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

mod model;
mod import;
mod configuration;

use configuration::Configuration;

fn main() {
    // Load configuration
    let conf = Configuration::load();
    println!("Hello, world!");
}
