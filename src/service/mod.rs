use duct::{cmd, Expression};
use failure::Error;
use service::messages::{TaskMessage, TaskResponse};
use shellwords;
use std::collections::HashMap;
use std::convert::TryFrom;
use std::io;
use std::io::BufRead;
use std::iter::FromIterator;
use std::str::FromStr;
use std::sync::mpsc::{Receiver, Sender};
use Result;

pub mod messages;
pub mod worker;

type CommandArgs = Vec<String>;
type CommandList = Vec<CommandArgs>;

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
    /// let service = Service::new("hello", "while :; do sleep 1; done");
    ///
    /// assert_eq!(&String::from("hello"), &service.name);
    /// ```
    pub fn new(name: &str, command: &str) -> Service {
        Service {
            name: String::from(name),
            command: String::from(command),
        }
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
    /// use straw_boss::service::Service;
    ///
    /// let input = b"web: start web-server\n\
    ///               worker: start worker\n\
    ///               queue: queue-mgr\n";
    /// let services = Service::read_procfile(&input[..]).unwrap();
    /// assert_eq!(services, vec![
    ///     Service::new("web", "start web-server"),
    ///     Service::new("worker", "start worker"),
    ///     Service::new("queue", "queue-mgr"),
    /// ]);
    /// ```
    pub fn read_procfile<R: io::Read>(input: R) -> Result<Vec<Service>> {
        io::BufReader::new(input)
            .lines()
            .filter_map(|result| result.ok())
            .filter(|line| !line.trim_left().starts_with('#'))
            .map(|s| s.parse())
            .collect::<Result<Vec<Service>>>()
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
    /// // assert_eq!("start web-server", &service.command);
    /// ```
    fn from_str(s: &str) -> Result<Service> {
        let mut parts = s.splitn(2, ':');
        let name = parts
            .next()
            .ok_or_else(|| format_err!("Invalid Procfile line: {:?}", &s))?;
        let command = parts
            .next()
            .ok_or_else(|| format_err!("Invalid Procfile line: {:?}", &s))?
            .trim();
        Ok(Service::new(&name, &command))
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
///     Service::new("web", "start web-server"),
///     Service::new("worker", "start worker"),
///     Service::new("queue", "queue-mgr"),
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

fn split_piped_commands(commands: &str) -> Result<CommandList> {
    let split = shellwords::split(commands)
        .map_err(|err| format_err!("Unable to parse command: {}: {:?}", &commands, &err))?
        .split(|word| word == "|")
        .map(|command| command.to_vec())
        .collect::<CommandList>();
    if split.iter().any(|c| c.is_empty()) {
        Err(format_err!("Empty command in: {}", &commands))
    } else {
        Ok(split)
    }
}

impl TryFrom<Service> for Expression {
    type Error = Error;

    /// Converts from a service into a `duct::Expression` that can be executed.
    ///
    /// ```rust
    /// #![feature(try_from)]
    /// extern crate straw_boss;
    /// extern crate duct;
    ///
    /// use duct::Expression;
    /// use std::convert::TryFrom;
    /// use straw_boss::service::Service;
    ///
    /// let service = Service::new("hello-world", "bash -c \"echo hello, world\"");
    /// let mut command = Expression::try_from(service).unwrap();
    /// let output = command.read().unwrap();
    /// assert_eq!("hello, world", output.trim());
    /// ```
    fn try_from(service: Service) -> Result<Expression> {
        let commands = split_piped_commands(&service.command)?;
        let mut commands = commands.into_iter();
        let initial = commands
            .next()
            .map(|command| cmd(&command[0], &command[1..]))
            .ok_or_else(|| format_err!("Invalid pipeline. No command."))?;
        let pipeline = commands.fold(initial, |p, c| p.pipe(cmd(&c[0], &c[1..])));

        Ok(pipeline)
    }
}

/// This takes the channels to communicate over and the service to run, and it executes the
/// service. This is meant to be run in a new thread.
pub fn run(service: Service, rx: Receiver<TaskMessage>, tx: Sender<TaskResponse>) -> Result<()> {
    let service_name = service.name.clone();
    let handle = Expression::try_from(service)?.start()?;

    let message = rx.recv().map_err(|err| {
        format_err!(
            "Unable to receive message for service {:?}: {:?}",
            &service_name,
            &err
        )
    })?;
    match message {
        TaskMessage::Join => {
            let output = handle.output().map_err(|err| {
                format_err!("Error waiting for service {}: {:?}", &service_name, &err)
            })?;
            tx.send(TaskResponse::Joined(output)).map_err(|err| {
                format_err!(
                    "Error while sending wait for service {}: {:?}",
                    &service_name,
                    &err
                )
            })
        }
        TaskMessage::Kill => {
            handle
                .kill()
                .map_err(|err| format_err!("Error killing service {}: {:?}", &service_name, &err))
        }
    }
}

#[cfg(test)]
mod test;
