mod service {
    use std::collections::HashMap;
    use std::str::FromStr;
    use super::super::{index_services, Service};

    #[test]
    fn test_from_str_identifies_service_names() {
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
    fn test_from_str_associates_names_with_commands() {
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
