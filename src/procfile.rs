use service::Service;
use std::fs::File;
use std::path::PathBuf;
use Result;

/// A `Procfile`. This is a newtype for a `PathBuf`.
#[derive(Debug)]
pub struct Procfile(PathBuf);

impl Procfile {
    /// Create a new `Procfile` from a `PathBuf`.
    pub fn new(procfile: PathBuf) -> Procfile {
        Procfile(procfile)
    }

    /// Read a vector of `Service` instances from a `Procfile`.
    pub fn read_services(&self) -> Result<Vec<Service>> {
        let &Procfile(ref procfile) = self;
        let f = File::open(&procfile)
            .map_err(|err| format_err!("Unable to open Procfile: {:?}\n{}", &procfile, &err))?;
        Service::read_procfile(f).map_err(|err| {
            format_err!(
                "Unable to read data from Procfile: {:?}\n{}",
                &procfile,
                &err
            )
        })
    }
}

#[cfg(test)]
mod test {
    mod read_services {
        use procfile::Procfile;
        use service::Service;
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
