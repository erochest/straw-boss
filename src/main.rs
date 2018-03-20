#[macro_use]
extern crate clap;
extern crate straw_boss;

use std::env;
use clap::{Arg, SubCommand};

use straw_boss::actions::{Action, Procfile};

fn main() {
    let action = parse_args();
    straw_boss::run(action);
}

fn parse_args() -> Action {
    let matches = app_from_crate!()
        .arg(
            Arg::with_name("procfile")
                .short("p")
                .long("procfile")
                .value_name("FILENAME")
                .default_value("Procfile")
                .help("The Procfile defining the services to run locally."),
        )
        .subcommand(
            SubCommand::with_name("start")
                .about("starts the processes")
                .help("This starts all of the processes listed in the Procfile."),
        )
        .subcommand(
            SubCommand::with_name("yamlize")
                .about("prints the process information as YAML")
                .help(
                    "This reads the process information from the Procfile and prints it as \
                     YAML.",
                ),
        )
        .get_matches();

    let pwd = env::current_dir().expect("Cannot get current directory.");
    // This should be safe to unwrap.
    let procfile = matches.value_of("procfile").unwrap_or("Procfile");
    let procfile = pwd.join(&procfile);

    if matches.subcommand_matches("yamlize").is_some() {
        Action::Yamlize(Procfile::new(procfile))
    } else {
        Action::Start(Procfile::new(procfile))
    }
}
