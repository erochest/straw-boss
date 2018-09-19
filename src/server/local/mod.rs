use messaging::{Receiver, Sender};
use server::{daemonize, ManagerServer, RequestMessage, ResponseMessage};
use service::service::Service;
use service::worker::{ServiceWorker, Worker};
use std::fs;
use std::os::unix::net::UnixListener;
use std::path::{Path, PathBuf};
use Result;

pub const DOMAIN_SOCKET: &'static str = "/tmp/straw-boss-server.sock";

pub struct RestManagerServer {
    socket_path: PathBuf,
    pid_file: Option<PathBuf>,
    listener: Option<UnixListener>,
    workers: Vec<ServiceWorker>,
}

impl RestManagerServer {
    pub fn new() -> RestManagerServer {
        RestManagerServer::at_path(PathBuf::from(DOMAIN_SOCKET))
    }

    pub fn at_path(socket_path: PathBuf) -> RestManagerServer {
        RestManagerServer {
            socket_path,
            pid_file: None,
            listener: None,
            workers: vec![],
        }
    }

    fn initialize(&mut self) -> Result<()> {
        let listener = UnixListener::bind(&self.socket_path).map_err(|err| {
            format_err!("Unable to open socket: {:?}: {:?}", &self.socket_path, &err)
        })?;
        self.listener = Some(listener);

        Ok(())
    }
}

impl ManagerServer for RestManagerServer {
    fn daemonize<P: AsRef<Path>>(&mut self, pid_file: P) -> Result<()> {
        daemonize(&pid_file)?;
        self.pid_file = Some(PathBuf::from(pid_file.as_ref()));
        Ok(())
    }

    fn start_workers(&mut self, workers: Vec<Service>) -> Result<()> {
        self.workers = workers
            .into_iter()
            .map(ServiceWorker::new)
            .map(|mut w| w.start().and(Ok(w)))
            .collect::<Result<Vec<_>>>()?;
        Ok(())
    }

    fn start_server(&mut self) -> Result<()> {
        self.initialize()?;
        self.listener.iter().try_for_each(|listener| {
            for stream in listener.incoming() {
                let mut stream = stream
                    .map_err(|err| format_err!("Unable to read from listener: {:?}", &err))?;
                let request: RequestMessage = stream.recv()?;
                match request {
                    RequestMessage::GetWorkers => {
                        let response = ResponseMessage::Workers(
                            self.workers.iter().map(|sw| sw.service().clone()).collect(),
                        );
                        stream.send(response)?;
                    }
                    RequestMessage::Stop => break,
                }
            }
            Ok(())
        })
    }
}

impl Drop for RestManagerServer {
    fn drop(&mut self) {
        self.listener.take().into_iter().for_each(|_| {
            if self.socket_path.exists() {
                // Eating the error b/c we're trying to shutdown.
                let _ = fs::remove_file(&self.socket_path);
            }
        });
        self.pid_file.take().into_iter().for_each(|pid_file| {
            if pid_file.exists() {
                let _ = fs::remove_file(pid_file);
            }
        });
    }
}

#[cfg(test)]
mod test;
