#![feature(try_from)]
#![feature(plugin)]
#![feature(proc_macro)]

extern crate clap;
#[macro_use]
extern crate failure;
//#[macro_use]
//extern crate failure_derive;
#[cfg(test)]
extern crate reqwest;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_yaml;
extern crate shellwords;
#[cfg(test)]
extern crate spectral;
extern crate thunder;

use std::env;
use std::io;
use std::io::{BufWriter, Write};
use std::path::PathBuf;
use thunder::thunderclap;

pub mod actions;
pub mod service;

use actions::Procfile;

/// A convenience type alias for a specialization of `Result` that uses `failure::Error` for
/// exceptions.
pub type Result<A> = std::result::Result<A, failure::Error>;

pub fn run() {
    StrawBossApp::start();
}

pub struct StrawBossApp;

/// Run commands.
#[thunderclap]
impl StrawBossApp {
    /// This starts all of the processes listed in a Procfile.
    fn launch(procfile: Option<&str>) {
        let procfile = build_procfile(procfile);
        let mut writer = default_writer();
        actions::start(&procfile, &mut writer).expect("Error running start.");
    }

    /// This reads the process information from a Procfile and prints it as YAML.
    fn yamlize(procfile: Option<&str>) {
        let procfile = build_procfile(procfile);
        let mut writer = default_writer();
        actions::yamlize(&procfile, &mut writer).expect("Error yamlizing Procfile.");
    }
}

fn build_procfile(procfile: Option<&str>) -> Procfile {
    let pwd = env::current_dir().expect("Cannot get current directory.");
    let procfile = PathBuf::from(procfile.unwrap_or("Procfile"));
    let procfile = pwd.join(&procfile);
    Procfile::new(procfile)
}

fn default_writer() -> BufWriter<impl Write>
{
    let stdout = io::stdout();
    io::BufWriter::new(stdout)
}

