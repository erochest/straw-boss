use client::{ManagerClient, ManagerStatus};
use std::collections::HashMap;
use Result;

/// Query a daemonized server to get the status of all of the tasks it's running.
pub fn status<C: ManagerClient>(client: &C) -> Result<ManagerStatus> {
    if client.is_running() {
        client
            .get_workers()
            .map(|ws| {
                ws.into_iter()
                    .map(|w| (w.name, w.command))
                    .collect::<HashMap<String, String>>()
            }).map(ManagerStatus::RunningTasks)
            .map_err(|err| format_err!("Unable to query workers: {:?}", &err))
    } else {
        Ok(ManagerStatus::NotFound)
    }
}

#[cfg(test)]
mod test {
    use super::status;
    use client::ManagerClient;
    use service::service::Service;
    use spectral::prelude::*;
    use std::collections::HashMap;
    use tasks::TaskSpec;
    use Result;

    struct FakeManagerClient {
        running: bool,
        workers: Result<Vec<Service>>,
    }

    impl ManagerClient for FakeManagerClient {
        fn is_running(&self) -> bool {
            self.running
        }

        fn get_workers(&self) -> Result<Vec<Service>> {
            match self.workers {
                Ok(ref w) => Ok(w.clone()),
                Err(ref e) => Err(format_err!("{:?}", &e)),
            }
        }

        fn stop(&self, _task: TaskSpec) -> Result<()> {
            unimplemented!()
        }
    }

    #[test]
    fn test_no_response() {
        let client = FakeManagerClient {
            running: false,
            workers: Err(format_err!("not running")),
        };

        let actual = status(&client);
        assert_that(&actual).is_ok();
    }

    #[test]
    fn test_gets_worker_list() {
        let worker = Service::new("web", "run all the web");
        let client = FakeManagerClient {
            running: true,
            workers: Ok(vec![worker]),
        };

        let mut task_map: HashMap<String, String> = HashMap::new();
        task_map.insert("web".into(), "run all the web".into());
        let actual = status(&client);

        assert_that(&actual).is_ok();
    }
}
