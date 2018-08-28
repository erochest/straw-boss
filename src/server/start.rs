use daemonize::Daemonize;
use procfile::Procfile;
use server::rest::RestManagerServer;
use server::{ManagerServer, ServerRunMode};
use service::worker::ServiceWorker;
use std::env;
use std::io::Write;
use std::path::PathBuf;
use Result;

/// Start all the processes described in the `Procfile`.
///
/// If `is_daemon` is given, the server is started in the background, and this
/// function returns immediately.
pub fn start<W: Write>(
    procfile: &Procfile,
    _writer: &mut W,
    run_mode: &ServerRunMode,
    socket_domain: &str,
) -> Result<()> {
    if let ServerRunMode::Daemon(ref pid_file) = run_mode {
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
