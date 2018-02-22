use std::collections::HashMap;
use std::io;
use std::io::{BufRead, Read};
use std::iter::FromIterator;
use std::str::FromStr;
use failure::Error;

#[derive(Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Service {
    pub name: String,
    pub command: String,
}

impl Service {
    pub fn from_command(name: &str, command: &str) -> Service {
        Service {
            name: name.into(),
            command: command.into(),
        }
    }
}

impl FromStr for Service {
    type Err = Error;

    fn from_str(s: &str) -> Result<Service, Self::Err> {
        let mut parts = s.splitn(2, ':');
        let name = parts
            .next()
            .ok_or_else(|| format_err!("Invalid Procfile line: {:?}", &s))?;
        let command = parts
            .next()
            .ok_or_else(|| format_err!("Invalid Procfile line: {:?}", &s))?;
        Ok(Service::from_command(name, command.trim()))
    }
}

pub fn index_services(services: &[Service]) -> HashMap<String, &Service> {
    HashMap::from_iter(services.into_iter().map(|s| (s.name.clone(), s)))
}

pub fn read_procfile<R: io::Read>(input: R) -> Result<Vec<Service>, Error> {
    let reader = io::BufReader::new(input);
    let lines = reader
        .lines()
        .collect::<io::Result<Vec<String>>>()
        .map_err(|err| format_err!("Unable to read from Procfile input: {:?}", err))?;

    lines
        .into_iter()
        .map(|s| s.parse())
        .collect::<Result<Vec<Service>, Error>>()
}

#[cfg(test)]
mod test;
