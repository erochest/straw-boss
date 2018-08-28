#[macro_use]
extern crate clap;
extern crate straw_boss;
#[macro_use]
extern crate failure;

use clap::{Arg, ArgMatches, SubCommand};
use std::env;

use straw_boss::actions::Action;
use straw_boss::procfile::Procfile;
use straw_boss::server::rest::DOMAIN_SOCKET;
use straw_boss::Result;

fn main() -> Result<()> {
    let action = parse_args()?;
    straw_boss::run(action)
}

// TODO: Take an option/envvar for the domain socket path.
fn parse_args() -> Result<Action> {
    let procfile = Arg::with_name("procfile")
        .short("p")
        .long("procfile")
        .value_name("FILENAME")
        .default_value("Procfile")
        .help("The Procfile defining the services to run locally.");
    let matches =
        app_from_crate!()
            .subcommand(
                SubCommand::with_name("start")
                    .about("This starts all of the processes listed in the Procfile.")
                    .arg(procfile.clone())
                    .arg(Arg::with_name("daemon").short("d").long("daemon").help(
                        "Run the straw boss task manager in the background as a server/daemon.",
                    )),
            ).subcommand(SubCommand::with_name("status").about("This queries daemonized tasks."))
            .subcommand(
                SubCommand::with_name("yamlize")
                    .about(
                        "This reads the process information from the Procfile and prints it as \
                         YAML.",
                    ).arg(procfile.clone()),
            ).get_matches();

    if let Some(sub_matches) = matches.subcommand_matches("start") {
        let procfile = get_procfile(&sub_matches)?;
        let socket_path =
            env::var("STRAWBOSS_SOCKET_PATH").unwrap_or_else(|_| String::from(DOMAIN_SOCKET));
        Ok(Action::Start(
            procfile,
            sub_matches.is_present("daemon"),
            socket_path,
        ))
    } else if let Some(_sub_matches) = matches.subcommand_matches("status") {
        let socket_path =
            env::var("STRAWBOSS_SOCKET_PATH").unwrap_or_else(|_| String::from(DOMAIN_SOCKET));
        Ok(Action::Status(socket_path))
    } else if let Some(sub_matches) = matches.subcommand_matches("yamlize") {
        let procfile = get_procfile(&sub_matches)?;
        Ok(Action::Yamlize(procfile))
    } else {
        Err(format_err!(
            "Unknown subcommand: {:?}",
            &matches.subcommand()
        ))
    }
}

fn get_procfile(matches: &ArgMatches) -> Result<Procfile> {
    let pwd = env::current_dir()
        .map_err(|err| format_err!("Cannot get current directory: {:?}", &err))?;
    let procfile = matches.value_of("procfile").unwrap_or("Procfile");
    let procfile = Procfile::new(pwd.join(&procfile));
    Ok(procfile)
}
