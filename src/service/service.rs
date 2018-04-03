use failure::Error;
use Result;
use service::TaskMessage;
use service::TaskResponse;
use std::collections::HashMap;
use std::convert::TryFrom;
use std::fmt::Debug;
use std::io;
use std::io::BufRead;
use std::process::Command;
use std::str::FromStr;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::Sender;
use std::thread::current;
use shellwords;
use std::iter::FromIterator;

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

    pub fn run(self, tx: Sender<TaskResponse>, rx: Receiver<TaskMessage>) -> Result<()> {
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
                        .map_err(|err| format_err!("Unable to kill {}: {:?}", &name, &err));
                }
            };
        }

        Ok(())
    }

    pub fn wrap_err_detail<D: Debug>(&self, message: &str, detail: &D) -> Error {
        format_err!("{} {}: {:?}", &message, &self.name, &detail)
    }
}

impl TryFrom<Service> for Command {
    type Error = Error;
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
    /// use straw_boss::service::Service;
    ///
    /// let service: Service = "web: start web-server".parse().unwrap();
    /// assert_eq!("web", &service.name);
    /// assert_eq!("start web-server", &service.command);
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

#[cfg(test)]
mod test {
    mod from_str {
        use std::str::FromStr;
        use service::service::Service;
        use spectral::assert_that;
        use spectral::prelude::*;

        #[test]
        fn test_identifies_service_names() {
            let input = vec!["web: start web-server", "worker: start worker"];
            let service_names = input
                .into_iter()
                .map(Service::from_str)
                .map(|p| p.unwrap().name)
                .collect::<Vec<String>>();
            assert_that(&service_names).contains(String::from("web"));
            assert_that(&service_names).contains(String::from("worker"));
        }

        #[test]
        fn test_associates_names_with_commands() {
            let input = vec!["web: start web-server", "worker: start worker"];
            let expected = vec![
                Service::from_command("web", "start web-server"),
                Service::from_command("worker", "start worker"),
            ];
            let services = input
                .into_iter()
                .map(|s| Service::from_str(s).unwrap())
                .collect::<Vec<Service>>();
            assert_that(&services).equals_iterator(&expected.iter());
        }

        #[test]
        fn test_returns_an_error_on_empty() {
            assert_that(&"".parse::<Service>()).is_err();
        }

        #[test]
        fn test_returns_an_error_if_no_colon() {
            assert_that(&"something no colon".parse::<Service>()).is_err();
        }
    }

    mod index_services {
        use service::service::Service;
        use service::service::index_services;
        use spectral::assert_that;
        use spectral::prelude::*;

        #[test]
        fn test_index_returns_expected_count() {
            let input = vec![
                Service::from_command("web", "start web-server"),
                Service::from_command("worker", "start worker"),
            ];
            let index = index_services(&input);
            let keys = index.keys().collect::<Vec<&String>>();
            assert_that(&keys).contains(&String::from("web"));
            assert_that(&keys).contains(&String::from("worker"));
        }

        #[test]
        fn test_index_associates_commands_with_names() {
            let input = vec![
                Service::from_command("web", "start web-server"),
                Service::from_command("worker", "start worker"),
            ];
            let index = index_services(&input);
            assert_that(&index.get("web"))
                .mapped_contains(|s| s.command.clone(), &String::from("start web-server"));
            assert_that(&index.get("worker"))
                .mapped_contains(|s| s.command.clone(), &String::from("start worker"));
        }

        #[test]
        fn test_index_keeps_last_duplicate() {
            let input = vec![
                Service::from_command("web", "start web-server"),
                Service::from_command("worker", "start worker"),
                Service::from_command("web", "second web-server"),
            ];
            let index = index_services(&input);
            assert_that(&index.get("web"))
                .mapped_contains(|s| s.command.clone(), &String::from("second web-server"));
        }
    }

    mod read_procfile {
        use service::service::Service;
        use spectral::assert_that;
        use spectral::prelude::*;

        #[test]
        fn test_reads_one_service_per_line() {
            let input = b"web: start web-server\nworker: start worker\n";
            let services = Service::read_procfile(&input[..]);
            assert_that(&services).is_ok();
            assert_that(&services.unwrap()).has_length(2);
        }

        #[test]
        fn test_errors_on_invalid_input() {
            let input = b"web start web-server\nworker: start worker\n";
            let services = Service::read_procfile(&input[..]);
            assert_that(&services).is_err();
        }

        #[test]
        fn test_reads_names() {
            let input = b"web: start web-server\nworker: start worker\n";
            let services = Service::read_procfile(&input[..]).expect("To read the services.");
            let names = services
                .into_iter()
                .map(|s| s.name)
                .collect::<Vec<String>>();
            assert_that(&names).contains(String::from("web"));
            assert_that(&names).contains(String::from("worker"));
        }

        #[test]
        fn test_reads_commands() {
            let input = b"web: start web-server\nworker: start worker\n";
            let services = Service::read_procfile(&input[..]).expect("To read the services.");
            let names = services
                .into_iter()
                .map(|s| s.command)
                .collect::<Vec<String>>();
            assert_that(&names).contains(String::from("start web-server"));
            assert_that(&names).contains(String::from("start worker"));
        }
    }

