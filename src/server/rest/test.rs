use chrono::prelude::*;
use client::rest::RestManagerClient;
use client::ManagerClient;
use reqwest;
use server::rest::RestManagerServer;
use server::ManagerServer;
use service::service::Service;
use spectral::prelude::*;
use std::env;
use std::fs;
use std::path::PathBuf;
use std::process;
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
fn test_starts_workers() {
    let socket_path = setup("test_starts_workers");
    let mut server = RestManagerServer::at_path(socket_path);
    let now = Utc::now();
    let filename = format!(
        "straw-boss.starts-workers.{}.{}",
        now.timestamp(),
        process::id()
    );
    let tmpfile = env::temp_dir().join(filename);

    if tmpfile.exists() {
        fs::remove_file(&tmpfile).unwrap();
    }

    assert_that(&server.start_workers(vec![Service::new(
        "touch",
        &format!("touch {:?}", &tmpfile),
    )])).is_ok();
    thread::sleep(Duration::from_secs(1));
    assert_that(&tmpfile).exists();
}

#[test]
fn test_get_workers_response_with_workers() {
    let socket_path = setup("test_get_workers_response_with_workers");
    let server_socket = socket_path.clone();

    let handle = thread::spawn(|| {
        let mut server = RestManagerServer::at_path(server_socket);
        server
            .start_workers(vec![Service::new("python", "python3 -m http.server 3040")])
            .unwrap();
        server.start_server().unwrap();
    });

    thread::sleep(Duration::from_secs(1));
    assert_that(&socket_path).exists();

    let client = RestManagerClient::at_path(socket_path.clone());
    let workers = client.get_workers();
    assert_that(&workers)
        .is_ok()
        .is_equal_to(&vec![Service::new("python", "python3 -m http.server 3040")]);

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
        server
            .start_workers(vec![Service::new("web", "python3 -m http.server 9874")])
            .unwrap();
        server.start_server().unwrap();
    });

    thread::sleep(Duration::from_secs(1));
    assert_that(&socket_path).exists();

    let response = reqwest::get("http://localhost:9874/").unwrap();
    assert_that(&response.status()).is_equal_to(&reqwest::StatusCode::Ok);

    let client = RestManagerClient::at_path(socket_path.clone());
    assert_that(&client.stop_server()).is_ok();
    assert_that(&handle.join()).is_ok();
}

#[test]
fn test_deletes_socket_when_quit() {
    let socket_path = setup("test_deletes_socket_when_quit");
    let server_socket = socket_path.clone();

    let handle = thread::spawn(move || {
        let mut server = RestManagerServer::at_path(server_socket);
        server.start_server().unwrap();
    });

    thread::sleep(Duration::from_secs(1));
    assert_that(&socket_path).exists();

    let client = RestManagerClient::at_path(socket_path.clone());
    client.stop_server().unwrap();
    handle.join().unwrap();
    assert_that(&socket_path).does_not_exist();
}

#[test]
fn test_stops_tasks_when_quit() {
    let socket_path = setup("test_stops_tasks_when_quit");
    let server_socket = socket_path.clone();

    let handle = thread::spawn(move || {
        let mut server = RestManagerServer::at_path(server_socket);
        server
            .start_workers(vec![Service::new("web", "python3 -m http.server 9875")])
            .unwrap();
        server.start_server().unwrap();
    });

    thread::sleep(Duration::from_secs(1));
    assert_that(&socket_path).exists();

    let client = RestManagerClient::at_path(socket_path.clone());
    client.stop_server().unwrap();
    handle.join().unwrap();

    assert_that(&reqwest::get("http://localhost:9875/")).is_err();
}
