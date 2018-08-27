use client::rest::RestManagerClient;
use client::ManagerClient;
use server::rest::RestManagerServer;
use server::{ManagerServer, Worker};
use service::service::Service;
use service::worker::ServiceWorker;
use spectral::prelude::*;
use std::fs;
use std::path::PathBuf;
use std::thread;
use std::time::Duration;

fn setup(name: &str) -> PathBuf {
    let socket_path = PathBuf::from(format!("/tmp/straw-boss-server.{}.sock", name));
    if socket_path.exists() {
        fs::remove_file(&socket_path).unwrap();
    }
    socket_path
}

#[test]
fn test_opens_domain_socket_for_ipc() {
    let socket_path = setup("test_opens_domain_socket_for_ips");

    let mut server = RestManagerServer::at_path(socket_path.clone());
    server.initialize().unwrap();

    assert_that(&socket_path).exists();
}

#[test]
fn test_removes_domain_socket_when_done() {
    let socket_path = setup("test_removes_domain_socket_when_done");
    let server_socket = socket_path.clone();

    {
        let mut server = RestManagerServer::at_path(server_socket);
        server.initialize().unwrap();
        assert_that(&socket_path).exists();
    }

    assert_that(&socket_path).does_not_exist();
}

#[test]
fn test_get_workers_response_with_workers() {
    let socket_path = setup("test_get_workers_response_with_workers");
    let server_socket = socket_path.clone();

    let handle = thread::spawn(|| {
        let mut server = RestManagerServer::at_path(server_socket);
        server.set_workers(vec![ServiceWorker::new(Service::new(
            "python",
            "python3 -m http.server 3040",
        ))]);
        server.initialize().unwrap();
        server.start().unwrap();
    });

    thread::sleep(Duration::from_secs(1));
    assert_that(&socket_path).exists();

    let client = RestManagerClient::at_path(socket_path.clone());
    let workers = client.get_workers();
    assert_that(&workers)
        .is_ok()
        .is_equal_to(&vec![Worker::new("python", "python3 -m http.server 3040")]);

    assert_that(&client.stop_server()).is_ok();
    assert_that(&handle.join()).is_ok();
    assert_that(&socket_path).does_not_exist();
}

#[test]
fn test_exits_when_quit() {
    let socket_path = setup("test_exits_when_quit");
    let server_socket = socket_path.clone();

    let handle = thread::spawn(move || {
        let mut server = RestManagerServer::at_path(server_socket);
        server.initialize().unwrap();
        server.start().unwrap();
    });

    thread::sleep(Duration::from_secs(1));
    assert_that(&socket_path).exists();

    let client = RestManagerClient::at_path(socket_path.clone());
    assert_that(&client.stop_server()).is_ok();
    assert_that(&handle.join()).is_ok();
    assert_that(&socket_path).does_not_exist();
}
