extern crate spectral;
extern crate straw_boss;
extern crate sysinfo;

use self::spectral::assert_that;
use self::spectral::prelude::*;
use self::sysinfo::{ProcessExt, System, SystemExt};
use std::thread;
use std::time;
use straw_boss::actions::{start, Procfile};

#[test]
fn test_start() {
    let procfile = Procfile::new("./fixtures/Procfile.python".into());
    let mut buffer = Vec::with_capacity(4095);
    let mut system = System::new();

    let _join = thread::spawn(move || {
        let _task = start(&procfile, &mut buffer).unwrap();
    });
    thread::sleep(time::Duration::from_secs(2));

    system.refresh_all();
    let processes = system.get_process_list();
    let processes = processes
        .into_iter()
        .filter(|p| command_contains(p.1.cmd(), "http.server"))
        .filter(|p| command_contains(p.1.cmd(), "3040"))
        .nth(0);
    processes.map(|p| p.1.kill(sysinfo::Signal::Kill));

    assert_that(&processes).is_some();
}

fn command_contains(parts: &[String], part: &str) -> bool {
    let part = String::from(part);
    parts.iter().filter(|p| **p == part).nth(0).is_some()
}
