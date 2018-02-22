#[macro_use]
extern crate clap;

use std::env;
use std::path::PathBuf;
use clap::Arg;

fn main() {
    let procfile = parse_args();
    println!("PROCFILE = {:?}", &procfile);
}

fn parse_args() -> PathBuf {
    let matches = app_from_crate!()
        .arg(
            Arg::with_name("procfile")
                .short("p")
                .long("procfile")
                .value_name("FILENAME")
                .default_value("Procfile")
                .help("The Procfile defining the services to run locally."),
        )
        .get_matches();

    let pwd = env::current_dir().expect("Cannot get current directory.");
    // This should be safe to unwrap.
    let procfile = matches.value_of("procfile").unwrap_or("Procfile");
    pwd.join(&procfile)
}
