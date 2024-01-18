use clap::ArgMatches;

#[derive(Debug, Clone)]
pub struct ServerArgs {
    pub port: u32,
}

impl ServerArgs {
    pub fn new_with(args: &ArgMatches) -> Result<Self, String> {
        Ok(ServerArgs {
            port: parse_port(args).unwrap(),
        })
    }
}

fn parse_port(args: &ArgMatches) -> Result<u32, String> {
    let port = args
        .value_of("port")
        .ok_or("Could not find port argument").unwrap();
    port.parse()
        .map_err(|_err| "port is not a number".to_string())
}
