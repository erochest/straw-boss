use messaging::{Receiver, Sender};
use server::RequestMessage;
use server::RequestMessage::*;
use server::ResponseMessage::*;
use service::service::Service;
use std::fs;
use std::os::unix::net::UnixListener;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};

struct DisappearingFile(PathBuf);

impl Drop for DisappearingFile {
    fn drop(&mut self) {
        if self.0.exists() {
            let _ = fs::remove_file(&self.0);
        }
    }
}

struct MockServer {
    pub socket_path: PathBuf,
    pub workers: Vec<Service>,
    pub calls: Arc<RwLock<Vec<RequestMessage>>>,
}

impl MockServer {
    fn new<P: AsRef<Path>>(
        socket_path: P,
        workers: Vec<Service>,
        calls: Arc<RwLock<Vec<RequestMessage>>>,
    ) -> MockServer {
        let socket_path = socket_path.as_ref().to_path_buf();
        MockServer {
            socket_path,
            workers,
            calls,
        }
    }

    fn run(&mut self) {
        let socket = &self.socket_path;
        if socket.exists() {
            fs::remove_file(socket).unwrap();
        }

        let listener = UnixListener::bind(socket).unwrap();
        for stream in listener.incoming() {
            let mut stream = stream.unwrap();
            let request: RequestMessage = stream.recv().unwrap();
            match request {
                GetWorkers => {
                    {
                        let mut calls = self.calls.write().unwrap();
                        calls.push(GetWorkers);
                    }
                    let response = Workers(self.workers.clone());
                    stream.send(response).unwrap();
                }
                stop_message => {
                    let mut calls = self.calls.write().unwrap();
                    calls.push(stop_message);
                    return;
                }
            }
        }
    }
}

// TODO: Make this a random path
fn make_socket_name(name: &str) -> PathBuf {
    let socket_name = PathBuf::from(format!("/tmp/straw-boss.{}.sock", &name));
    if socket_name.exists() {
        fs::remove_file(&socket_name).unwrap();
    }
    socket_name
}

mod is_running {
    use super::super::super::ManagerClient;
    use super::super::RestManagerClient;
    use super::{make_socket_name, DisappearingFile};
    use spectral::prelude::*;
    use std::fs;

    #[test]
    fn test_returns_false_if_no_server() {
        let socket_path = make_socket_name("test_returns_false_if_no_server");
        let client = RestManagerClient::at_path(socket_path);
        assert_that(&client.is_running()).is_false();
    }

    #[test]
    fn test_returns_true_if_server() {
        let socket_path = make_socket_name("test_returns_true_if_server");
        let socket = DisappearingFile(socket_path.clone());
        let _ = fs::File::create(&socket.0).unwrap();
        let client = RestManagerClient::at_path(socket_path.clone());
        assert_that(&client.is_running()).is_true();
    }
}

mod get_workers {
    use super::{make_socket_name, MockServer};
    use client::local::RestManagerClient;
    use client::ManagerClient;
    use rmp_serde::Serializer;
    use serde::Serialize;
    use server::RequestMessage::*;
    use service::service::Service;
    use spectral::prelude::*;
    use std::os::unix::net::UnixStream;
    use std::path::Path;
    use std::sync::{Arc, RwLock};
    use std::thread;
    use std::time::Duration;
    use Result;

    fn stop_server<P: AsRef<Path>>(socket_path: P) -> Result<()> {
        let socket_path = socket_path.as_ref();
        let mut stream = UnixStream::connect(socket_path)
            .map_err(|err| format_err!("Unable to connect to {:?}: {:?}", socket_path, &err))?;
        let mut ser = Serializer::new(&mut stream);
        StopServer
            .serialize(&mut ser)
            .map_err(|err| format_err!("Unable to send Quit: {:?}", &err))
    }

    #[test]
    fn test_calls_server() {
        let socket_path = make_socket_name("test_calls_server");
        let server_socket_path = socket_path.clone();
        let calls = Arc::new(RwLock::new(vec![]));
        let server_calls = calls.clone();
        let client = RestManagerClient::at_path(socket_path.clone());

        let handle = thread::spawn(move || {
            let mut server = MockServer::new(server_socket_path, vec![], server_calls);
            server.run();
        });

        thread::sleep(Duration::from_secs(1));
        assert_that(&client.get_workers()).is_ok();
        {
            let calls = calls.read().unwrap();
            assert_that(&calls[0]).is_equal_to(&GetWorkers);
        }

        stop_server(&socket_path).unwrap();
        handle.join().unwrap();
    }

    #[test]
    fn test_returns_err_if_no_server() {
        let socket_path = make_socket_name("test_returns_err_if_no_server");
        let client = RestManagerClient::at_path(socket_path);
        assert_that(&client.get_workers()).is_err();
    }

    #[test]
    fn test_returns_list_workers() {
        let socket_path = make_socket_name("test_returns_list_workers");
        let server_socket_path = socket_path.clone();
        let calls = Arc::new(RwLock::new(vec![]));
        let server_calls = calls.clone();
        let client = RestManagerClient::at_path(socket_path.clone());
        let workers = vec![Service::new("web", "spawn server")];

        let handle = thread::spawn(move || {
            let mut server = MockServer::new(server_socket_path, workers, server_calls);
            server.run();
        });

        thread::sleep(Duration::from_secs(1));
        assert_that(&client.get_workers())
            .is_ok_containing(vec![Service::new("web", "spawn server")]);
        {
            let calls = calls.read().unwrap();
            assert_that(&calls[0]).is_equal_to(&GetWorkers);
        }

        assert_that(&stop_server(&socket_path)).is_ok();
        assert_that(&handle.join()).is_ok();
    }
}

mod stop {
    use super::{make_socket_name, MockServer};
    use client::local::RestManagerClient;
    use client::ManagerClient;
    use server::RequestMessage::*;
    use spectral::prelude::*;
    use std::sync::{Arc, RwLock};
    use std::thread;
    use std::time::Duration;
    use tasks::TaskSpec;

    #[test]
    fn test_sends_stop_server() {
        let socket_path = make_socket_name("test_sends_stop_server");
        let server_socket_path = socket_path.clone();
        let calls = Arc::new(RwLock::new(vec![]));
        let server_calls = calls.clone();
        let client = RestManagerClient::at_path(socket_path.clone());
        let workers = vec![];

        let handle = thread::spawn(move || {
            let mut server = MockServer::new(server_socket_path, workers, server_calls);
            server.run();
        });

        thread::sleep(Duration::from_secs(1));
        assert_that(&client.stop(TaskSpec::All)).is_ok();
        assert_that(&handle.join()).is_ok();

        let calls = calls.read().unwrap();
        assert_that(&calls[0]).is_equal_to(&StopServer);
    }

    #[test]
    fn test_sends_stop_task() {
        let socket_path = make_socket_name("test_sends_stop_task");
        let server_socket_path = socket_path.clone();
        let calls = Arc::new(RwLock::new(vec![]));
        let server_calls = calls.clone();
        let client = RestManagerClient::at_path(socket_path.clone());
        let workers = vec![];

        let handle = thread::spawn(move || {
            let mut server = MockServer::new(server_socket_path, workers, server_calls);
            server.run();
        });

        thread::sleep(Duration::from_secs(1));
        assert_that(&client.stop(TaskSpec::List(vec![
            String::from("web1"),
            String::from("web2"),
        ]))).is_ok();
        assert_that(&handle.join()).is_ok();

        let calls = calls.read().unwrap();
        assert_that(&calls[0])
            .is_equal_to(&StopTasks(vec![String::from("web1"), String::from("web2")]));
    }
}
