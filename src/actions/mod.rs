use client::local::RestManagerClient;
use client::status::status;
use client::ManagerClient;
use procfile::Procfile;
use server::local::RestManagerServer;
use server::start::start;
use server::ServerRunMode;
use std::io::Write;
use std::path::PathBuf;
use tasks::TaskSpec;
use yamlize::yamlize;
use Result;

/// An action that the straw boss can do.
#[derive(Debug)]
pub enum Action {
    Start(Procfile, ServerRunMode, PathBuf),
    Status(PathBuf),
    Stop(PathBuf, TaskSpec),
    Yamlize(Procfile),
}

impl Action {
    /// Execute an action. This dispatches to the appropriate function to take the action
    /// described. It writes its output to the `Write` implementor passed in.
    pub fn execute<W: Write>(self, writer: &mut W) -> Result<()> {
        match self {
            Action::Start(procfile, run_mode, socket_domain) => {
                let mut server = RestManagerServer::at_path(socket_domain);
                let services = procfile.read_services()?;
                start(&mut server, run_mode, services)
            }
            Action::Status(socket_domain) => {
                let client = RestManagerClient::at_path(socket_domain);
                status(&client).and_then(|ms| {
                    writer
                        .write_all(ms.get_message().as_bytes())
                        .map_err(|err| format_err!("Unable to write output: {:?}", &err))
                })
            }
            Action::Stop(socket_domain, tasks) => {
                let client = RestManagerClient::at_path(socket_domain);
                client.stop(tasks)
            }
            Action::Yamlize(ref procfile) => yamlize(procfile, writer),
        }
    }
}