    mod try_into_command {
        use std::convert::TryFrom;
        use std::process::Command;
        use shellwords::split;
        use service::service::Service;
        use spectral::assert_that;
        use spectral::prelude::*;

        #[test]
        fn test_returns_err_if_cannot_find_program() {
            let service = Service::from_command("will-error", "./does-not-exist");
            let mut command = Command::try_from(service).unwrap();
            let result = command.spawn();
            assert_that(&result).is_err();
        }

        #[test]
        fn test_runs_single_commands() {
            let service = Service::from_command("ls", "ls");
            let mut command = Command::try_from(service).unwrap();
            let status = command.status().unwrap();
            assert_that(&status.success()).is_true();
        }

        #[test]
        fn test_runs_commands_with_arguments() {
            let service = Service::from_command("lslh", "ls -lh");
            let mut command = Command::try_from(service).unwrap();
            let output = command.output().unwrap();
            assert_that(&output.status.success()).is_true();

            let stdout = String::from(String::from_utf8_lossy(&output.stdout));
            assert_that(&stdout).contains("drwxr-xr-x");
        }

        #[test]
        fn test_runs_commands_with_arguments_quoted_with_backslashes() {
            let service = Service::from_command("ls-hello", "ls fixtures/hello\\ there");
            let mut command = Command::try_from(service).unwrap();
            let status = command.status().unwrap();
            assert_that(&status.success()).is_true();
        }

        #[test]
        fn test_runs_commands_with_arguments_quoted_with_double_quotes() {
            let service = Service::from_command("ls-hello", "ls \"fixtures/hello there\"");
            let mut command = Command::try_from(service).unwrap();
            let status = command.status().unwrap();
            assert_that(&status.success()).is_true();
        }

        #[test]
        fn test_splits_commands() {
            let words = split("ls hello-there").unwrap();
            assert_that(&words).has_length(2);
            assert_that(&words)
                .contains_all_of(&vec![&String::from("ls"), &String::from("hello-there")]);
        }
    }

    mod start {
        use std::thread;
        use std::time;
        use reqwest;
        use service::service::Service;
        use service::manager::ServiceManager;
        use spectral::assert_that;
        use spectral::prelude::*;

        #[test]
        fn test_runs_in_separate_thread() {
            let service = Service::from_command("sleep", "sleep 3");
            let task = ServiceManager::start(service).unwrap();
            let current = thread::current();
            assert_that(&task.thread_id().unwrap()).is_not_equal_to(current.id());
        }

        #[test]
        fn test_runs_successfully() {
            let service = Service::from_command("sleep", "sleep 3");
            let mut task = ServiceManager::start(service).unwrap();
            let status = task.join().unwrap();
            assert_that(&status.success()).named(&format!("Status code: {:?}", &status.code())).is_true();
        }

        #[test]
        fn test_child_runs_in_background() {
            let service = Service::from_command("web", "python3 -m http.server 3052");
            let _task = ServiceManager::start(service).unwrap();
            thread::sleep(time::Duration::from_secs(1));
            let response = reqwest::get("http://localhost:3052");
            assert_that(&response).is_ok();
            assert_that(&response.unwrap().status().is_success()).is_true();
        }
    }

    mod join {
        use std::time::{Duration, Instant};
        use service::service::Service;
        use service::manager::ServiceManager;
        use spectral::assert_that;
        use spectral::prelude::*;

        #[test]
        fn test_waits_for_child_to_finish() {
            let service = Service::from_command("sleep", "sleep 3");
            let start_time = Instant::now();
            let mut task = ServiceManager::start(service).unwrap();
            let _child = task.join().unwrap();
            let end_time = Instant::now();
            assert_that(&(end_time - start_time)).is_greater_than(Duration::from_secs(2));
        }
    }

    mod kill {
        use std::thread;
        use std::time;
        use reqwest;
        use service::service::Service;
        use service::manager::ServiceManager;
        use spectral::assert_that;
        use spectral::prelude::*;

        #[test]
        fn test_stops_child_task() {
            let service = Service::from_command("web", "python3 -m http.server 3051");
            let task = ServiceManager::start(service).unwrap();
            thread::sleep(time::Duration::from_secs(1));
            task.kill().unwrap();
            let response = reqwest::get("http://localhost:3051");
            assert_that(&response).is_err();
        }
    }

    mod thread_id {}

    mod process_id {}

    mod drop {
        use std::thread;
        use std::time;
        use reqwest;
        use service::service::Service;
        use service::manager::ServiceManager;
        use spectral::assert_that;
        use spectral::prelude::*;

        #[test]
        fn test_drop_kills_child_task() {
            let service = Service::from_command("web", "python3 -m http.server 3050");
            {
                let _task = ServiceManager::start(service).unwrap();
                thread::sleep(time::Duration::from_secs(1));
            }
            let response = reqwest::get("http://localhost:3050");
            assert_that(&response).is_err();
        }
    }
}
