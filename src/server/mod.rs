use service::service::Service;
use service::worker::ServiceWorker;
use std::path::PathBuf;
use Result;

pub mod rest;
pub mod start;

#[derive(Debug)]
pub enum ServerRunMode {
    Foreground,
    Daemon(PathBuf),
}

pub trait ManagerServer {
    fn start(&self) -> Result<()>;
}

#[derive(Debug, Serialize, Deserialize, PartialEq, PartialOrd)]
pub enum RequestMessage {
    Quit,
    GetWorkers,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, PartialOrd)]
pub enum ResponseMessage {
    Workers(Vec<Worker>),
    Ok,
}

#[derive(Debug, Serialize, Deserialize, Clone, Eq, Ord, PartialEq, PartialOrd)]
pub struct Worker {
    pub name: String,
    pub command: String,
}

impl Worker {
    pub fn new(name: &str, command: &str) -> Worker {
        Worker {
            name: name.into(),
            command: command.into(),
        }
    }
}

// TODO: Probably need to just use either `Service` or `Worker`.
impl From<Service> for Worker {
    fn from(service: Service) -> Worker {
        Worker {
            name: service.name,
            command: service.command,
        }
    }
}

impl From<ServiceWorker> for Worker {
    fn from(sw: ServiceWorker) -> Worker {
        Worker::from(sw.service().clone())
    }
}
