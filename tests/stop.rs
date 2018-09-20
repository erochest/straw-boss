extern crate assert_cmd;
#[macro_use]
extern crate failure;
extern crate spectral;
extern crate straw_boss;
extern crate sysinfo;

use assert_cmd::prelude::*;
use spectral::prelude::*;
use std::fs;
use std::io::Read;
use std::path::PathBuf;
use std::thread;
use std::time::Duration;
use sysinfo::{AsU32, System, SystemExt};

mod utils;

use utils::command::StopServer;
use utils::poll;

#[test]
fn test_stop_from_start() {
    let mut server = StopServer::new("test-stop-from-start");
    server.start("./fixtures/Procfile.python").unwrap();

    let process_info = poll::poll_processes("http.server", "3040", 10);
    assert_that(&process_info).is_some();

    let _ = server.stop().unwrap();
    thread::sleep(Duration::from_secs(1));

    let process_info = poll::poll_processes("http.server", "3040", 10);
    assert_that(&process_info).is_none();
}

#[test]
fn test_stop_from_daemon() {
    // Start the server
    let mut server = StopServer::new("test-stop-from-daemon");
    server.daemonize("./fixtures/Procfile.python").unwrap();

    // Check that it started a server
    let process_info = poll::poll_processes("http.server", "3040", 10);
    assert_that(&process_info).is_some();

    thread::sleep(Duration::from_secs(2));

    // Read the PID
    let pid_file = PathBuf::from(server.pid_file());
    assert_that(&pid_file).exists();

    let mut pid_file_io = fs::File::open(&pid_file).unwrap();
    let mut pid_buffer = String::new();
    pid_file_io.read_to_string(&mut pid_buffer).unwrap();
    let server_pid: u32 = pid_buffer.parse().unwrap();

    // Stop the server and wait
    let _ = server.stop().unwrap();
    thread::sleep(Duration::from_secs(1));

    // Get the process status for the SB server
    let mut system = System::new();
    system.refresh_processes();
    let processes = system.get_process_list();
    let server_process = processes
        .into_iter()
        .filter(|(pid, _)| pid.as_u32() == server_pid)
        .nth(0);
    assert_that(&server_process).is_none();

    // Has the background server stopped?
    let process_info = poll::poll_processes("http.server", "3040", 10);
    assert_that(&process_info).is_none();

    assert_that(&pid_file).does_not_exist();
}

#[test]
fn test_stop_single_task() {
    let mut server = StopServer::new("test-stop-single-task");
    server.start("./fixtures/Procfile.two-http").unwrap();

    let process_info = poll::poll_processes("http.server", "3041", 10);
    assert_that(&process_info).is_some();
    let process_info = poll::poll_processes("http.server", "3042", 10);
    assert_that(&process_info).is_some();

    let _client = server
        .build_client()
        .unwrap()
        .arg("stop")
        .arg("--task")
        .arg("web1")
        .ok()
        .map_err(|_| format_err!("Unable to stop task: web1"))
        .unwrap();

    let process_info = poll::poll_processes("http.server", "3041", 10);
    assert_that(&process_info).is_none();
    let process_info = poll::poll_processes("http.server", "3042", 10);
    assert_that(&process_info).is_some();

    let _ = server.stop().unwrap();
    thread::sleep(Duration::from_secs(1));
}
