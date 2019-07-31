use multiping::{Client, Message};

#[macro_use]
extern crate log;

fn main() {
    env_logger::init();

    let client = Client::new("127.0.0.1:3000");
    // let resp = client.send(Message::Ping).unwrap();
    debug!("ping");
    match client.send(Message::Ping) {
        Ok(msg) => println!("respose: {}", msg),
        Err(e) => error!("error: {}", e),
    }
}
