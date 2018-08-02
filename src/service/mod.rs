use std::process::Output;

pub mod service;
pub mod worker;

/// Messages to the service workers.
#[derive(Eq, Ord, PartialEq, PartialOrd, Debug)]
pub enum TaskMessage {
    /// Wait for the thing to finish.
    Join,
    /// Kill the running service.
    Kill,
}

/// Response to messages.
#[derive(Debug)]
pub enum TaskResponse {
    /// The result of the running process after it's finished.
    Joined(Output),
}
