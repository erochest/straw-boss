mod procfile {
    mod read_services {
        use actions::Procfile;
        use service::service::Service;
        use spectral::assert_that;
        use spectral::prelude::*;

        #[test]
        fn test_reads_list_of_services() {
            let procfile = Procfile::new("fixtures/Procfile".into());
            let services = procfile.read_services().unwrap();
            let expected = vec![
                Service::new("ticker", "ruby ./ticker $PORT"),
                Service::new("error", "ruby ./error"),
                Service::new("utf8", "ruby ./utf8"),
                Service::new("spawner", "./spawner"),
            ];
            let mut assert = assert_that(&services);
            assert.contains(&expected[0]);
            assert.contains(&expected[1]);
            assert.contains(&expected[2]);
            assert.contains(&expected[3]);
        }
    }
}

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
