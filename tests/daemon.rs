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
fn test_daemon() {
    // The test here is really that we're not spawning this into another thread.
    let status = Command::main_binary()
        .unwrap()
        .env("STRAWBOSS_PID_FILE", "/tmp/straw-boss.test-daemon.pid")
        .arg("start")
        .arg("--procfile")
        .arg("./fixtures/Procfile.python")
        .arg("--daemon")
        .status()
        .unwrap();

    let process_info = poll_processes("http.server", "3040", 10);
    process_info.as_ref().map(|p| p.kill(sysinfo::Signal::Kill));

    assert_that(&status.success()).is_true();
    assert_that(&process_info).is_some();
}
