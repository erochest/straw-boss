use messaging::{Receiver, Sender};
use server::Worker;
use server::{ManagerServer, RequestMessage, ResponseMessage};
use service::worker::ServiceWorker;
use std::fs;
use std::os::unix::net::UnixListener;
use std::path::PathBuf;
use Result;

pub const DOMAIN_SOCKET: &'static str = "/tmp/straw-boss-server.sock";

pub struct RestManagerServer {
    socket_path: PathBuf,
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
            listener: None,
            workers: vec![],
        }
    }

    pub fn initialize(&mut self) -> Result<()> {
        let listener = UnixListener::bind(&self.socket_path).map_err(|err| {
            format_err!("Unable to open socket: {:?}: {:?}", &self.socket_path, &err)
        })?;
        self.listener = Some(listener);

        Ok(())
    }

    pub fn set_workers(&mut self, workers: Vec<ServiceWorker>) {
        self.workers = workers;
    }
}

impl ManagerServer for RestManagerServer {
    fn start(&self) -> Result<()> {
        self.listener.iter().try_for_each(|listener| {
            for stream in listener.incoming() {
                let mut stream = stream
                    .map_err(|err| format_err!("Unable to read from listener: {:?}", &err))?;
                let request: RequestMessage = stream.recv()?;
                match request {
                    RequestMessage::GetWorkers => {
                        let response = ResponseMessage::Workers(
                            self.workers
                                .iter()
                                .map(|sw| {
                                    let service = sw.service();
                                    Worker::new(&service.name, &service.command)
                                }).collect(),
                        );
                        stream.send(response)?;
                    }
                    RequestMessage::Quit => break,
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
    }
}

#[cfg(test)]
mod test;
