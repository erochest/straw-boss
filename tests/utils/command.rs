use assert_cmd::prelude::*;
use std::fmt::Debug;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::thread::{spawn, JoinHandle};
use straw_boss::Result;

#[derive(Debug)]
pub struct StopServer {
    tag: String,
    server_task: ServerTask,
    socket_path: PathBuf,
}

#[derive(Debug)]
#[allow(dead_code)]
enum ServerTask {
    Unstarted,
    Join(Option<JoinHandle<Result<Output>>>),
    Output(Output),
}

impl StopServer {
    pub fn new(tag: &str) -> StopServer {
        let tag = String::from(tag);
        let socket_path = PathBuf::from(format!("/tmp/straw-boss.{}.sock", &tag));

        StopServer {
            tag,
            server_task: ServerTask::Unstarted,
            socket_path,
        }
    }

    pub fn pid_file(&self) -> String {
        format!("/tmp/straw-boss.{}.pid", &self.tag)
    }

    #[allow(dead_code)]
    pub fn start<P: AsRef<Path>>(&mut self, procfile: P) -> Result<()> {
        let pid_file = self.pid_file();
        let mut command = build_start_command(&pid_file, &self.socket_path, procfile)?;

        let join = spawn(move || {
            command
                .output()
                .map_err(|err| format_err!("Unable to execute start command: {:?}", &err))
        });

        self.server_task = ServerTask::Join(Some(join));
        Ok(())
    }

    #[allow(dead_code)]
    pub fn daemonize<P: AsRef<Path>>(&mut self, procfile: P) -> Result<()> {
        let pid_file = self.pid_file();
        let mut command = build_start_command(&pid_file, &self.socket_path, procfile)?;

        command.arg("--daemon");
        let output = command
            .output()
            .map_err(|err| format_err!("Unable to spawn daemon: {:?}", &err))?;

        self.server_task = ServerTask::Output(output);
        Ok(())
    }

    pub fn stop(&mut self) -> Result<Output> {
        match self.server_task {
            ServerTask::Unstarted => Err(format_err!("Task {} hasn't started yet.", &self.tag)),
            ServerTask::Join(ref mut join) => {
                send_stop(&self.socket_path)?;
                join.take()
                    .ok_or_else(|| format_err!("Unable to get join handle"))
                    .and_then(|j| {
                        j.join()
                            .map_err(|e| format_err!("Unable to join: {:?}", &e))
                            .and_then(|r| r)
                    })
            }
            ServerTask::Output(ref output) => {
                send_stop(&self.socket_path)?;
                Ok(output.clone())
            }
        }
    }

    #[allow(dead_code)]
    pub fn build_client(&self) -> Result<Command> {
        build_client(&self.socket_path.to_string_lossy())
    }
}

fn send_stop<P: AsRef<Path> + Debug>(socket_path: P) -> Result<()> {
    let socket_path = socket_path.as_ref();
    if socket_path.exists() {
        let mut client = build_client(&socket_path.to_string_lossy())?;

        client
            .arg("stop")
            .ok()
            .map_err(|err| format_err!("Unable to stop server: {:?}", &err))?;
    }

    Ok(())
}

fn build_client(socket_path: &str) -> Result<Command> {
    let mut command = Command::main_binary()
        .map_err(|err| format_err!("Unable to find main binary: {:?}", &err))?;
    command.env("STRAWBOSS_SOCKET_PATH", socket_path);
    Ok(command)
}

fn build_start_command<P: AsRef<Path>, R: AsRef<Path>, Q: AsRef<Path>>(
    pid_file: P,
    socket_path: Q,
    procfile: R,
) -> Result<Command> {
    let pid_file = pid_file.as_ref();
    let socket_path = socket_path.as_ref();

    if socket_path.exists() {
        fs::remove_file(&socket_path).map_err(|err| {
            format_err!(
                "Unable to remove socket path {:?}: {:?}",
                &socket_path,
                &err
            )
        })?;
    }
    if pid_file.exists() {
        fs::remove_file(&pid_file)
            .map_err(|err| format_err!("Unable to remove PID file {:?}: {:?}", &pid_file, &err))?;
    }

    let mut command = Command::main_binary()
        .map_err(|err| format_err!("Unable to find main binary: {:?}", &err))?;
    command
        .env("STRAWBOSS_PID_FILE", &*pid_file.to_string_lossy())
        .env("STRAWBOSS_SOCKET_PATH", &*socket_path.to_string_lossy())
        .arg("start")
        .arg("--procfile")
        .arg(&procfile.as_ref());

    Ok(command)
}

impl Drop for StopServer {
    fn drop(&mut self) {
        let _ = self.stop();
    }
}
