extern crate straw_boss;

use std::path::PathBuf;
use straw_boss::actions::{yamlize, Procfile};

#[test]
fn test_writes_services_to_output() {
    let fixture = Procfile::new(PathBuf::from("./fixtures/Procfile"));
    let mut buffer: Vec<u8> = Vec::with_capacity(1024);
    let result = yamlize(&fixture, &mut buffer);
    assert!(result.is_ok());
    let output = String::from_utf8(buffer).unwrap();
    assert!(output.contains("error:"));
    assert!(output.contains("ruby ./error"));
    assert!(output.contains("utf8:"));
    assert!(output.contains("ruby ./utf8"));
    assert!(output.contains("ticker:"));
    assert!(output.contains("ruby ./ticker $PORT"));
    assert!(output.contains("spawner:"));
    assert!(output.contains("./spawner"));
}
