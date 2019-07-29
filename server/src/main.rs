use multiping::Server;

#[macro_use]
extern crate log;

fn main() {
    env_logger::init();

    debug!("Server main:");

    Server::new().run("127.0.0.1:3000").unwrap();
}
