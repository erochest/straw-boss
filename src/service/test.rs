mod service {
    mod from_str {
        use std::str::FromStr;
        use super::super::super::Service;

        #[test]
        fn test_identifies_service_names() {
            let input = vec!["web: start web-server", "worker: start worker"];
            assert_eq!(
                vec![String::from("web"), String::from("worker")],
                input
                    .into_iter()
                    .map(Service::from_str)
                    .map(|p| p.unwrap().name)
                    .collect::<Vec<String>>()
            );
        }

        #[test]
        fn test_associates_names_with_commands() {
            let input = vec!["web: start web-server", "worker: start worker"];
            let expected = vec![
                Service::from_command("web", "start web-server"),
                Service::from_command("worker", "start worker"),
            ];
            assert_eq!(
                expected,
                input
                    .into_iter()
                    .map(|s| Service::from_str(s).unwrap())
                    .collect::<Vec<Service>>()
            )
        }

        #[test]
        fn test_returns_an_error_on_empty() {
            assert!("".parse::<Service>().is_err());
        }

        #[test]
        fn test_returns_an_error_if_no_colon() {
            assert!("something no colon".parse::<Service>().is_err());
        }
    }

    mod index_services {
        use super::super::super::{index_services, Service};

        #[test]
        fn test_index_returns_expected_count() {
            let input = vec![
                Service::from_command("web", "start web-server"),
                Service::from_command("worker", "start worker"),
            ];
            let index = index_services(&input);
            let mut keys = index.keys().collect::<Vec<&String>>();
            keys.sort();
            assert_eq!(vec![&String::from("web"), &String::from("worker")], keys);
        }

        #[test]
        fn test_index_associates_commands_with_names() {
            let input = vec![
                Service::from_command("web", "start web-server"),
                Service::from_command("worker", "start worker"),
            ];
            let index = index_services(&input);
            assert_eq!(
                Some(&String::from("start web-server")),
                index.get("web").map(|s| &s.command)
            );
            assert_eq!(
                Some(&String::from("start worker")),
                index.get("worker").map(|s| &s.command)
            );
        }

        #[test]
        fn test_index_keeps_last_duplicate() {
            let input = vec![
                Service::from_command("web", "start web-server"),
                Service::from_command("worker", "start worker"),
                Service::from_command("web", "second web-server"),
            ];
            let index = index_services(&input);
            assert_eq!(
                Some(&String::from("second web-server")),
                index.get("web").map(|s| &s.command)
            );
        }
    }

    mod read_procfile {
        use super::super::super::read_procfile;

        #[test]
        fn test_reads_one_service_per_line() {
            let input = b"web: start web-server\nworker: start worker\n";
            let services = read_procfile(&input[..]);
            assert!(services.is_ok());
            assert_eq!(2, services.unwrap().len());
        }

        #[test]
        fn test_errors_on_invalid_input() {
            let input = b"web start web-server\nworker: start worker\n";
            let services = read_procfile(&input[..]);
            assert!(services.is_err());
        }

        #[test]
        fn test_reads_names() {
            let input = b"web: start web-server\nworker: start worker\n";
            let services = read_procfile(&input[..]).expect("To read the services.");
            let names = services
                .into_iter()
                .map(|s| s.name)
                .collect::<Vec<String>>();
            assert_eq!(vec![String::from("web"), String::from("worker")], names);
        }

        #[test]
        fn test_reads_commands() {
            let input = b"web: start web-server\nworker: start worker\n";
            let services = read_procfile(&input[..]).expect("To read the services.");
            let names = services
                .into_iter()
                .map(|s| s.command)
                .collect::<Vec<String>>();
            assert_eq!(
                vec![
                    String::from("start web-server"),
                    String::from("start worker"),
                ],
                names
            );
        }
    }

    mod try_into_command {
        use std::convert::TryFrom;
        use std::process::Command;
        use service::Service;
        use shellwords::split;

        #[test]
        fn test_returns_err_if_cannot_find_program() {
            let service = Service::from_command("will-error", "./does-not-exist");
            let mut command = Command::try_from(service).unwrap();
            let result = command.spawn();
            assert!(result.is_err());
        }

        #[test]
        fn test_runs_single_commands() {
            let service = Service::from_command("ls", "ls");
            let mut command = Command::try_from(service).unwrap();
            let status = command.status().unwrap();
            assert!(status.success());
        }

        #[test]
        fn test_runs_commands_with_arguments() {
            let service = Service::from_command("lslh", "ls -lh");
            let mut command = Command::try_from(service).unwrap();
            let output = command.output().unwrap();
            assert!(output.status.success());

            let stdout = String::from_utf8_lossy(&output.stdout);
            assert!(stdout.contains("drwxr-xr-x"));
        }

        #[test]
        fn test_runs_commands_with_arguments_quoted_with_backslashes() {
            let service = Service::from_command("ls-hello", "ls fixtures/hello\\ there");
            let mut command = Command::try_from(service).unwrap();
            let status = command.status().unwrap();
            assert!(status.success());
        }

        #[test]
        fn test_runs_commands_with_arguments_quoted_with_double_quotes() {
            let service = Service::from_command("ls-hello", "ls \"fixtures/hello there\"");
            let mut command = Command::try_from(service).unwrap();
            let status = command.status().unwrap();
            assert!(status.success());
        }

        #[test]
        fn test_splits_commands() {
            let words = split("ls hello-there").unwrap();
            assert_eq!(2, words.len());
            assert_eq!(vec![String::from("ls"), String::from("hello-there")], words);
        }

    }

    mod start {
        use std::thread;
        use std::time;
        use service::Service;
        use reqwest;

        #[test]
        fn test_runs_in_separate_thread() {
            let service = Service::from_command("sleep", "sleep 3");
            let task = service.start().unwrap();
            let current = thread::current();
            assert_ne!(current.id(), task.thread_id().unwrap());
        }

        #[test]
        fn test_runs_successfully() {
            let service = Service::from_command("sleep", "sleep 3");
            let mut task = service.start().unwrap();
            let status = task.join().unwrap();
            assert!(status.success(), "Status code: {:?}", &status.code());
        }

        #[test]
        fn test_child_runs_in_background() {
            let service = Service::from_command("web", "python3 -m http.server 3052");
            let _task = service.start().unwrap();
            thread::sleep(time::Duration::from_secs(1));
            let response = reqwest::get("http://localhost:3052");
            assert!(response.is_ok());
            assert!(response.unwrap().status().is_success());
        }
    }

    mod join {
        use service::Service;
        use std::time::{Duration, Instant};

        #[test]
        fn test_waits_for_child_to_finish() {
            let service = Service::from_command("sleep", "sleep 3");
            let start_time = Instant::now();
            let mut task = service.start().unwrap();
            let _child = task.join().unwrap();
            let end_time = Instant::now();
            assert!((end_time - start_time) > Duration::from_secs(2));
        }
    }

    mod kill {
        use service::Service;
        use std::thread;
        use std::time;
        use reqwest;

        #[test]
        fn test_stops_child_task() {
            let service = Service::from_command("web", "python3 -m http.server 3051");
            let task = service.start().unwrap();
            thread::sleep(time::Duration::from_secs(1));
            task.kill().unwrap();
            let response = reqwest::get("http://localhost:3051");
            assert!(response.is_err());
        }
    }

    mod thread_id {}

    mod process_id {}

    mod drop {
        use std::thread;
        use std::time;
        use service::Service;
        use reqwest;

        #[test]
        fn test_drop_kills_child_task() {
            let service = Service::from_command("web", "python3 -m http.server 3050");
            {
                let _task = service.start().unwrap();
                thread::sleep(time::Duration::from_secs(1));
            }
            let response = reqwest::get("http://localhost:3050");
            assert!(response.is_err());
        }
    }
}
