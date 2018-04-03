#![feature(try_from)]
#![feature(plugin)]

extern crate clap;
#[macro_use]
extern crate failure;
#[macro_use]
extern crate failure_derive;
#[cfg(test)]
extern crate reqwest;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_yaml;
extern crate shellwords;
extern crate spectral;

use std::io;

pub mod actions;
pub mod service;

/// The main entry point to the straw_boss library and executable.
///
/// This parses a `Procfile` and prints out the information in it in a more explicit, YAML format.
///
/// # Arguments
///
/// * `action`: The `Action` object to run.
pub fn run(action: actions::Action) {
    let stdout = io::stdout();
    let mut writer = io::BufWriter::new(stdout);
    action
        .execute(&mut writer)
        .map_err(|err| format_err!("ERROR: {}", &err))
        .unwrap();
}
