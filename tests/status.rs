extern crate assert_cmd;
extern crate spectral;
extern crate straw_boss;
extern crate sysinfo;

use assert_cmd::prelude::*;
use spectral::assert_that;
use spectral::prelude::*;
use std::process::Command;
use sysinfo::ProcessExt;

mod utils;

use utils::poll_processes;

#[test]
fn test_status() {
    let status = Command::main_binary()
        .unwrap()
        .arg("start")
        .arg("--procfile")
        .arg("./fixtures/Procfile.python")
        .arg("--daemon")
        .status()
        .unwrap();

    assert_that(&status.success()).is_true();

    let command = Command::main_binary()
        .unwrap()
        .arg("status")
        .output()
        .unwrap();

    let output = String::from_utf8(command.stdout.clone()).unwrap();
    command.assert().success();

    assert_that(&output).contains("python: python3 -m http.server 3040");

    let process_info = poll_processes("http.server", "3040", 10);
    process_info.as_ref().map(|p| p.kill(sysinfo::Signal::Kill));

    assert_that(&process_info).is_some();
}
