use failure::Error;
use std::process::ExitStatus;
use std::thread::ThreadId;

pub mod manager;
pub mod service;
mod worker;

#[derive(Eq, Ord, PartialEq, PartialOrd, Debug)]
pub enum TaskMessage {
    Join,
    ThreadId,
    Kill,
}

#[derive(Debug)]
pub enum TaskResponse {
    Joined(Result<ExitStatus, Error>),
    ThreadId(ThreadId),
}
