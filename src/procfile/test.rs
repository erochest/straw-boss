mod procfile_job {
    use std::str::FromStr;
    use super::super::ProcfileJob;

    #[test]
    fn test_from_str_identifies_service_names() {
        let input = vec!["web: start web-server", "worker: start worker"];
        assert_eq!(
            vec![String::from("web"), String::from("worker")],
            input
                .into_iter()
                .map(ProcfileJob::from_str)
                .map(|p| p.unwrap().0)
                .collect::<Vec<String>>()
        );
    }

    #[test]
    fn test_from_str_associates_names_with_commands() {
        let input = vec!["web: start web-server", "worker: start worker"];
        let expected = vec![
            ProcfileJob(String::from("web"), String::from("start web-server")),
            ProcfileJob(String::from("worker"), String::from("start worker")),
        ];
        assert_eq!(
            expected,
            input
                .into_iter()
                .map(|s| ProcfileJob::from_str(s).unwrap())
                .collect::<Vec<ProcfileJob>>()
        )
    }

    #[test]
    fn test_returns_an_error_on_empty() {
        assert!("".parse::<ProcfileJob>().is_err());
    }

    #[test]
    fn test_returns_an_error_if_no_colon() {
        assert!("something no colon".parse::<ProcfileJob>().is_err());
    }
}
