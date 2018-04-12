use Result;
use service::service::Service;
use service::{TaskMessage, TaskResponse};
use std::convert::TryFrom;
use std::process::{Command, ExitStatus};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;

/// Information about a running service task.
#[derive(Debug)]
struct Running(
    thread::JoinHandle<Result<()>>,
    Sender<TaskMessage>,
    Receiver<TaskResponse>,
);

impl Running {
    /// Message to wait until the task finishes, then receive the task's `ExitStatus` and return
    /// it. This consumes the `Running`.
    fn join(self, service_name: &str) -> Result<ExitStatus> {
        let Running(_, tx, rx) = self;
        tx.send(TaskMessage::Join)
            .map_err(|err| format_err!("Unable to send message to {}: {:?}", &service_name, &err))?;
        let response = rx.recv().map_err(|err| {
            format_err!(
                "Unable to receive message from {}: {:?}",
                &service_name,
                &err
            )
        })?;

        if let TaskResponse::Joined(status) = response {
            status
        } else {
            Err(format_err!(
                "Invalid response to `Join` on {}: {:?}",
                &service_name,
                &response
            ))
        }
    }
}

/// A worker. This represents a possibly running `Service`.
#[derive(Debug)]
pub struct ServiceWorker {
    service: Service,
    worker: Option<Running>,
}

impl ServiceWorker {
    /// Create a new `ServiceWorker` from a `Service`. This takes ownership of the `Service`.
    pub fn new(service: Service) -> ServiceWorker {
        ServiceWorker {
            service,
            worker: None,
        }
    }

    /// Start the service executing on a separate thread.
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

        self.worker = Some(Running(join_handle, manager_tx, worker_rx));

        Ok(())
    }

    /// Return the OS thread ID that the task is executing in.
    pub fn thread_id(&self) -> Option<thread::ThreadId> {
        match self.worker {
            Some(ref worker) => Some(worker.0.thread().id()),
            None => None,
        }
    }

    /// Return the OS process ID that the task is executing in.
    pub fn process_id(&self) -> Option<u32> {
        if let Some(Running(_, ref tx, ref rx)) = self.worker {
            tx.send(TaskMessage::ProcessId).ok()?;
            let reply = rx.recv().ok()?;
            if let TaskResponse::ProcessId(pid) = reply {
                Some(pid)
            } else {
                None
            }
        } else {
            None
        }
    }

    /// Wait for the task to complete and return its `ExitStatus`.
    pub fn join(&mut self) -> Result<ExitStatus> {
        let worker = self.worker.take();
        worker
            .ok_or_else(|| format_err!("Service {} not running.", &self.service.name))
            .and_then(|running| running.join(&self.service.name))
    }

    /// Kill the task. Doesn't wait for it to actually finish, but you lose any relationship to it.
    /// This is really a last resort.
    pub fn kill(&mut self) -> Result<()> {
        let worker = self.worker.take();
        if let Some(Running(_, ref tx, _)) = worker {
            tx.send(TaskMessage::Kill).map_err(|err| {
                format_err!("Error sending KILL to {}: {:?}", &self.service.name, &err)
            })
        } else {
            Ok(())
        }
    }

    /// Is this task running?
    pub fn is_running(&self) -> bool {
        self.worker.is_some()
    }
}

impl Drop for ServiceWorker {
    fn drop(&mut self) {
        let _ = self.kill();
    }
}

/// This takes the channels to communicate over and the service to run, and it executes the
/// service. This is meant to be run in a new thread.
fn run(service: Service, rx: Receiver<TaskMessage>, tx: Sender<TaskResponse>) -> Result<()> {
    let service_name = service.name.clone();
    let mut child = Command::try_from(service)?.spawn()?;

    for message in rx {
        match message {
            TaskMessage::ProcessId => {
                let _ = tx.send(TaskResponse::ProcessId(child.id())).map_err(|err| {
                    format_err!(
                        "Error sending process ID of service {}: {:?}",
                        &service_name,
                        &err
                    )
                });
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
            TaskMessage::Kill => {
                return child.kill().map_err(|err| {
                    format_err!("Error killing service {}: {:?}", &service_name, &err)
                });
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod test;
