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
