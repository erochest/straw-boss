use std::thread;
use std::time;
use sysinfo::{Process, ProcessExt, System, SystemExt};

pub fn poll_processes(module: &str, port: &str, times: u8) -> Option<Process> {
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

fn command_contains(parts: &[String], part: &str) -> bool {
    let part = String::from(part);
    parts.iter().filter(|p| **p == part).nth(0).is_some()
}
