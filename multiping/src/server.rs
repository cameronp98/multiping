//! Core server stuff

use std::collections::{HashMap, VecDeque};
use std::net::{TcpListener, TcpStream};
use std::sync::mpsc::{channel, Receiver, Sender, TryRecvError};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};

use crate::error::{Error, Result};
use crate::message::Message;

// A type alias that represents
type JobHandle = JoinHandle<()>;

#[derive(Debug, Clone)]
enum ServerMessage {
    Message(Message),
    Disconnect,
}

type ServerMessages = Arc<Mutex<VecDeque<(RemoteClientId, ServerMessage)>>>;

/// The multiping server
#[derive(Debug)]
pub struct Server {
    listener: Option<JoinHandle<()>>,
    // clients: Arc<Mutex<HashMap<RemoteClientId, RemoteClient>>>,
    clients: Arc<Mutex<RemoteClients>>,
}

impl Server {
    /// Create a server and bind it to the given address
    pub fn new() -> Server {
        debug!("Creating server...");

        Server {
            listener: None,
            clients: Arc::new(Mutex::new(RemoteClients::new())),
        }
    }

    /// Spawn a new thread to listen for connections
    pub fn run(&mut self, addr: &str) -> Result<()> {
        debug!("Running server on {}...", addr);

        println!("Server running on {}", addr);

        let messages = Arc::new(Mutex::new(VecDeque::new()));

        let listener = TcpListener::bind(addr)?;
        let clients = self.clients.clone();

        // spawn listener thread
        self.listener = Some(thread::spawn(move || {
            // Spawn a thread for each incoming connection
            for stream in listener.incoming() {
                stream
                    .map(|s| clients.lock().unwrap().create_client(s, messages.clone()))
                    .map_err(|e| warn!("Error on incoming stream: {:?}", e));
            }
        }));

        Ok(())
    }
}

impl std::ops::Drop for Server {
    /// Try and close the connection or error
    fn drop(&mut self) {
        debug!("Dropping server:");

        debug!("Terminating all clients...");
        match self.clients.lock().unwrap().terminate_all() {
            Ok(_) => debug!("Success."),
            Err(e) => error!("Error terminating all clients: {:?}", e),
        }
    }
}

#[derive(Debug, Clone)]
enum RemoteClientMessage {
    Terminate,
    Message(Message),
}

pub type RemoteClientId = usize;
type WorkerResult = Result<()>;

/// A collection of remote clients
#[derive(Debug)]
pub struct RemoteClients {
    next_id: RemoteClientId,
    clients: Arc<Mutex<HashMap<RemoteClientId, RemoteClient>>>,
}

impl RemoteClients {
    fn new() -> RemoteClients {
        RemoteClients {
            next_id: 0,
            clients: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    fn create_client(&mut self, stream: TcpStream, messages: ServerMessages) -> RemoteClientId {
        let id = self.next_id;
        self.next_id += 1;

        let client = RemoteClient::new(id, stream, messages);
        self.clients.lock().unwrap().insert(id, client).unwrap();

        id
    }

    fn send_one(&mut self, msg: RemoteClientMessage, id: RemoteClientId) -> Result<()> {
        self.clients
            .lock()
            .unwrap()
            .get_mut(&id)
            .ok_or(Error::InvalidRemoteClientId(id))
            .and_then(|client| client.send(msg))
    }

    fn send_all(&mut self, msg: RemoteClientMessage) {
        debug!("Sending to all clients {:?}...", msg);

        let mut dead_clients = Vec::new();

        for (&id, client) in self.clients.lock().unwrap().iter_mut() {
            debug!("Sending message.");
            match client.send(msg.clone()) {
                Ok(_) => debug!("Ok."),
                Err(e) => {
                    warn!("Found dead client {}: {:?}", id, e);
                    dead_clients.push(id);
                }
            }
        }

        for id in dead_clients {
            debug!("Removing dead client {}", id);
            self.remove(id);
        }
    }

    fn remove(&mut self, id: RemoteClientId) -> Result<RemoteClient> {
        debug!("Removing client {}...", id);
        self.clients
            .lock()
            .unwrap()
            .remove(&id)
            .ok_or(Error::InvalidRemoteClientId(id))
    }

    fn terminate_all(&mut self) -> Result<Vec<WorkerResult>> {
        debug!("Terminating all remote clients...");

        self.clients
            .lock()
            .unwrap()
            .drain()
            .map(|(_, client)| client.terminate())
            .collect()
    }
}

/// Remote client
///
/// * `handle` - A handle to the worker thread
/// * `sender` - The `Sender` used to send `RemoteClientMessage`s to the worker thread
#[derive(Debug)]
struct RemoteClient {
    id: RemoteClientId,
    handle: JoinHandle<WorkerResult>,
    sender: Sender<RemoteClientMessage>,
}

impl RemoteClient {
    /// Create a new client with a queue to send responses to
    fn new(id: RemoteClientId, mut stream: TcpStream, messages: ServerMessages) -> RemoteClient {
        let (tx, rx) = channel();

        let handle = thread::spawn(move || {
            loop {
                // Read a command from the server
                match rx.recv()? {
                    RemoteClientMessage::Terminate => {
                        Message::Disconnect.write(&mut stream)?;
                        break;
                    }
                    RemoteClientMessage::Message(m) => {
                        debug!("RemoteCient({}) got actual message {:?}", id, m)
                    }
                }

                // Read a message from the client and send it back to the server
                match Message::read(&mut stream) {
                    Ok(msg) => {
                        debug!("Got message {:?}", msg);
                        messages
                            .lock()
                            .unwrap()
                            .push_back((id, ServerMessage::Message(msg)));
                    }
                    Err(e) => {
                        error!("Error reading message from stream: {:?}", e);
                    }
                }
            }

            Ok(())
        });

        RemoteClient {
            id,
            handle,
            sender: tx,
        }
    }

    fn send(&mut self, msg: RemoteClientMessage) -> Result<()> {
        debug!("Sending message...");

        self.sender.send(msg).map_err(|e| {
            error!("Error sending message.");
            Error::SendError
        })
    }

    fn terminate(mut self) -> Result<WorkerResult> {
        debug!("Stopping client...");

        debug!("Requesting termination...");
        self.send(RemoteClientMessage::Terminate)?;

        debug!("Joining thread...");
        self.handle.join().map_err(|_| Error::JoinError)
    }
}

#[cfg(test)]
mod tests {}
