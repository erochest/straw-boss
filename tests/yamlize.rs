extern crate assert_cmd;
extern crate spectral;
extern crate straw_boss;

use assert_cmd::prelude::*;
use spectral::assert_that;
use spectral::prelude::*;
use std::process::Command;

#[test]
fn test_writes_services_to_output() {
    let command = Command::main_binary()
        .unwrap()
        .arg("yamlize")
        .arg("--procfile")
        .arg("./fixtures/Procfile")
        .unwrap();

    let output = String::from_utf8(command.stdout.clone()).unwrap();
    command.assert().success();

    assert_that(&output).contains("error:");
    assert_that(&output).contains("ruby ./error");
    assert_that(&output).contains("utf8:");
    assert_that(&output).contains("ruby ./utf8");
    assert_that(&output).contains("ticker:");
    assert_that(&output).contains("ruby ./ticker $PORT");
    assert_that(&output).contains("spawner:");
    assert_that(&output).contains("./spawner");
}
