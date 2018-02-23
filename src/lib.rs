#[macro_use]
extern crate failure;
#[macro_use]
extern crate failure_derive;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_yaml;

pub mod service;

use std::fs::File;
use std::path::PathBuf;

/// The main entry point to the straw_boss library and executable.
///
/// This parses a `Procfile` and prints out the information in it in a more explicit, YAML format.
///
/// # Arguments
///
/// * `procfile`: The path to a [`Procfile`](https://devcenter.heroku.com/articles/procfile)
///   containing information about the services this straw boss should oversee.
pub fn run(procfile: PathBuf) {
    let f = File::open(&procfile).expect(&format!("Unable to open Procfile: {:?}", &procfile));
    let services: Vec<service::Service> = service::read_procfile(f).expect(&format!(
        "Unable to read data from Procfile: {:?}",
        &procfile
    ));
    let index = service::index_services(&services);
    let yaml = serde_yaml::to_string(&index).expect("Cannot convert index to YAML.");

    println!("{}", yaml);
}
