use rmp_serde::{Deserializer, Serializer};
use serde::{Deserialize, Serialize};
use server::rest::DOMAIN_SOCKET;
use std::fmt::Debug;
use std::os::unix::net::UnixStream;
use std::path::Path;
use Result;

pub trait Sender {
    fn send<M: Serialize + Debug>(&mut self, msg: M) -> Result<()>;
}

pub trait Receiver {
    fn recv<'de, M: Deserialize<'de> + Debug>(&mut self) -> Result<M>;
}

impl Sender for UnixStream {
    fn send<M: Serialize + Debug>(&mut self, msg: M) -> Result<()> {
        let mut ser = Serializer::new(self);
        msg.serialize(&mut ser)
            .map_err(|err| format_err!("Unable to send {:?} to server: {:?}", &msg, &err))
    }
}

impl Receiver for UnixStream {
    fn recv<'de, M: Deserialize<'de> + Debug>(&mut self) -> Result<M> {
        let mut deser = Deserializer::new(self);
        Deserialize::deserialize(&mut deser)
            .map_err(|err| format_err!("Unable to receive: {:?}", &err))
    }
}

pub fn connect<P: AsRef<Path>>(socket: P) -> Result<UnixStream> {
    UnixStream::connect(socket).map_err(|err| {
        format_err!(
            "Unable to connect to server on {:?}: {:?}",
            &DOMAIN_SOCKET,
            &err
        )
    })
}

#[cfg(test)]
mod test {
    use std::fs;
    use std::os::unix::net::UnixListener;
    use std::path::PathBuf;

    fn setup(name: &str) -> UnixListener {
        let path = PathBuf::from(name);
        if path.exists() {
            fs::remove_file(&path).unwrap();
        }
        UnixListener::bind(&path).unwrap()
    }

    mod send {
        use super::super::send;
        use super::setup;
        use spectral::prelude::*;
        use std::io::{Read, Write};
        use std::os::unix::net::UnixStream;
        use std::sync::{Arc, RwLock};
        use std::thread;
        use std::time::Duration;

        #[test]
        fn test_connects_to_socket() {
            let socket_path = "/tmp/straw-boss.connects-to-socket.sock";

            let handle = thread::spawn(move || {
                let _server = setup(socket_path);
                thread::sleep(Duration::from_secs(2));
            });

            thread::sleep(Duration::from_secs(1));
            assert_that(&UnixStream::connect(socket_path)).is_ok();

            handle.join().unwrap();
        }

        #[test]
        fn test_sends_value() {
            let socket_path = "/tmp/straw-boss.sends-value.sock";
            let value: Arc<RwLock<Option<Vec<u8>>>> = Arc::new(RwLock::new(None));

            let handle = thread::spawn(move || {
                let server = setup(socket_path);
                let (mut stream, _) = server.accept().unwrap();
                stream.write(b"hello, world").unwrap();
                thread::sleep(Duration::from_secs(2));
            });

            thread::sleep(Duration::from_secs(1));
            assert_that(&UnixStream::connect(socket_path)).is_ok();

            handle.join().unwrap();
        }
    }

    mod recv {}

    mod connect {}
}
