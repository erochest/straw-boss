use std::collections::HashMap;
use std::convert::TryFrom;
use std::fmt::Debug;
use std::io;
use std::io::BufRead;
use std::iter::FromIterator;
use std::process::{Command, ExitStatus};
use std::str::FromStr;
use std::sync::Arc;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread::{current, Builder, JoinHandle, ThreadId};
use failure::Error;
use shellwords;

pub mod manager;
pub mod service;

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
