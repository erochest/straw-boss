extern crate assert_cmd;
extern crate spectral;
extern crate straw_boss;
extern crate sysinfo;

use assert_cmd::prelude::*;
use spectral::assert_that;
use spectral::prelude::*;
use std::process::Command;
use std::thread;
use std::time;
use sysinfo::{Process, ProcessExt, System, SystemExt};

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

fn poll_processes(module: &str, port: &str, times: u8) -> Option<Process> {
    if times == 0 {
        return None;
    }
    thread::sleep(time::Duration::from_secs(1));

    let mut system = System::new();
    system.refresh_all();

    system
        .get_process_list()
        .into_iter()
        .map(|p| p.1)
        .filter(|p| command_contains(p.cmd(), module))
        .filter(|p| command_contains(p.cmd(), port))
        .nth(0)
        .clone()
        .map(|p| p.clone())
        .or_else(|| poll_processes(module, port, times - 1))
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

fn command_contains(parts: &[String], part: &str) -> bool {
    let part = String::from(part);
    parts.iter().filter(|p| **p == part).nth(0).is_some()
}
