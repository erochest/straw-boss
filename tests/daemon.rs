extern crate assert_cmd;
#[macro_use]
extern crate failure;
extern crate spectral;
extern crate straw_boss;
extern crate sysinfo;

use spectral::assert_that;
use spectral::prelude::*;

mod utils;

use utils::command::StopServer;
use utils::poll::poll_processes;

#[test]
fn test_daemon() {
    let mut server = StopServer::new("test-daemon");
    server.daemonize("./fixtures/Procfile.python").unwrap();

    let process_info = poll_processes("http.server", "3040", 10);
    assert_that(&process_info).is_some();

    let output = server.stop().unwrap();
    let status = output.status;
    assert_that(&status.success()).is_true();
}
