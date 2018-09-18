#[macro_use]
extern crate clap;
extern crate straw_boss;
#[macro_use]
extern crate failure;

use clap::{Arg, SubCommand};
use std::env;

use straw_boss::actions::Action;
use straw_boss::procfile::Procfile;
use straw_boss::Result;

fn main() -> Result<()> {
    let action = parse_args()?;
    straw_boss::run(action)
}

fn parse_args() -> Result<Action> {
    let matches =
        app_from_crate!()
            .arg(
                Arg::with_name("procfile")
                    .short("p")
                    .long("procfile")
                    .value_name("FILENAME")
                    .default_value("Procfile")
                    .help("The Procfile defining the services to run locally."),
            ).subcommand(
                SubCommand::with_name("start")
                    .about("starts the processes")
                    .help("This starts all of the processes listed in the Procfile.")
                    .arg(Arg::with_name("daemon").short("d").long("daemon").help(
                        "Run the straw boss task manager in the background as a server/daemon.",
                    )),
            ).subcommand(
                SubCommand::with_name("yamlize")
                    .about("prints the process information as YAML")
                    .help(
                        "This reads the process information from the Procfile and prints it as \
                         YAML.",
                    ),
            ).get_matches();

    let pwd = env::current_dir().expect("Cannot get current directory.");
    // This should be safe to unwrap.
    let procfile = matches.value_of("procfile").unwrap_or("Procfile");
    let procfile = Procfile::new(pwd.join(&procfile));

    if let Some(sub_matches) = matches.subcommand_matches("start") {
        Ok(Action::Start(procfile, sub_matches.is_present("daemon")))
    } else if matches.subcommand_matches("yamlize").is_some() {
        Ok(Action::Yamlize(procfile))
    } else {
        Err(format_err!(
            "Unknown subcommand: {:?}",
            &matches.subcommand()
        ))
    }
}
