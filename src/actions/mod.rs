use client::rest::RestManagerClient;
use client::status::status;
use client::ManagerStatus;
use procfile::Procfile;
use server::start::start;
use server::ServerRunMode;
use std::io::Write;
use std::path::PathBuf;
use yamlize::yamlize;
use Result;

/// An action that the straw boss can do.
#[derive(Debug)]
pub enum Action {
    Start(Procfile, ServerRunMode, String),
    Status(String),
    Yamlize(Procfile),
}

impl Action {
    /// Execute an action. This dispatches to the appropriate function to take the action
    /// described. It writes its output to the `Write` implementor passed in.
    pub fn execute<W: Write>(&self, writer: &mut W) -> Result<()> {
        match *self {
            Action::Start(ref procfile, ref daemon, ref socket_domain) => {
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
