mod status {
    use actions::status;
    use client::{ManagerClient, ManagerStatus};
    use server::Worker;
    use spectral::prelude::*;
    use std::clone::Clone;
    use std::collections::HashMap;
    use Result;

    struct FakeManagerClient {
        running: bool,
        workers: Result<Vec<Worker>>,
    }

    impl ManagerClient for FakeManagerClient {
        fn is_running(&self) -> bool {
            self.running
        }

        fn get_workers(&self) -> Result<Vec<Worker>> {
            match self.workers {
                Ok(ref w) => Ok(w.clone()),
                Err(ref e) => Err(format_err!("{:?}", &e)),
            }
        }

        fn stop_server(&self) -> Result<()> {
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
        assert_that(&actual).is_ok_containing(&ManagerStatus::NotFound);
    }

    #[test]
    fn test_gets_worker_list() {
        let worker = Worker {
            name: "web".into(),
            command: "run all the web".into(),
        };
        let client = FakeManagerClient {
            running: true,
            workers: Ok(vec![worker]),
        };

        let mut task_map = HashMap::new();
        task_map.insert("web".into(), "run all the web".into());
        let actual = status(&client);

        assert_that(&actual).is_ok_containing(&ManagerStatus::RunningTasks(task_map));
    }
}
