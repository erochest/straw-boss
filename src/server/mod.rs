use daemonize::Daemonize;
use service::service::Service;
use std::env;
use std::path::{Path, PathBuf};
use Result;

pub mod rest;
pub mod start;

#[derive(Debug)]
pub enum ServerRunMode {
    Foreground,
    Daemon(PathBuf),
}

// TODO: Compose in a manager to run the workers. Don't have the server do it.
pub trait ManagerServer {
    fn daemonize<P: AsRef<Path>>(&self, pid_file: P) -> Result<()> {
        let cwd = env::current_dir()
            .map_err(|err| format_err!("Unable to get current working directory: {:?}", &err))?;

        Daemonize::new()
            .pid_file(pid_file)
            .working_directory(cwd)
            .start()
            .map_err(|err| format_err!("Unable to start daemon: {:?}", &err))
    }

    fn start_workers(&mut self, workers: Vec<Service>) -> Result<()>;
    fn start_server(&mut self) -> Result<()>;
}

#[derive(Debug, Serialize, Deserialize, PartialEq, PartialOrd)]
pub enum RequestMessage {
    GetWorkers,
    Quit,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, PartialOrd)]
pub enum ResponseMessage {
    Workers(Vec<Service>),
}