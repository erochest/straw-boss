mod procfile {
    mod read_services {
        use actions::Procfile;
        use service::service::Service;

        #[test]
        fn test_reads_list_of_services() {
            let procfile = Procfile::new("fixtures/Procfile".into());
            let services = procfile.read_services().unwrap();
            assert_eq!(
                vec![
                    Service::from_command("ticker", "ruby ./ticker $PORT"),
                    Service::from_command("error", "ruby ./error"),
                    Service::from_command("utf8", "ruby ./utf8"),
                    Service::from_command("spawner", "./spawner"),
                ],
                services
            );
        }
    }
}
