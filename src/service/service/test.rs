mod from_str {
    use service::service::Service;
    use spectral::assert_that;
    use spectral::prelude::*;
    use std::str::FromStr;

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
            String::from("start web-server"),
            String::from("start worker"),
        ];
        let services = input
            .into_iter()
            .map(|s| Service::from_str(s).unwrap().command)
            .collect::<Vec<String>>();
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
    use service::service::index_services;
    use service::service::Service;
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
            .is_some()
            .map(|s| &s.command)
            .is_equal_to(&String::from("start web-server"));
        assert_that(&index.get("worker"))
            .is_some()
            .map(|s| &s.command)
            .is_equal_to(&String::from("start worker"));
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
            .is_some()
            .map(|s| &s.command)
            .is_equal_to(&String::from("second web-server"));
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
    use service::service::Service;
    use shellwords::split;
    use spectral::assert_that;
    use spectral::prelude::*;
    use std::convert::TryFrom;
    use std::process::Command;

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
