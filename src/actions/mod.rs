use daemonize::Daemonize;
use serde_yaml;
use service::service;
use service::worker::ServiceWorker;
use std::env;
use std::fs::File;
use procfile::Procfile;
use std::io::Write;
use std::path::PathBuf;
use Result;

/// An action that the straw boss can do.
#[derive(Debug)]
pub enum Action {
    Start(Procfile, bool),
    Yamlize(Procfile),
}

impl Action {
    /// Execute an action. This dispatches to the appropriate function to take the action
    /// described. It writes its output to the `Write` implementor passed in.
    pub fn execute<W: Write>(&self, writer: &mut W) -> Result<()> {
        match *self {
            Action::Start(ref procfile, daemon) => start(procfile, writer, daemon),
            Action::Yamlize(ref procfile) => yamlize(procfile, writer),
        }
    }
}

/// Start all the processes described in the `Procfile`.
///
/// If `is_daemon` is given, the server is started in the background, and this
/// function returns immediately.
pub fn start<W: Write>(procfile: &Procfile, writer: &mut W, is_daemon: bool) -> Result<()> {
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
