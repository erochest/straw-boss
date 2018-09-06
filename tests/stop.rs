extern crate assert_cmd;
#[macro_use]
extern crate failure;
extern crate spectral;
extern crate straw_boss;
extern crate sysinfo;

use spectral::prelude::*;
use std::thread;
use std::time::Duration;

mod utils;

use utils::command::StopServer;
use utils::poll;

#[test]
fn test_stop() {
    let mut server = StopServer::new("test-stop");
    server.start("./fixtures/Procfile.python").unwrap();

    let process_info = poll::poll_processes("http.server", "3040", 10);
    assert_that(&process_info).is_some();

    let _ = server.stop().unwrap();
    thread::sleep(Duration::from_secs(2));

    let process_info = poll::poll_processes("http.server", "3040", 10);
    assert_that(&process_info).is_none();
}
