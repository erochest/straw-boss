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
        let expected = vec!["start web-server".into(), "start worker".into()];
        let services = input
            .into_iter()
            .map(|s| Service::from_str(s).unwrap().command)
            .collect::<Vec<String>>();
        assert_that(&services).equals_iterator(&expected.iter());
    }

    #[test]
    fn test_returns_multiple_commands_for_service_when_piped() {
        let input = vec![
            "web: start web-server | tee web-server.log",
            "worker: start worker | tee worker.log",
        ];
        let expected = vec![
            "start web-server | tee web-server.log".to_string(),
            "start worker | tee worker.log".to_string(),
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
            Service::new("web", "start web-server"),
            Service::new("worker", "start worker"),
        ];
        let index = index_services(&input);
        let keys = index.keys().collect::<Vec<&String>>();
        assert_that(&keys).contains(&String::from("web"));
        assert_that(&keys).contains(&String::from("worker"));
    }

    #[test]
    fn test_index_associates_commands_with_names() {
        let input = vec![
            Service::new("web", "start web-server"),
            Service::new("worker", "start worker"),
        ];
        let index = index_services(&input);
        assert_that(&index.get("web"))
            .is_some()
            .map(|s| &s.command)
            .is_equal_to(&"start web-server".into());
        assert_that(&index.get("worker"))
            .is_some()
            .map(|s| &s.command)
            .is_equal_to(&"start worker".into());
    }

    #[test]
    fn test_index_keeps_last_duplicate() {
        let input = vec![
            Service::new("web", "start web-server"),
            Service::new("worker", "start worker"),
            Service::new("web", "second web-server"),
        ];
        let index = index_services(&input);
        assert_that(&index.get("web"))
            .is_some()
            .map(|s| &s.command)
            .is_equal_to(&"second web-server".to_string());
    }
}

mod read_procfile {
    use service::service::Service;
    use spectral::assert_that;
    use spectral::prelude::*;

    #[test]
    fn test_skips_comments() {
        let input = b"# commentary\nweb: start web-server\n";
        let services = Service::read_procfile(&input[..]);
        assert_that(&services).is_ok().has_length(1);
    }

    #[test]
    fn test_reads_one_service_per_line() {
        let input = b"web: start web-server\nworker: start worker\n";
        let services = Service::read_procfile(&input[..]);
        assert_that(&services).is_ok().has_length(2);
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
        assert_that(&names).contains(&"start web-server".into());
        assert_that(&names).contains(&"start worker".into());
    }
}

mod split_piped_commands {
    use super::super::split_piped_commands;
    use spectral::prelude::*;

    #[test]
    fn test_returns_err_on_invalid_shell() {
        assert_that(&split_piped_commands("echo \"hi")).is_err();
    }

}
