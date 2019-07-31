//! Core server stuff

use std::net::TcpListener;
use std::sync::mpsc::channel;
use std::sync::{Arc, Mutex, MutexGuard};
use std::thread::{self, JoinHandle};

use crate::connection::ConnectionRegistry;
use crate::error::{Error, Result};
use crate::message::Message;

/// The multiping server
#[derive(Debug)]
pub struct Server {
    listener: Option<JoinHandle<Result<()>>>,
    connections: Arc<Mutex<ConnectionRegistry>>,
}

impl Server {
    /// Create a server and bind it to the given address
    pub fn new() -> Server {
        debug!("create server");

        Server {
            listener: None,
            connections: Arc::new(Mutex::new(ConnectionRegistry::new())),
        }
    }

    pub fn connections(&mut self) -> MutexGuard<ConnectionRegistry> {
        debug!("acquire lock on client registry");
        self.connections.lock().expect("mutex poisoned")
    }

    /// Spawn a new thread to listen for connections
    pub fn run(&mut self, addr: &str) -> Result<()> {
        debug!("Running server on {}...", addr);

        println!("Server running on {}", addr);

        let (msg_tx, msg_rx) = channel();

        let listener = TcpListener::bind(addr)?;
        let conns = self.connections.clone();

        // spawn listener thread
        self.listener = Some(thread::spawn(move || {
            // Spawn a thread for each incoming connection
            for stream in listener.incoming() {
                info!("new incoming connection");
                match stream {
                    Ok(s) => {
                        // Add the client to the registry
                        debug!("lock client registry and register new connection");
                        conns.lock().expect("mutex poisoned").add(s, msg_tx.clone());
                    }
                    Err(e) => {
                        warn!("failed to accept connection: {}", e);
                    }
                }
            }

            Ok(())
        }));

        loop {
            // Read messages received from all connections
            debug!("wait for queued message from client handlers");

            match msg_rx.recv() {
                Ok((id, msg)) => {
                    debug!("received message from client {}: {}", id, msg);

                    match msg {
                        Message::Ping | Message::Text(_) => {
                            // distribute the message to the other clients
                            if let Err(e) = self.connections().forward_to_all(msg, id) {
                                error!("failed to forward message to all connections: {}", e);
                            }
                        }
                        Message::Disconnect => {
                            // disconnect the connection that produced the message
                            self.connections().disconnect(id)?;
                        }
                        _ => return Err(Error::UnexpectedMessage(msg)),
                    }
                }
                Err(e) => {
                    // failed to `recv` a message, all senders are dead
                    error!("error whilst receiving message: {}", e);
                    break;
                }
            }
        }

        Ok(())
    }
}

impl Default for Server {
    fn default() -> Self {
        Self::new()
    }
}

impl std::ops::Drop for Server {
    /// Try and close the connection or error
    fn drop(&mut self) {
        debug!("drop server");

        // @TODO implement messaging for listener so it can't hang
        if let Some(listener) = self.listener.take() {
            debug!("join listener thread");
            match listener.join() {
                Ok(res) => match res {
                    Ok(()) => debug!("listener joined"),
                    Err(err) => debug!("error in listener thread: {}", err),
                },
                Err(_) => error!("error joining listener"),
            }
        }

        debug!("disconnect all connections");

        match self.connections().disconnect_all() {
            Ok(_) => debug!("all connections disconnected successfully"),
            Err(e) => error!("error disconnecting all clients: {}", e),
        }
    }
}

#[cfg(test)]
mod tests {}
