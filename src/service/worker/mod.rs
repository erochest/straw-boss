use Result;
use service::service::Service;
use service::{TaskMessage, TaskResponse};
use std::convert::TryFrom;
use std::process::{Command, ExitStatus};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;

#[derive(Debug)]
enum Worker {
    Stopped,
    Running(
        thread::JoinHandle<Result<()>>,
        Sender<TaskMessage>,
        Receiver<TaskResponse>,
    ),
}

#[derive(Debug)]
pub struct ServiceWorker {
    service: Service,
    worker: Worker,
}

impl ServiceWorker {
    pub fn new(service: Service) -> ServiceWorker {
        ServiceWorker {
            service,
            worker: Worker::Stopped,
        }
    }

    pub fn start(&mut self) -> Result<()> {
        let service_run = self.service.clone();
        let service_name = self.service.name.clone();
        let (manager_tx, manager_rx) = channel();
        let (worker_tx, worker_rx) = channel();

        let join_handle = thread::Builder::new()
            .spawn(|| run(service_run, manager_rx, worker_tx))
            .map_err(|err| {
                format_err!(
                    "Error spawning thread for service {}: {:?}",
                    &service_name,
                    &err
                )
            })?;

        self.worker = Worker::Running(join_handle, manager_tx, worker_rx);

        Ok(())
    }

    pub fn thread_id(&self) -> Option<thread::ThreadId> {
        match self.worker {
            Worker::Running(ref join_handle, _, _) => Some(join_handle.thread().id()),
            Worker::Stopped => None,
        }
    }

    pub fn join(&mut self) -> Result<ExitStatus> {
        match self.worker {
            Worker::Stopped => Err(format_err!("Service {} not running.", &self.service.name)),
            Worker::Running(_, ref tx, ref rx) => {
                tx.send(TaskMessage::Join)
                    .map_err(|err| format_err!("Unable to send message to {}: {:?}", &self.service.name, &err))?;
                let response = rx.recv()
                    .map_err(|err| format_err!("Unable to receive message from {}: {:?}", &self.service.name, &err))?;
                match response {
                    TaskResponse::Joined(status) => status,
                    _ => Err(format_err!("Invalid response to `Join` on {}: {:?}", &self.service.name, &response)),
                }
            },
        }
    }
}

fn run(service: Service, rx: Receiver<TaskMessage>, tx: Sender<TaskResponse>) -> Result<()> {
    let service_name = service.name.clone();
    let mut child = Command::try_from(service)?.spawn()?;

    for message in rx {
        match message {
            TaskMessage::ThreadId => {
                let thread_id = thread::current().id();
                tx.send(TaskResponse::ThreadId(thread_id)).map_err(|err| {
                    format_err!("Error while running service {}: {:?}", &service_name, &err)
                })?;
            }
            TaskMessage::Join => {
                let result = child.wait().map_err(|err| {
                    format_err!("Error waiting for service {}: {:?}", &service_name, &err)
                });
                tx.send(TaskResponse::Joined(result)).map_err(|err| {
                    format_err!(
                        "Error while sending wait for service {}: {:?}",
                        &service_name,
                        &err
                    )
                })?;
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod test;
