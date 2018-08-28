#[macro_use]
extern crate clap;
extern crate straw_boss;
#[macro_use]
extern crate failure;

use clap::{Arg, ArgMatches, SubCommand};
use std::env;
use std::path::PathBuf;

use straw_boss::actions::Action;
use straw_boss::procfile::Procfile;
use straw_boss::server::rest::DOMAIN_SOCKET;
use straw_boss::server::ServerRunMode;
use straw_boss::Result;

const SOCKET_PATH_VAR: &'static str = "STRAWBOSS_SOCKET_PATH";
const PID_FILE_VAR: &'static str = "STRAWBOSS_PID_FILE";

fn main() -> Result<()> {
    let action = parse_args()?;
    straw_boss::run(action)
}

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
        let socket_path = env::var(SOCKET_PATH_VAR).unwrap_or_else(|_| String::from(DOMAIN_SOCKET));
        let run_mode = if sub_matches.is_present("daemon") {
            let pid_file = env::var(PID_FILE_VAR).unwrap_or_else(|_| {
                String::from(env::temp_dir().join("straw-boss.pid").to_string_lossy())
            });
            ServerRunMode::Daemon(PathBuf::from(pid_file))
        } else {
            ServerRunMode::Foreground
        };
        Ok(Action::Start(procfile, run_mode, socket_path))
    } else if let Some(_sub_matches) = matches.subcommand_matches("status") {
        let socket_path = env::var(SOCKET_PATH_VAR).unwrap_or_else(|_| String::from(DOMAIN_SOCKET));
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
