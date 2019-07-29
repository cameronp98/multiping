//! Core server stuff

use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};

use crate::error::Result;
use crate::message::Message;

// A type alias that represents the result returned from by a joined job
type JobResult = Result<()>;
// A type alias that represents
type JobHandle = JoinHandle<JobResult>;

/// The multiping server
#[derive(Debug)]
pub struct Server {
    handles: Arc<Mutex<Vec<Option<JobHandle>>>>,
    listener: TcpListener,
}

impl Server {
    /// Create a server and bind it to the given address
    pub fn bind(addr: &str) -> Result<Server> {
        debug!("Creating listener on {}...", addr);
        let listener = TcpListener::bind(addr)?;

        Ok(Server {
            handles: Arc::new(Mutex::new(Vec::new())),
            listener,
        })
    }

    /// Spawn a new thread to listen for connections
    pub fn run(&mut self) {
        debug!("Running server:");

        println!("Server running on {}", self.listener.local_addr().unwrap());

        // Spawn a thread for each incoming connection
        for stream in self.listener.incoming() {
            match stream {
                Ok(s) => {
                    debug!("Spawning client handler thread.");
                    let handle = thread::spawn(|| handle_client(s));
                    debug!("Aquiring lock and saving thread handle...");
                    self.handles.lock().unwrap().push(Some(handle));
                }
                Err(e) => warn!("Error on incoming stream: {:?}", e),
            }
        }
    }
}

impl std::ops::Drop for Server {
    /// Try and close the connection or error
    fn drop(&mut self) {
        debug!("Dropping server:");

        debug!("Acquiring lock on saved handles...");
        for handle in self.handles.lock().unwrap().iter_mut() {
            if let Some(h) = handle.take() {
                debug!("Joining thread {:?}...", h);
                h.join().expect("Failed to join thread").unwrap();
                debug!("Thread joined.");
            }
        }
    }
}

/// Read the client's message, reverse it and send it back
///
/// Note: be careful with sending unicode strings as the message will be reversed as raw bytes
fn handle_client(mut stream: TcpStream) -> JobResult {
    debug!("Handling new client:");

    // read the message
    debug!("Reading message...");
    let msg = Message::read(&mut stream)?;

    // process the message
    debug!("Processing message...");
    let resp = match msg {
        Message::Ping => Message::Pong,
        Message::Text(text) => Message::Text(crate::util::reverse(text)),
        _ => Message::InvalidRequest,
    };

    // write the message and flush it to allow the client to begin reading
    debug!("Sending response...");
    resp.write(&mut stream)?;

    debug!("Client handled successfully.");

    Ok(())
}

#[cfg(test)]
mod tests {}
