use std::fmt;
use std::io::{BufRead, BufReader, Read, Write};

use serde::{Deserialize, Serialize};

use crate::Result;

/// A message that can be sent and received over a stream#
///
/// @TODO upgrade this enum to a trait
#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum Message {
    Ping,
    Text(String),
    InvalidMessage,
    Disconnect,
    Error(String),
}

impl Message {
    /// Encode the message as JSON and send it down the stream
    pub fn write<W: Write>(&self, writer: &mut W) -> Result<()> {
        debug!("write message {}", self);

        // Serialize the message as JSON
        debug!("serialize message");
        let json = serde_json::to_string(&self)?;

        // Write the JSON followed by a newline
        debug!("write json + newline");
        writer.write_all(json.as_bytes())?;
        writeln!(writer)?;
        debug!("flush");
        writer.flush()?;

        debug!("message sent successfully");

        Ok(())
    }

    /// Try and construct a message from JSON read from a stream
    pub fn recv<R: Read>(reader: &mut R) -> Result<Message> {
        debug!("parse message from reader");

        let mut buf = BufReader::new(reader);

        // Read a line of JSON from the stream
        let mut json = String::new();
        debug!("read line");
        buf.read_line(&mut json)?;

        // Deserialize message from JSON
        debug!("deserialize");
        let msg = serde_json::from_str(&json)?;

        debug!("message read sucessfully: {}", msg);

        Ok(msg)
    }
}

impl fmt::Display for Message {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Message::Disconnect => write!(f, "Disconnect"),
            Message::InvalidMessage => write!(f, "Invalid message"),
            Message::Ping => write!(f, "Ping"),
            Message::Text(s) => write!(f, "'{}'", s),
            Message::Error(e) => write!(f, "error: {}", e),
        }
    }
}
