use rmp_serde::{Deserializer, Serializer};
use serde::{Deserialize, Serialize};
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

pub fn connect<P: AsRef<Path> + Debug>(socket: P) -> Result<UnixStream> {
    UnixStream::connect(&socket)
        .map_err(|err| format_err!("Unable to connect to server on {:?}: {:?}", &socket, &err))
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
        use super::super::Sender;
        use super::setup;
        use spectral::prelude::*;
        use std::collections::HashMap;
        use std::io::{Read, Write};
        use std::net::Shutdown;
        use std::os::unix::net::UnixStream;
        use std::sync::{Arc, RwLock};
        use std::thread;
        use std::time::Duration;

        #[test]
        fn test_connects_to_socket() {
            let socket_path = "/tmp/straw-boss.send.connects-to-socket.sock";

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
            let threaded_value = value.clone();

            let handle = thread::spawn(move || {
                let server = setup(socket_path);
                let (mut stream, _) = server.accept().unwrap();
                let mut buffer = Vec::new();

                stream.read_to_end(&mut buffer).unwrap();
                {
                    let mut value_lock = threaded_value.write().unwrap();
                    (*value_lock).replace(buffer);
                }

                thread::sleep(Duration::from_secs(2));
            });

            thread::sleep(Duration::from_secs(1));
            let stream = UnixStream::connect(socket_path);
            assert_that(&stream).is_ok();
            if let Ok(mut stream) = stream {
                let mut input: HashMap<String, u8> = HashMap::new();
                input.insert(String::from("answer"), 42);
                assert_that(&stream.send(input)).is_ok();
                stream.flush().unwrap();
                stream.shutdown(Shutdown::Both).unwrap();

                handle.join().unwrap();

                let value = value.clone();
                let value = value.read().unwrap();
                assert_that(&*value)
                    .is_some()
                    .is_equal_to(&b"\x81\xa6\x61\x6e\x73\x77\x65\x72\x2a".to_vec());
            }
        }
    }

    mod recv {
        use super::super::Receiver;
        use super::setup;
        use spectral::prelude::*;
        use std::collections::HashMap;
        use std::io::Write;
        use std::net::Shutdown;
        use std::os::unix::net::UnixStream;
        use std::sync::{Arc, RwLock};
        use std::thread;
        use std::time::Duration;
        use Result;

        #[test]
        fn test_receives_value() {
            let socket_path = "/tmp/straw-boss.recv.receives-value.sock";
            let value: Arc<RwLock<Option<Result<HashMap<String, u8>>>>> =
                Arc::new(RwLock::new(None));
            let threaded_value = value.clone();

            let handle = thread::spawn(move || {
                let server = setup(socket_path);
                for socket in server.incoming() {
                    let mut socket = socket.unwrap();
                    let recvd = socket.recv();
                    {
                        let mut value_lock = threaded_value.write().unwrap();
                        (*value_lock).replace(recvd);
                    }
                    break;
                }
            });

            thread::sleep(Duration::from_secs(1));
            let mut stream = UnixStream::connect(socket_path).unwrap();
            stream
                .write(b"\x81\xa6\x61\x6e\x73\x77\x65\x72\x2a")
                .unwrap();
            stream.flush().unwrap();
            stream.shutdown(Shutdown::Both).unwrap();

            handle.join().unwrap();

            let mut answer: HashMap<String, u8> = HashMap::new();
            answer.insert(String::from("answer"), 42);

            let value = value.clone();
            let value = value.read().unwrap();
            assert_that(&*value).is_some().is_ok().is_equal_to(&answer);
        }
    }

    mod connect {
        use super::super::connect;
        use spectral::prelude::*;
        use std::fs;
        use std::os::unix::net::UnixListener;
        use std::path;
        use std::thread;
        use std::time::Duration;

        #[test]
        fn test_cannot_connect_no_socket() {
            let socket = path::PathBuf::from("/tmp/straw-boss.test-cannot-connect-no-socket.sock");
            if socket.exists() {
                fs::remove_file(&socket).unwrap();
            }

            assert_that(&connect(&socket)).is_err();
        }

        #[test]
        fn test_connects_open_socket() {
            let socket = path::PathBuf::from("/tmp/straw-boss.test-connects-open-socket.sock");
            let server_socket = socket.clone();
            if socket.exists() {
                fs::remove_file(&socket).unwrap();
            }

            let handle = thread::spawn(move || {
                let _server = UnixListener::bind(&server_socket).unwrap();
                thread::sleep(Duration::from_secs(2));
            });

            thread::sleep(Duration::from_secs(1));
            assert_that(&connect(&socket)).is_ok();

            handle.join().unwrap();
        }
    }
}
