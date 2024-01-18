use log::error;

pub trait ExitGracefully<T, E> {
    fn ok_or_exit(self, message: &str) -> T;
}

impl<T, E: std::fmt::Debug> ExitGracefully<T, E> for Result<T, E> {
    fn ok_or_exit(self, message: &str) -> T {
        match self {
            Ok(val) => val,
            Err(err) => {
                error!("{:?}: {}", err, message);
                std::process::exit(1);
            }
        }
    }
}

#[macro_export]
macro_rules! create_app {
    () => {
        App::new("Vsock Client")
            .about("Hello world example for vsock server and client communication.")
            .setting(AppSettings::ArgRequiredElseHelp)
            .version(env!("CARGO_PKG_VERSION"))
            .arg(
                Arg::with_name("port")
                    .long("port")
                    .help("port")
                    .takes_value(true)
                    .required(true),
            )
            .arg(
                Arg::with_name("cid")
                    .long("cid")
                    .help("cid")
                    .takes_value(true)
                    .required(true),
            )
    };
}
