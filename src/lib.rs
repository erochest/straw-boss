#![feature(try_from)]
#![feature(plugin)]
#![feature(option_replace)]

extern crate chrono;
extern crate clap;
extern crate daemonize;
extern crate duct;
#[macro_use]
extern crate failure;
//#[macro_use]
//extern crate failure_derive;
#[cfg(test)]
extern crate reqwest;
extern crate serde;
#[macro_use]
extern crate serde_derive;
#[cfg(test)]
extern crate assert_fs;
#[cfg(test)]
extern crate predicates;
extern crate rmp_serde;
extern crate serde_yaml;
extern crate shellwords;
#[cfg(test)]
extern crate spectral;

use std::io;

pub mod actions;
pub mod client;
pub mod messaging;
pub mod procfile;
pub mod server;
pub mod service;
pub mod yamlize;

/// A convenience type alias for a specialization of `Result` that uses `failure::Error` for
/// exceptions.
pub type Result<A> = std::result::Result<A, failure::Error>;

/// The main entry point to the straw_boss library and executable.
///
/// This parses a `Procfile` and prints out the information in it in a more explicit, YAML format.
///
/// # Arguments
///
/// * `action`: The `Action` object to run.
pub fn run(action: actions::Action) -> Result<()> {
    let stdout = io::stdout();
    let mut writer = io::BufWriter::new(stdout);
    action
        .execute(&mut writer)
        .map_err(|err| format_err!("ERROR: {}", &err))
}
