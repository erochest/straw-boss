use std::io::Write;
use std::fs::File;
use std::path::PathBuf;

use failure::Error;
use serde_yaml;

use service;

#[derive(Debug)]
pub struct Procfile(PathBuf);

impl Procfile {
    pub fn new(procfile: PathBuf) -> Procfile {
        Procfile(procfile)
    }

    pub fn read_services(&self) -> Result<Vec<service::Service>, Error> {
        let &Procfile(ref procfile) = self;
        let f = File::open(&procfile)
            .map_err(|err| format_err!("Unable to open Procfile: {:?}\n{}", &procfile, &err))?;
        service::read_procfile(f).map_err(|err| {
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
    pub fn execute<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        match *self {
            Action::Start(ref procfile) => start(procfile, writer).map(|_| ()),
            Action::Yamlize(ref procfile) => yamlize(procfile, writer),
        }
    }
}

pub fn start<W: Write>(
    procfile: &Procfile,
    _writer: &mut W,
) -> Result<Vec<Result<service::ServiceManager, Error>>, Error> {
    let services = procfile.read_services()?;
    Ok(services.into_iter().map(|s| s.start()).collect())
}

pub fn yamlize<W: Write>(procfile: &Procfile, writer: &mut W) -> Result<(), Error> {
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
