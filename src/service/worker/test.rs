use service::service::Service;
use service::worker::{ServiceWorker, Worker};

fn setup(name: &str, command: &str) -> ServiceWorker {
    let line = format!("{}: {}", &name, &command);
    let service = line.parse::<Service>().unwrap();
    let mut worker = ServiceWorker::new(service);
    worker.start().unwrap();

    worker
}

mod start {
    use super::setup;
    use reqwest;
    use service::worker::Worker;
    use spectral::assert_that;
    use spectral::prelude::*;
    use std::thread;
    use std::time;

    #[test]
    fn test_runs_in_separate_thread() {
        let worker = setup("test_runs_in_separate_thread", "sleep 3");
        let thread_id = worker.thread_id();

        assert_that(&thread_id).is_some();
        if let Some(thread_id) = thread_id {
            let current = thread::current();
            assert_that(&thread_id).is_not_equal_to(current.id());
        }
    }

    #[test]
    fn test_runs_successfully() {
        let mut worker = setup("test_runs_successfully", "sleep 3");
        let result = worker.join();
        assert_that(&result).is_ok();
    }

    #[test]
    fn test_child_runs_in_background() {
        let _worker = setup(
            "test_child_runs_in_background",
            "python3 -m http.server 3052",
        );
        thread::sleep(time::Duration::from_secs(1));

        let response = reqwest::get("http://localhost:3052");
        assert_that(&response).is_ok();
        assert_that(&response.unwrap().status().is_success()).is_true();
    }
}

mod join {
    use super::setup;
    use service::worker::Worker;
    use spectral::assert_that;
    use spectral::prelude::*;
    use std::time::{Duration, Instant};

    #[test]
    fn test_waits_for_child_to_finish() {
        let mut worker = setup("test_waits_for_child_to_finish", "sleep 3");
        let start_time = Instant::now();
        let output = worker.join();
        let end_time = Instant::now();

        assert_that(&(end_time - start_time)).is_greater_than(Duration::from_secs(2));
        assert_that(&output).is_ok();
    }

    #[test]
    fn test_after_join_not_running() {
        let mut worker = setup("test_after_join_not_running", "sleep 3");

        assert_that(&worker.is_running()).is_true();

        let output = worker.join();
        assert_that(&output).is_ok();

        assert_that(&worker.is_running()).is_false();
    }
}

mod kill {
    use super::setup;
    use reqwest;
    use service::worker::Worker;
    use spectral::assert_that;
    use spectral::prelude::*;
    use std::thread;
    use std::time;

    #[test]
    fn test_stops_child_task() {
        let mut worker = setup("test_stops_child_task", "python3 -m http.server 3051");

        thread::sleep(time::Duration::from_secs(1));
        worker.kill().unwrap();

        let response = reqwest::get("http://localhost:3051");
        assert_that(&response).is_err();
    }
}

mod thread_id {
    use super::setup;
    use service::worker::Worker;
    use spectral::assert_that;
    use std::thread;

    #[test]
    fn test_returns_the_processes_thread_id() {
        let worker = setup("test_returns_the_processes_thread_id", "sleep 3");
        let thread_id = worker.thread_id();
        assert_that(&thread_id).is_not_equal_to(&Some(thread::current().id()));
    }
}

mod drop {
    use super::setup;
    use reqwest;
    use spectral::assert_that;
    use spectral::prelude::*;
    use std::thread;
    use std::time;

    #[test]
    fn test_drop_kills_child_task() {
        {
            let _worker = setup("test_drop_kills_child_task", "python3 -m http.server 3050");
            thread::sleep(time::Duration::from_secs(1));
        }
        thread::sleep(time::Duration::from_secs(1));
        let response = reqwest::get("http://localhost:3050");
        assert_that(&response).is_err();
    }
}
