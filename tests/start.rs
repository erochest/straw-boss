extern crate assert_cmd;
#[macro_use]
extern crate failure;
extern crate spectral;
extern crate straw_boss;
extern crate sysinfo;

use assert_cmd::prelude::*;
use spectral::assert_that;
use spectral::prelude::*;
use std::thread;
use std::time::Duration;

mod utils;

use utils::command::StopServer;
use utils::poll::poll_processes;

#[test]
fn test_start() {
    let mut stop_server = StopServer::new("test-start");
    stop_server.start("./fixtures/Procfile.python").unwrap();
    let process_info = poll_processes("http.server", "3040", 10);
    assert_that(&process_info).is_some();
}

#[test]
fn test_pipe_commands() {
    let mut stop_server = StopServer::new("test-pipe-commands");
    stop_server.start("./fixtures/Procfile.pipe").unwrap();
    thread::sleep(Duration::from_secs(2));
    let output = stop_server.stop().unwrap();

    let text_output = String::from_utf8(output.stdout.clone()).unwrap();
    output.assert().success();

    assert_that(&text_output).contains("2");
}
