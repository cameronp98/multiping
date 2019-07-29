#[macro_use]
extern crate log;

use clap::{App, Arg};

use multiping::Server;

fn main() {
    env_logger::init();

    debug!("Server main:");

    let matches = App::new("Multiping Server")
        .arg(
            Arg::with_name("address")
                .short("a")
                .long("address")
                .help("Sets the adress to bind the server to")
                .takes_value(true)
                .required(true)
        )
        .get_matches();

    let addr = matches.value_of("address").unwrap();

    match Server::new().run(addr) {
        Ok(()) => {
            info!("Server exited successfully.");
        },
        Err(e) => {
            error!("Error whilst running server: {}", e);
        }
    }
}
