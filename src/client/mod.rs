use service::Service;
use std::collections::HashMap;
use tasks::TaskSpec;
use Result;

pub mod local;
pub mod status;

pub trait ManagerClient {
    fn is_running(&self) -> bool;
    fn get_workers(&self) -> Result<Vec<Service>>;
    fn stop(&self, TaskSpec) -> Result<()>;
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum ManagerStatus {
    NotFound,
    RunningTasks(HashMap<String, String>),
}

impl ManagerStatus {
    pub fn get_message(&self) -> String {
        match self {
            ManagerStatus::NotFound => String::from(
                "Straw-boss not running. Why don't you try `straw-boss start --daemon`",
            ),
            ManagerStatus::RunningTasks(tasks) => tasks
                .into_iter()
                .map(|(k, v)| format!("{}: {}\n", &k, &v))
                .fold(String::new(), |a, b| a + &b),
        }
    }
}
