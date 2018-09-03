use client::ManagerClient;
use messaging::{connect, Receiver, Sender};
use server::rest::DOMAIN_SOCKET;
use server::{RequestMessage, ResponseMessage};
use service::service::Service;
use std::os::unix::net::UnixStream;
use std::path::PathBuf;
use Result;

pub struct RestManagerClient {
    socket_path: PathBuf,
}

impl RestManagerClient {
    pub fn new() -> RestManagerClient {
        RestManagerClient::at_path(PathBuf::from(DOMAIN_SOCKET))
    }

    pub fn at_path(socket_path: PathBuf) -> RestManagerClient {
        RestManagerClient { socket_path }
    }

    fn connect(&self) -> Result<UnixStream> {
        connect(&self.socket_path)
    }
}

impl ManagerClient for RestManagerClient {
    fn is_running(&self) -> bool {
        self.socket_path.exists()
    }

    fn get_workers(&self) -> Result<Vec<Service>> {
        let mut stream = self.connect()?;
        stream.send(RequestMessage::GetWorkers)?;
        stream.recv().map(|response: ResponseMessage| {
            let ResponseMessage::Workers(workers) = response;
            workers
        })
    }

    fn stop_server(&self) -> Result<()> {
        let mut stream = self.connect()?;
        stream.send(RequestMessage::Quit)
    }
}

#[cfg(test)]
mod test;
