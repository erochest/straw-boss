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
}
