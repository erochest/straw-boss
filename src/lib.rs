#[macro_use]
extern crate failure;
#[macro_use]
extern crate failure_derive;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_yaml;

mod service;

use std::fs::File;
use std::io;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::str::FromStr;

pub fn run(procfile: PathBuf) {
    let f = File::open(&procfile).expect(&format!("Unable to open Procfile: {:?}", &procfile));
    let reader = BufReader::new(f);
    let services = reader
        .lines()
        .collect::<io::Result<Vec<String>>>()
        .expect(&format!("Unable to read from Procfile: {:?}.", &procfile))
        .into_iter()
        .map(|s| s.parse())
        .collect::<Result<Vec<service::Service>, failure::Error>>()
        .expect("Unable to parse Procfile line.");
    let index = service::index_services(&services);
    let yaml = serde_yaml::to_string(&index).expect("Cannot convert index to YAML.");

    println!("{}", yaml);
}
