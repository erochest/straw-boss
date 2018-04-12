use Result;
use failure::Error;
use shellwords;
use std::collections::HashMap;
use std::convert::TryFrom;
use std::io;
use std::io::BufRead;
use std::iter::FromIterator;
use std::process::Command;
use std::str::FromStr;

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
    /// use straw_boss::service::service::Service;
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
    /// use straw_boss::service::service::Service;
    ///
    /// let input = b"web: start web-server\n\
    ///               worker: start worker\n\
    ///               queue: queue-mgr\n";
    /// let services = Service::read_procfile(&input[..]).unwrap();
    /// assert_eq!(services, vec![
    ///     Service::from_command("web", "start web-server"),
    ///     Service::from_command("worker", "start worker"),
    ///     Service::from_command("queue", "queue-mgr"),
    /// ]);
    /// ```
    pub fn read_procfile<R: io::Read>(input: R) -> Result<Vec<Service>> {
        let reader = io::BufReader::new(input);
        let lines = reader
            .lines()
            .collect::<io::Result<Vec<String>>>()
            .map_err(|err| format_err!("Unable to read from Procfile input: {:?}", err))?;

        lines
            .into_iter()
            .map(|s| s.parse())
            .collect::<Result<Vec<Service>>>()
    }
}

impl TryFrom<Service> for Command {
    type Error = Error;

    /// Converts from a service into a `Command` that can be executed.
    ///
    /// ```rust
    /// use straw_boss::service::service::Service;
    /// use std::process::Command;
    /// use std::convert::TryFrom;
    ///
    /// let service = Service::from_command("hellow-world", "bash -c \"echo hello, world\"");
    /// let mut command = Command::try_from(service).unwrap();
    /// let output = command.output().unwrap();
    /// assert_eq!(output.stdout, b"hello, world\n");
    /// ```
    fn try_from(value: Service) -> Result<Command> {
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
    /// use straw_boss::service::service::Service;
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
/// use straw_boss::service::service::{index_services, Service};
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

#[cfg(test)]
mod test;
