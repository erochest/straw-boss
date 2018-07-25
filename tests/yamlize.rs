extern crate spectral;
extern crate straw_boss;

use self::spectral::assert_that;
use self::spectral::prelude::*;
use std::path::PathBuf;
use straw_boss::actions::{yamlize, Procfile};

#[test]
fn test_writes_services_to_output() {
    let fixture = Procfile::new(PathBuf::from("./fixtures/Procfile"));
    let mut buffer: Vec<u8> = Vec::with_capacity(1024);

    let result = yamlize(&fixture, &mut buffer);
    assert_that(&result).is_ok();

    let output = String::from_utf8(buffer).unwrap();
    assert_that(&output).contains("error:");
    assert_that(&output).contains("ruby ./error");
    assert_that(&output).contains("utf8:");
    assert_that(&output).contains("ruby ./utf8");
    assert_that(&output).contains("ticker:");
    assert_that(&output).contains("ruby ./ticker $PORT");
    assert_that(&output).contains("spawner:");
    assert_that(&output).contains("./spawner");
}
