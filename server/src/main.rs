#[macro_use]
extern crate log;

use clap::{App, Arg};

use multiping::Server;

fn main() {
    env_logger::init();

    debug!("parse cli args");

    let matches = App::new("Multiping Server")
        .arg(
            Arg::with_name("address")
                .short("a")
                .long("address")
                .help("Sets the adress to bind the server to")
                .takes_value(true)
                .required(true),
        )
        .get_matches();

    debug!("cli args parsed successfully");

    let addr = matches.value_of("address").unwrap();

    debug!("run server");
    match Server::new().run(addr) {
        Ok(()) => {
            info!("server exited successfully.");
        }
        Err(e) => {
            error!("failed to run server: {}", e);
        }
    }
}
