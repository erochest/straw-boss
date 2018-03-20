use std::collections::HashMap;
use std::convert::TryFrom;
use std::fmt::Debug;
use std::io;
use std::io::BufRead;
use std::iter::FromIterator;
use std::process::{Command, ExitStatus};
use std::str::FromStr;
use std::sync::Arc;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread::{current, Builder, JoinHandle, ThreadId};
use failure::Error;
use shellwords;

/// A service that the straw boss will manage.
#[derive(Debug, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize, Clone)]
pub struct Service {
    /// The name/identifier for the service. This must be unique in the system.
    pub name: String,
    /// The command to execute to start this service.
    pub command: String,
}

impl Service {
    /// Create a
    ///
    /// # Arguments
    ///
    /// * `name`:
    /// * `command`:
    ///
    /// # Returns
    ///
    /// A `Service`.
    ///
    /// # Example
    ///
    /// ```rust
    /// use straw_boss::service::Service;
    ///
    /// let service = Service::from_command("hello", "while :; do sleep 1; done");
    ///
    /// assert_eq!(&String::from("hello"), &service.name);
    /// assert_eq!(&String::from("while :; do sleep 1; done"), &service.command);
    /// ```
    pub fn from_command(name: &str, command: &str) -> Service {
        Service {
            name: name.into(),
            command: command.into(),
        }
    }

    pub fn start(self) -> Result<ServiceManager, Error> {
        ServiceManager::start(self)
    }

    fn run(self, tx: Sender<TaskResponse>, rx: Receiver<TaskMessage>) -> Result<(), Error> {
        let name = self.name.clone();
        let mut command = Command::try_from(self)?;
        let mut child = command
            .spawn()
            .map_err(|err| format_err!("Error spawning service {}: {:?}", &name, &err))?;

        for message in rx.recv() {
            match message {
                TaskMessage::Join => {
                    let exit_status = child.wait().map_err(|err| {
                        format_err!(
                            "Unable to kill child {} to join the task: {:?}",
                            &name,
                            &err
                        )
                    });
                    tx.send(TaskResponse::Joined(exit_status)).map_err(|err| {
                        format_err!(
                            "Unable to send exit status for reporting {}: {:?}",
                            &name,
                            &err
                        )
                    })?;
                    return Ok(());
                }
                TaskMessage::ThreadId => {
                    let thread_id = current().id();
                    tx.send(TaskResponse::ThreadId(thread_id)).map_err(|err| {
                        format_err!("Unable to send {} thread-id response: {:?}", &name, &err)
                    })?;
                }
                TaskMessage::Kill => {
                    return child
                        .kill()
                        .map_err(|err| format_err!("Unable to kill {}: {:?}", &name, &err))
                }
            };
        }

        Ok(())
    }

    fn wrap_err_detail<D: Debug>(&self, message: &str, detail: &D) -> Error {
        format_err!("{} {}: {:?}", &message, &self.name, &detail)
    }
}

impl TryFrom<Service> for Command {
    type Error = Error;
    fn try_from(value: Service) -> Result<Command, Error> {
        let mut command_line = shellwords::split(&value.command)
            .map_err(|err| format_err!("Error parsing command {}: {:?}", &value.name, &err))?
            .into_iter();
        let program = command_line
            .next()
            .ok_or_else(|| format_err!("Invalid command line for service {}.", &value.name))?;
        let mut command = Command::new(program);
        command.args(command_line);
        Ok(command)
    }
}

impl FromStr for Service {
    type Err = Error;

    /// Parses a line from a `Procfile` into a `Service`.
    ///
    /// This requires the name of be separated from the command by a comma. The command is
    /// whitespace-trimmed.
    ///
    /// # Arguments
    ///
    /// * `s`: The string to parse.
    ///
    /// # Returns
    ///
    /// A `Service` wrapped in a `Result`. If there is no comma or nothing after the comma, it will
    /// return a `failure::Error`.
    ///
    /// # Example
    ///
    /// ```rust
    /// use straw_boss::service::Service;
    ///
    /// let service: Service = "web: start web-server".parse().unwrap();
    /// assert_eq!("web", &service.name);
    /// assert_eq!("start web-server", &service.command);
    /// ```
    fn from_str(s: &str) -> Result<Service, Self::Err> {
        let mut parts = s.splitn(2, ':');
        let name = parts
            .next()
            .ok_or_else(|| format_err!("Invalid Procfile line: {:?}", &s))?;
        let command = parts
            .next()
            .ok_or_else(|| format_err!("Invalid Procfile line: {:?}", &s))?;
        Ok(Service::from_command(name, command.trim()))
    }
}

