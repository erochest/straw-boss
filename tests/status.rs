extern crate assert_cmd;
#[macro_use]
extern crate failure;
extern crate spectral;
extern crate straw_boss;
extern crate sysinfo;

use assert_cmd::prelude::*;
use spectral::assert_that;
use spectral::prelude::*;

mod utils;

use utils::command::StopServer;
use utils::poll::poll_processes;

#[test]
fn test_status() {
    let mut server = StopServer::new("test-status");
    server.daemonize("./fixtures/Procfile.status").unwrap();

    let command = server
        .build_client()
        .unwrap()
        .arg("status")
        .output()
        .unwrap();

    let output = String::from_utf8(command.stdout.clone()).unwrap();
    command.assert().success();

    assert_that(&output).contains("python: python3 -m http.server 3040");
    assert_that(&output).contains("ls: ls fixtures");

    let process_info = poll_processes("http.server", "3040", 10);
    assert_that(&process_info).is_some();
}
