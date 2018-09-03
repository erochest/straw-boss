use server::{ManagerServer, ServerRunMode};
use service::service::Service;
use Result;

/// Start all the processes described in the `Procfile`.
///
/// If `is_daemon` is given, the server is started in the background, and this
/// function returns immediately.
pub fn start(
    server: &mut impl ManagerServer,
    run_mode: ServerRunMode,
    workers: Vec<Service>,
) -> Result<()> {
    if let ServerRunMode::Daemon(pid_file) = run_mode {
        server.daemonize(pid_file)?;
    }

    server.start_workers(workers)?;
    server.start_server()
}

#[cfg(test)]
mod test {
    use super::start;
    use server::ManagerServer;
    use server::ServerRunMode;
    use service::service::Service;
    use spectral::prelude::*;
    use std::path::Path;
    use std::sync::RwLock;
    use Result;

    #[derive(Debug, PartialEq)]
    enum ServerCalls {
        Daemonize,
        StartServer,
        StartWorkers,
    }

    struct MockServer {
        calls: RwLock<Vec<ServerCalls>>,
    }

    impl MockServer {
        fn new() -> MockServer {
            MockServer {
                calls: RwLock::new(Vec::new()),
            }
        }

        fn push(&self, call: ServerCalls) -> Result<()> {
            let mut calls = self
                .calls
                .write()
                .map_err(|e| format_err!("Unable to write to calls: {:?}", &e))?;
            (*calls).push(call);
            Ok(())
        }
    }

    impl ManagerServer for MockServer {
        fn daemonize<P: AsRef<Path>>(&self, _pid_file: P) -> Result<()> {
            self.push(ServerCalls::Daemonize)
        }

        fn start_server(&mut self) -> Result<()> {
            self.push(ServerCalls::StartServer)
        }

        fn start_workers(&mut self, _workers: Vec<Service>) -> Result<()> {
            self.push(ServerCalls::StartWorkers)
        }
    }

    #[test]
    fn test_daemon_mode_calls_daemonize() {
        let mut server = MockServer::new();
        assert_that(&start(
            &mut server,
            ServerRunMode::Daemon("/dev/null".into()),
            Vec::new(),
        )).is_ok();
        let calls = server.calls.read().unwrap();
        assert_that(&*calls).contains(ServerCalls::Daemonize);
    }

    #[test]
    fn test_foreground_does_not_call_daemonize() {
        let mut server = MockServer::new();
        assert_that(&start(&mut server, ServerRunMode::Foreground, Vec::new())).is_ok();
        let calls = server.calls.read().unwrap();
        assert_that(&*calls).does_not_contain(ServerCalls::Daemonize);
    }

    #[test]
    fn test_starts_workers() {
        let mut server = MockServer::new();
        assert_that(&start(&mut server, ServerRunMode::Foreground, Vec::new())).is_ok();
        let calls = server.calls.read().unwrap();
        assert_that(&*calls).contains(ServerCalls::StartWorkers);
    }

    #[test]
    fn test_starts_server() {
        let mut server = MockServer::new();
        assert_that(&start(&mut server, ServerRunMode::Foreground, Vec::new())).is_ok();
        let calls = server.calls.read().unwrap();
        assert_that(&*calls).contains(ServerCalls::StartServer);
    }
}
