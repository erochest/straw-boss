use procfile::Procfile;
use serde_yaml;
use service::service;
use std::io::Write;
use Result;

/// Read the processes in the `Procfile` and write them back out as YAML.
pub fn yamlize<W: Write>(procfile: &Procfile, writer: &mut W) -> Result<()> {
    let services = procfile.read_services()?;
    let index = service::index_services(&services);
    let yaml = serde_yaml::to_string(&index)
        .map_err(|err| format_err!("Cannot convert index to YAML: {}", &err))?;

    writer
        .write_fmt(format_args!("{}", yaml))
        .map_err(|err| format_err!("Cannot write YAML: {:?}", &err))
}
