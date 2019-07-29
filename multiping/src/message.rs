use std::io::{BufRead, BufReader, Read, Write};

use serde::{Deserialize, Serialize};

use crate::Result;

/// A message that can be sent and received over a stream
#[derive(Debug, Deserialize, Serialize)]
pub enum Message {
    Ping,
    Pong,
    Text(String),
    InvalidRequest,
}

impl Message {
    /// Encode the message as JSON and send it down the stream
    pub fn write<W: Write>(self, writer: &mut W) -> Result<()> {
        debug!("Sending message:");

        // Serialize the message as JSON
        debug!("Serializing message...");
        let json = serde_json::to_string(&self)?;
        debug!("Message serialized.");

        // Write the JSON followed by a newline
        debug!("Writing bytes + newline...");
        writer.write_all(json.as_bytes())?;
        writeln!(writer)?;
        debug!("Bytes + newline written. Flushing...");
        writer.flush()?;
        debug!("Flushed.");

        debug!("Message sent succesfully.");

        Ok(())
    }

    /// Try and construct a message from JSON read from a stream
    pub fn read<R: Read>(reader: &mut R) -> Result<Message> {
        debug!("Reading a message:");

        let mut buf = BufReader::new(reader);

        // Read a line of JSON from the stream
        let mut json = String::new();
        debug!("Reading a line...");
        buf.read_line(&mut json)?;
        debug!("Line read.");

        // Deserialize message from JSON
        debug!("Deserializing message...");
        let msg = serde_json::from_str(&json)?;
        debug!("Deserialized.");

        debug!("Message read successfully.");

        Ok(msg)
    }
}
