use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

use serde_yaml;

use Result;
use service::manager;
use service::service;
use service::service::Service;
//use service::manager::ServiceManager;

#[derive(Debug)]
pub struct Procfile(PathBuf);

impl Procfile {
    pub fn new(procfile: PathBuf) -> Procfile {
        Procfile(procfile)
    }

    pub fn read_services(&self) -> Result<Vec<service::Service>> {
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

#[derive(Debug)]
pub enum Action {
    Start(Procfile),
    Yamlize(Procfile),
}

impl Action {
    pub fn execute<W: Write>(&self, writer: &mut W) -> Result<()> {
        match *self {
            Action::Start(ref procfile) => start(procfile, writer),
            Action::Yamlize(ref procfile) => yamlize(procfile, writer),
        }
    }
}

pub fn start<W: Write>(procfile: &Procfile, _writer: &mut W) -> Result<()> {
    let services = procfile.read_services()?;
    let managers: Vec<Result<manager::ServiceManager>> = services
        .into_iter()
        // TODO
        //.map(|s| ServiceManager::start(s))
        .map(|_s| unimplemented!())
        .collect();
    let mut result = None;
    let _erred = managers.iter().any(|r| r.is_err());

    for mut m in managers {
        match m {
            Ok(ref mut _manager) => {
                //TODO
                //if erred {
                //manager.kill();
                //}
                //manager.wait();
                ()
            }
            Err(err) => {
                result.get_or_insert_with(|| Err(format_err!("Error starting task: {:?}", &err)));
            }
        };
    }

    result.unwrap_or(Ok(()))
}

pub fn yamlize<W: Write>(procfile: &Procfile, writer: &mut W) -> Result<()> {
    let services = procfile.read_services()?;
    let index = service::index_services(&services);
    let yaml = serde_yaml::to_string(&index)
        .map_err(|err| format_err!("Cannot convert index to YAML: {}", &err))?;

    writer
        .write_fmt(format_args!("{}", yaml))
        .map_err(|err| format_err!("Cannot write YAML: {:?}", &err))
}

#[cfg(test)]
mod test;
