use multiping::{Client, Message};

#[macro_use]
extern crate log;

fn main() {
    env_logger::init();

    debug!("Client main:");

    let client = Client::new("127.0.0.1:3000");
    // let resp = client.send(Message::Ping).unwrap();
    match client.send(Message::Ping) {
        Ok(_) => println!("Message sent."),
        Err(e) => error!("Error: {}", e),
    }
}
