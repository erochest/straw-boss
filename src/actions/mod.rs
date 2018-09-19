use client::local::RestManagerClient;
use client::ManagerClient;
use daemonize::Daemonize;
use serde_yaml;
use service::service;
use service::worker::{ServiceWorker, Worker};
use std::env;
use std::fs::File;
use procfile::Procfile;
use server::local::RestManagerServer;
use server::start::start;
use server::ServerRunMode;
use std::io::Write;
use std::path::PathBuf;
use yamlize::yamlize;
use Result;

/// An action that the straw boss can do.
#[derive(Debug)]
pub enum Action {
    Start(Procfile, ServerRunMode, PathBuf),
    Stop(PathBuf),
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
            Action::Stop(socket_domain) => {
                let client = RestManagerClient::at_path(socket_domain);
                client.stop_server()
            }
            Action::Yamlize(ref procfile) => yamlize(procfile, writer),
        }
    }
}
