use failure::Error;
use Result;
use service::TaskMessage;
use service::TaskResponse;
use service::service::Service;
use std::fmt::Debug;
use std::process::ExitStatus;
use std::sync::Arc;
use std::sync::mpsc::channel;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::Sender;
use std::thread::Builder;
use std::thread::JoinHandle;
use std::thread::ThreadId;

#[derive(Debug)]
pub struct ServiceManager {
    service: Service,
    join_handle: Option<JoinHandle<Result<()>>>,
    tx: Sender<TaskMessage>,
    rx: Arc<Receiver<TaskResponse>>,
}

impl ServiceManager {
    pub fn start(service: Service) -> Result<ServiceManager> {
        let cloned_service = service.clone();
        let (in_tx, in_rx) = channel();
        let (out_tx, out_rx) = channel();

        let join_handle = Builder::new()
            .spawn(move || cloned_service.run(out_tx, in_rx))
            .map_err(|err| {
                format_err!(
                    "Error spawning thread for service {}: {:?}",
                    &service.name,
                    &err
                )
            })?;
        let join_handle = Some(join_handle);

        Ok(ServiceManager {
            service,
            join_handle,
            tx: in_tx,
            rx: Arc::new(out_rx),
        })
    }

    fn wrap_err_detail<D: Debug>(&self, message: &str, detail: &D) -> Error {
        self.service.wrap_err_detail(message, detail)
    }

    pub fn join(&mut self) -> Result<ExitStatus> {
        let name = &self.service.name.clone();
        let join_handle = self.join_handle
            .take()
            .ok_or_else(|| format_err!("Error joining task {}: No valid task", &name))?;

        // If the task is over, then this fails.
        self.tx
            .send(TaskMessage::Join)
            .map_err(|err| self.wrap_err_detail("Error joining task", &err))?;
        let rx = &self.rx.clone();

        let join_result = join_handle
            .join()
            .map_err(|err| format_err!("Error joining task {}: {:?}", &name, &err))?;
        join_result?;

        match rx.recv() {
            Ok(TaskResponse::Joined(exit_status)) => exit_status,
            Ok(msg) => Err(format_err!(
                "Invalid message from join on {}: {:?}",
                &name,
                &msg
            )),
            Err(err) => Err(format_err!("Error from join on {}: {:?}", &name, &err)),
        }
    }

    pub fn thread_id(&self) -> Result<ThreadId> {
        self.tx
            .send(TaskMessage::ThreadId)
            .map_err(|err| format_err!("Unable to request thread ID: {:?}", &err))?;
        match self.rx.recv() {
            Ok(TaskResponse::ThreadId(thread_id)) => Ok(thread_id),
            Ok(msg) => Err(format_err!(
                "Invalid message from thread_id on {}: {:?}",
                &self.service.name,
                &msg
            )),
            Err(err) => Err(format_err!(
                "Error from thread.id() on {}: {:?}",
                &self.service.name,
                &err
            )),
        }
    }

    pub fn process_id(&self) -> u32 {
        unimplemented!()
    }

    pub fn kill(&self) -> Result<()> {
        self.tx
            .send(TaskMessage::Kill)
            .map_err(|err| format_err!("Unable to send KILL message: {:?}", &err))
    }

    pub fn wait(&mut self) -> Result<()> {
        unimplemented!()
    }
}

impl Drop for ServiceManager {
    fn drop(&mut self) {
        if self.join_handle.is_some() {
            // Yes. We're ignoring this error. If it's already died, I don't need to kill it.
            let _result = self.kill();
        }
    }
}

#[cfg(test)]
mod test {}
