extern crate assert_cmd;
extern crate spectral;
extern crate straw_boss;
extern crate sysinfo;

use assert_cmd::prelude::*;
use spectral::assert_that;
use spectral::prelude::*;
use std::process::Command;
use std::thread;
use sysinfo::ProcessExt;

mod utils;

use utils::poll_processes;

#[test]
fn test_start() {
    let _join = thread::spawn(move || {
        let _command = Command::main_binary()
            .unwrap()
            .arg("--procfile")
            .arg("./fixtures/Procfile.python")
            .arg("start")
            .unwrap();
    });

    let process_info = poll_processes("http.server", "3040", 10);
    process_info.as_ref().map(|p| p.kill(sysinfo::Signal::Kill));

    assert_that(&process_info).is_some();
}

#[test]
fn test_pipe_commands() {
    let command = Command::main_binary()
        .unwrap()
        .arg("--procfile")
        .arg("./fixtures/Procfile.pipe")
        .arg("start")
        .unwrap();

    let output = String::from_utf8(command.stdout.clone()).unwrap();
    command.assert().success();

    assert_that(&output).contains("2");
}
