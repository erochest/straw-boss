use std::str::FromStr;
use failure::Error;

#[derive(Debug, Eq, PartialEq)]
pub struct ProcfileJob(String, String);

impl FromStr for ProcfileJob {
    type Err = Error;

    fn from_str(s: &str) -> Result<ProcfileJob, Self::Err> {
        let mut parts = s.splitn(2, ':');
        let name = parts
            .next()
            .ok_or_else(|| format_err!("Invalid Procfile line: {:?}", &s))?;
        let command = parts
            .next()
            .ok_or_else(|| format_err!("Invalid Procfile line: {:?}", &s))?;
        Ok(ProcfileJob(
            String::from(name),
            String::from(command.trim()),
        ))
    }
}

#[cfg(test)]
mod test;