/// Index a list of services by name. If more than one service uses the same name, only the second
/// will be kept.
///
/// # Arguments
///
/// * `services`: A slice of services to index.
///
/// # Returns
///
/// * A `HashMap` mapping service names to `Service` objects.
///
/// # Example
///
/// ```rust
/// use straw_boss::service::{index_services, Service};
/// use std::collections::HashMap;
///
/// let services = vec![
///     Service::from_command("web", "start web-server"),
///     Service::from_command("worker", "start worker"),
///     Service::from_command("queue", "queue-mgr"),
/// ];
/// let index = index_services(&services);
/// let mut items = index
///     .into_iter()
///     .map(|s| (s.0, &s.1.command))
///     .collect::<Vec<(String, &String)>>();
/// items.sort();
/// assert_eq!(vec![
///     (String::from("queue"), &String::from("queue-mgr")),
///     (String::from("web"), &String::from("start web-server")),
///     (String::from("worker"), &String::from("start worker"))
/// ], items);
/// ```
pub fn index_services(services: &[Service]) -> HashMap<String, &Service> {
    HashMap::from_iter(services.into_iter().map(|s| (s.name.clone(), s)))
}

/// Parses the data from a Procfile into a sequence of `Service` objects.
///
/// # Arguments
///
/// * `input`: Something that implements `std::io::Read`.
///
/// # Returns
///
/// This returns a vector of `Service` objects. If there's a problem reading from the input or
/// parsing the lines, it returns a `failure::Error`.
///
/// # Example
///
/// ```rust
/// use straw_boss::service::{read_procfile, Service};
///
/// let input = b"web: start web-server\n\
///               worker: start worker\n\
///               queue: queue-mgr\n";
/// let services = read_procfile(&input[..]).unwrap();
/// assert_eq!(services, vec![
///     Service::from_command("web", "start web-server"),
///     Service::from_command("worker", "start worker"),
///     Service::from_command("queue", "queue-mgr"),
/// ]);
/// ```
pub fn read_procfile<R: io::Read>(input: R) -> Result<Vec<Service>, Error> {
    let reader = io::BufReader::new(input);
    let lines = reader
        .lines()
        .collect::<io::Result<Vec<String>>>()
        .map_err(|err| format_err!("Unable to read from Procfile input: {:?}", err))?;

    lines
        .into_iter()
        .map(|s| s.parse())
        .collect::<Result<Vec<Service>, Error>>()
}

#[derive(Debug)]
pub struct ServiceManager {
    service: Service,
    join_handle: Option<JoinHandle<Result<(), Error>>>,
    tx: Sender<TaskMessage>,
    rx: Arc<Receiver<TaskResponse>>,
}

#[derive(Eq, Ord, PartialEq, PartialOrd, Debug)]
enum TaskMessage {
    Join,
    ThreadId,
    Kill,
}

#[derive(Debug)]
enum TaskResponse {
    Joined(Result<ExitStatus, Error>),
    ThreadId(ThreadId),
}

impl ServiceManager {
    pub fn start(service: Service) -> Result<ServiceManager, Error> {
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

    pub fn join(&mut self) -> Result<ExitStatus, Error> {
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

    pub fn thread_id(&self) -> Result<ThreadId, Error> {
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

    pub fn kill(self) -> Result<(), Error> {
        self.tx
            .send(TaskMessage::Kill)
            .map_err(|err| format_err!("Unable to send KILL message: {:?}", &err))
    }
}

impl Drop for ServiceManager {
    fn drop(&mut self) {
        if self.join_handle.is_some() {
            // Yes. We're ignoring this error. If it's already died, I don't need to kill it.
            let _result = self.tx.send(TaskMessage::Kill);
        }
    }
}

#[cfg(test)]
mod test;
