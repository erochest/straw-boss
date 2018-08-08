use daemonize::Daemonize;
use serde_yaml;
use service::service;
use service::service::Service;
use service::worker::ServiceWorker;
use std::env;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use Result;

/// A `Procfile`. This is a newtype for a `PathBuf`.
#[derive(Debug)]
pub struct Procfile(PathBuf);

impl Procfile {
    /// Create a new `Procfile` from a `PathBuf`.
    pub fn new(procfile: PathBuf) -> Procfile {
        Procfile(procfile)
    }

    /// Read a vector of `Service` instances from a `Procfile`.
    pub fn read_services(&self) -> Result<Vec<service::Service>> {
        let &Procfile(ref procfile) = self;
        let f = File::open(&procfile)
            .map_err(|err| format_err!("Unable to open Procfile: {:?}\n{}", &procfile, &err))?;
        Service::read_procfile(f).map_err(|err| {
            format_err!(
                "Unable to read data from Procfile: {:?}\n{}",
                &procfile,
                &err
            )
        })
    }
}

/// An action that the straw boss can do.
#[derive(Debug)]
pub enum Action {
    Server(Procfile),
    Start(Procfile),
    Yamlize(Procfile),
}

impl Action {
    /// Execute an action. This dispatches to the appropriate function to take the action
    /// described. It writes its output to the `Write` implementor passed in.
    pub fn execute<W: Write>(&self, writer: &mut W) -> Result<()> {
        match *self {
            Action::Server(ref procfile) => server(procfile, writer),
            Action::Start(ref procfile) => start(procfile, writer),
            Action::Yamlize(ref procfile) => yamlize(procfile, writer),
        }
    }
}

/// Start the processes with the straw-boss daemon.
pub fn server<W: Write>(procfile: &Procfile, writer: &mut W) -> Result<()> {
    let pid_file = env::temp_dir().join("straw-boss.pid");
    let cwd = env::current_dir()
        .map_err(|err| format_err!("Unable to current working directory: {:?}", &err))?;

    Daemonize::new()
        .pid_file(pid_file)
        .working_directory(cwd)
        .start()
        .map_err(|err| format_err!("Unable to start daemon: {:?}", &err))?;

    start(procfile, writer)
}

/// Start all the processes described in the `Procfile`.
pub fn start<W: Write>(procfile: &Procfile, writer: &mut W) -> Result<()> {
    let services = procfile.read_services()?;
    let workers: Vec<Result<ServiceWorker>> = services
        .into_iter()
        .map(|s| ServiceWorker::new(s))
        .map(|mut w| w.start().map(|_| w))
        .collect();

    let mut result = None;
    let erred = workers.iter().any(|r| r.is_err());

    for w in workers {
        match w {
            Ok(mut worker) => {
                if erred {
                    let _ = worker.kill();
                }
                let _ = worker.join();
            }
            Err(err) => {
                let _ = writer.write_fmt(format_args!("Error starting process: {:?}", &err));
                result = result.or(Some(Err(err)));
            }
        }
    }

    result.unwrap_or(Ok(()))
}

/// Read the processes in the `Procfile` and write them back out as YAML.
pub fn yamlize<W: Write>(procfile: &Procfile, writer: &mut W) -> Result<()> {
    let services = procfile.read_services()?;
    let index = service::index_services(&services);
    let yaml = serde_yaml::to_string(&index)
        .map_err(|err| format_err!("Cannot convert index to YAML: {}", &err))?;

    writer
        .write_fmt(format_args!("{}", yaml))
        .map_err(|err| format_err!("Cannot write YAML: {:?}", &err))
}

#[cfg(test)]
mod test;
