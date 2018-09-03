extern crate assert_cmd;
extern crate spectral;
extern crate straw_boss;
extern crate sysinfo;

use assert_cmd::prelude::*;
use spectral::assert_that;
use spectral::prelude::*;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::thread;
use sysinfo::ProcessExt;

mod utils;

use utils::poll_processes;

#[test]
fn test_start() {
    let socket = PathBuf::from("/tmp/straw-boss.test-start.sock");
    if socket.exists() {
        fs::remove_file(&socket).unwrap();
    }

    let _join = thread::spawn(move || {
        let _command = Command::main_binary()
            .unwrap()
            .env(
                "STRAWBOSS_SOCKET_PATH",
                String::from(socket.to_string_lossy()),
            ).arg("start")
            .arg("--procfile")
            .arg("./fixtures/Procfile.python")
            .unwrap();
    });

    let process_info = poll_processes("http.server", "3040", 10);
    process_info.as_ref().map(|p| p.kill(sysinfo::Signal::Kill));

    assert_that(&process_info).is_some();
}

#[test]
fn test_pipe_commands() {
    let socket = PathBuf::from("/tmp/straw-boss.test-pipe-commands.sock");
    if socket.exists() {
        fs::remove_file(&socket).unwrap();
    }

    let command = Command::main_binary()
        .unwrap()
        .env(
            "STRAWBOSS_SOCKET_PATH",
            String::from(socket.to_string_lossy()),
        ).arg("start")
        .arg("--procfile")
        .arg("./fixtures/Procfile.pipe")
        .unwrap();

    let output = String::from_utf8(command.stdout.clone()).unwrap();
    command.assert().success();

    assert_that(&output).contains("2");
}
