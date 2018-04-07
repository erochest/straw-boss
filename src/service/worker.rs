//use std::sync::mpsc::Receiver;
//use std::sync::mpsc::Sender;

//#[derive(Debug)]
//pub struct ServiceWorker {
//pub service: Service,
//rx: Receiver<Command>,
//pub tx: Sender<Reply>,
//}

#[cfg(test)]
mod test {
    //use std::sync::mpsc::channel;

    // #[test]
    // fn test_new_from_service() {
    //     let service =
    //     ServiceWorker::new()
    // }

    mod start {
        //use reqwest;
        //use service::manager::ServiceManager;
        //use service::service::Service;
        //use spectral::assert_that;
        //use spectral::prelude::*;
        //use std::thread;
        //use std::time;

        //#[test]
        //fn test_runs_in_separate_thread() {
        //let service = Service::from_command("sleep", "sleep 3");
        //let task = ServiceManager::start(service).unwrap();
        //let current = thread::current();
        //assert_that(&task.thread_id().unwrap()).is_not_equal_to(current.id());
        //}

        // #[test]
        // fn test_runs_successfully() {
        //     let service = Service::from_command("sleep", "sleep 3");
        //     let mut task = ServiceManager::start(service).unwrap();
        //     let status = task.join().unwrap();
        //     assert_that(&status.success()).named(&format!("Status code: {:?}", &status.code())).is_true();
        // }

        // #[test]
        // fn test_child_runs_in_background() {
        //     let service = Service::from_command("web", "python3 -m http.server 3052");
        //     let _task = ServiceManager::start(service).unwrap();
        //     thread::sleep(time::Duration::from_secs(1));
        //     let response = reqwest::get("http://localhost:3052");
        //     assert_that(&response).is_ok();
        //     assert_that(&response.unwrap().status().is_success()).is_true();
        // }
    }

    mod join {
        //use std::time::{Duration, Instant};
        //use service::service::Service;
        //use service::manager::ServiceManager;
        //use spectral::assert_that;
        //use spectral::prelude::*;

        // #[test]
        // fn test_waits_for_child_to_finish() {
        //     let service = Service::from_command("sleep", "sleep 3");
        //     let start_time = Instant::now();
        //     let mut task = ServiceManager::start(service).unwrap();
        //     let _child = task.join().unwrap();
        //     let end_time = Instant::now();
        //     assert_that(&(end_time - start_time)).is_greater_than(Duration::from_secs(2));
        // }
    }

    mod kill {
        //use std::thread;
        //use std::time;
        //use reqwest;
        //use service::service::Service;
        //use service::manager::ServiceManager;
        //use spectral::assert_that;
        //use spectral::prelude::*;

        // #[test]
        // fn test_stops_child_task() {
        //     let service = Service::from_command("web", "python3 -m http.server 3051");
        //     let task = ServiceManager::start(service).unwrap();
        //     thread::sleep(time::Duration::from_secs(1));
        //     task.kill().unwrap();
        //     let response = reqwest::get("http://localhost:3051");
        //     assert_that(&response).is_err();
        // }
    }

    mod thread_id {}

    mod process_id {}

    mod drop {
        //use std::thread;
        //use std::time;
        //use reqwest;
        //use service::service::Service;
        //use service::manager::ServiceManager;
        //use spectral::assert_that;
        //use spectral::prelude::*;

        // #[test]
        // fn test_drop_kills_child_task() {
        //     let service = Service::from_command("web", "python3 -m http.server 3050");
        //     {
        //         let _task = ServiceManager::start(service).unwrap();
        //         thread::sleep(time::Duration::from_secs(1));
        //     }
        //     let response = reqwest::get("http://localhost:3050");
        //     assert_that(&response).is_err();
        // }
    }
}
