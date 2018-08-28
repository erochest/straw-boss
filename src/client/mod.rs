use server::Worker;
use std::collections::HashMap;
use Result;

pub mod rest;
pub mod status;

pub trait ManagerClient {
    fn is_running(&self) -> bool;
    fn get_workers(&self) -> Result<Vec<Worker>>;
    fn stop_server(&self) -> Result<()>;
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum ManagerStatus {
    NotFound,
    RunningTasks(HashMap<String, String>),
}
