use std::net::TcpStream;

use crate::message::Message;
use crate::Result;

/// Client
pub struct Client<'a> {
    server_addr: &'a str,
}

impl<'a> Client<'a> {
    /// Create a new client for the server at `server_addr`
    pub fn new(server_addr: &'a str) -> Client {
        Client { server_addr }
    }

    /// Send a message to the server and return the response
    pub fn send(&self, msg: Message) -> Result<Message> {
        debug!("Message::send({})", msg);

        // connect to the server
        debug!("connect to server");
        let mut stream = TcpStream::connect(self.server_addr)?;

        // write the message and flush to allow the server to begin reading
        debug!("write to stream");
        msg.write(&mut stream)?;

        // read the response from the server
        let resp = Message::recv(&mut stream)?;

        Ok(resp)
    }
}

#[cfg(test)]
mod tests {}
