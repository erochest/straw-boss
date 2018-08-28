use client::rest::RestManagerClient;
use client::{ManagerClient, ManagerStatus};
use daemonize::Daemonize;
use procfile::Procfile;
use server::rest::RestManagerServer;
use server::ManagerServer;
use service::worker::ServiceWorker;
use std::collections::HashMap;
use std::env;
use std::io::Write;
use std::path::PathBuf;
use yamlize::yamlize;
use Result;

/// An action that the straw boss can do.
#[derive(Debug)]
pub enum Action {
    Start(Procfile, bool, String),
    Status(String),
    Yamlize(Procfile),
}

impl Action {
    /// Execute an action. This dispatches to the appropriate function to take the action
    /// described. It writes its output to the `Write` implementor passed in.
    pub fn execute<W: Write>(&self, writer: &mut W) -> Result<()> {
        match *self {
            Action::Start(ref procfile, daemon, ref socket_domain) => {
                start(procfile, writer, daemon, socket_domain)
            }
            Action::Status(ref socket_domain) => {
                let client = RestManagerClient::at_path(PathBuf::from(socket_domain));
                let status = status(&client)?;

                match status {
                    ManagerStatus::NotFound => Err(format_err!(
                        "Straw-boss not running. Why don't you try `straw-boss start --daemon`"
                    )),
                    ManagerStatus::RunningTasks(_) => Ok(()),
                }
            }
            Action::Yamlize(ref procfile) => yamlize(procfile, writer),
        }
    }
}

/// Start all the processes described in the `Procfile`.
///
/// If `is_daemon` is given, the server is started in the background, and this
/// function returns immediately.
pub fn start<W: Write>(
    procfile: &Procfile,
    _writer: &mut W,
    is_daemon: bool,
    socket_domain: &str,
) -> Result<()> {
    if is_daemon {
        let pid_file = env::temp_dir().join("straw-boss.pid");
        let cwd = env::current_dir()
            .map_err(|err| format_err!("Unable to get current working directory: {:?}", &err))?;

        Daemonize::new()
            .pid_file(pid_file)
            .working_directory(cwd)
            .start()
            .map_err(|err| format_err!("Unable to start daemon: {:?}", &err))?;
    }

    let services = procfile.read_services()?;
    let workers: Vec<Result<ServiceWorker>> = services
        .into_iter()
        .map(|s| ServiceWorker::new(s))
        .map(|mut w| w.start().map(|_| w))
        .collect();

    let mut server = RestManagerServer::at_path(PathBuf::from(socket_domain));
    server.initialize()?;
    server.set_workers(workers.into_iter().filter_map(|w| w.ok()).collect());
    server.start()
}

/// Query a daemonized server to get the status of all of the tasks it's running.
pub fn status<C: ManagerClient>(client: &C) -> Result<ManagerStatus> {
    if client.is_running() {
        client
            .get_workers()
            .map(|workers| {
                workers
                    .into_iter()
                    .map(|w| (w.name, w.command))
                    .collect::<HashMap<_, _>>()
            }).map(ManagerStatus::RunningTasks)
    } else {
        Ok(ManagerStatus::NotFound)
    }
}

#[cfg(test)]
mod test;
