use multiping::{Client, Message};

#[macro_use]
extern crate log;

fn main() {
    env_logger::init();

    debug!("Client main:");

    let client = Client::new("127.0.0.1:3000");
    // let resp = client.send(Message::Ping).unwrap();
    println!("{:?}", client.send(Message::Pong));
    println!("{:?}", client.send(Message::Ping));
    println!(
        "{:?}",
        client.send(Message::Text(String::from("Hello, World!")))
    );
    println!("{:?}", client.send(Message::InvalidRequest));
}
