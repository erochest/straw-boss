use service::service::{run, Service};
use service::{TaskMessage, TaskResponse};
use std::process::Output;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;
use Result;

pub trait Worker {
    fn start(&mut self) -> Result<()>;
    fn thread_id(&self) -> Option<thread::ThreadId>;
    fn join(&mut self) -> Result<Output>;
    fn kill(&mut self) -> Result<()>;
    fn is_running(&self) -> bool;
}

/// Information about a running service task.
#[derive(Debug)]
struct RunningWorker(
    thread::JoinHandle<Result<()>>,
    Sender<TaskMessage>,
    Receiver<TaskResponse>,
);

impl RunningWorker {
    /// Message to wait until the task finishes, then receive the task's `ExitStatus` and return
    /// it. This consumes the `RunningWorker`.
    fn join(self, service_name: &str) -> Result<Output> {
        let RunningWorker(_, tx, rx) = self;
        tx.send(TaskMessage::Join).map_err(|err| {
            format_err!("Unable to send message to {}: {:?}", &service_name, &err)
        })?;
        let response = rx.recv().map_err(|err| {
            format_err!(
                "Unable to receive message from {}: {:?}",
                &service_name,
                &err
            )
        })?;

        let TaskResponse::Joined(output) = response;
        Ok(output)
    }
}

/// A worker. This represents a possibly running `Service`.
#[derive(Debug)]
pub struct ServiceWorker {
    service: Service,
    worker: Option<RunningWorker>,
}

impl ServiceWorker {
    /// Create a new `ServiceWorker` from a `Service`. This takes ownership of the `Service`.
    pub fn new(service: Service) -> ServiceWorker {
        ServiceWorker {
            service,
            worker: None,
        }
    }

    pub fn service(&self) -> &Service {
        &self.service
    }
}

impl Worker for ServiceWorker {
    /// Start the service executing on a separate thread.
    fn start(&mut self) -> Result<()> {
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

        self.worker = Some(RunningWorker(join_handle, manager_tx, worker_rx));

        Ok(())
    }

    /// Return the OS thread ID that the task is executing in.
    fn thread_id(&self) -> Option<thread::ThreadId> {
        match self.worker {
            Some(ref worker) => Some(worker.0.thread().id()),
            None => None,
        }
    }

    /// Wait for the task to complete and return its `ExitStatus`.
    fn join(&mut self) -> Result<Output> {
        let worker = self.worker.take();
        worker
            .ok_or_else(|| format_err!("Service {} not running.", &self.service.name))
            .and_then(|running| running.join(&self.service.name))
    }

    /// Kill the task. Doesn't wait for it to actually finish, but you lose any relationship to it.
    /// This is really a last resort.
    fn kill(&mut self) -> Result<()> {
        let worker = self.worker.take();
        if let Some(RunningWorker(_, ref tx, _)) = worker {
            tx.send(TaskMessage::Kill).map_err(|err| {
                format_err!("Error sending KILL to {}: {:?}", &self.service.name, &err)
            })
        } else {
            Ok(())
        }
    }

    /// Is this task running?
    fn is_running(&self) -> bool {
        self.worker.is_some()
    }
}

impl Drop for ServiceWorker {
    fn drop(&mut self) {
        let _ = self.kill();
    }
}

#[cfg(test)]
mod test;
