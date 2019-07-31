use std::collections::HashMap;
use std::net::TcpStream;
use std::sync::mpsc::{channel, Sender, TryRecvError};
use std::thread::{self, JoinHandle};

use crate::{Error, Message, Result};

/// The poll interval for a worker thread handling incoming messages
// const POLL_READ_INTERVAL: Duration = Duration::from_millis(200);

#[derive(Debug)]
// The message type for communicating with worker threads
enum Action {
    /// Requests the worker to write the given message to its client stream
    Forward(Message),

    /// Request the worker to shutdown gracefully so that it can be `join`ed for its results
    ///
    /// Note: the client must be notified of the disconnect (via a [`Message::Disconnect`])
    /// before telling the worker thread to disconnect.
    /// For instance:
    /// ```
    /// worker_tx.send(Action::Forward(Message::Disconnect));
    /// worker_tx.send(Action::Disconnect);
    /// ```
    Disconnect,
}

/// The unique ID of a [`Connection`]
pub type ConnectionId = usize;
pub type ConnectionOutput = (ConnectionId, Message);

/// A connection which simultaneously sends and receives messages without blocking
///
/// @TODO upgrade to be generic over a message trait instead of [`crate::message::Message`]
#[derive(Debug)]
pub struct Connection {
    /// The unique ID of this connection (used by [`ConnectionRegistry'] and for debug info)
    id: ConnectionId,

    /// The worker thread which recives messages
    send_worker: Option<JoinHandle<Result<()>>>,

    /// The worker thread which writes outgoing messages to the client stream
    recv_worker: Option<JoinHandle<Result<()>>>,

    /// Sends actions to the sender worker
    send_tx: Sender<Action>,

    /// Sends actions to the receiver worker
    recv_tx: Sender<Action>,
}

impl Connection {
    /// Create a new connection sending and receiving on `stream`
    ///
    /// Creates two threads:
    /// * The sender thread, which
    pub fn new(
        id: ConnectionId,
        stream: TcpStream,
        sender: Sender<ConnectionOutput>,
    ) -> Connection {
        debug!("create connection");

        debug!("create worker threads");
        let (send_worker, send_tx) = spawn_recv_worker(
            id,
            stream
                .try_clone()
                .expect("failed to clone connection stream"),
            sender,
        );
        let (recv_worker, recv_tx) = spawn_send_worker(stream);

        debug!("connection created successfully");

        Connection {
            id,
            send_worker: Some(send_worker),
            recv_worker: Some(recv_worker),
            send_tx,
            recv_tx,
        }
    }

    /// Retrieve the connection's id
    pub fn id(&self) -> ConnectionId {
        self.id
    }

    /// Send a message to the client through the sender worker
    pub fn forward(&mut self, msg: Message) -> Result<()> {
        match self.send_tx.send(Action::Forward(msg)) {
            Ok(()) => unimplemented!(),
            Err(e) => {
                // sender closed, treat connection as disconnected
                error!("failed to send action to send worker: {}", e);
                return Err(Error::SenderDisconnected);
            }
        }

        debug!("message forwarded");

        Ok(())
    }

    /// Disconnect the connection
    pub fn disconnect(&mut self) {
        debug!("disconnecting connection");

        // tell the client we are disconnecting
        debug!("disconnecting client");
        if let Err(e) = self.forward(Message::Disconnect) {
            error!("failed to forward disconnect message: {}", e);
        }

        // disconnect the send worker
        debug!("disconnecting send worker");
        if self.send_worker.take().is_some() {
            let _ = self
                .send_tx
                .send(Action::Disconnect)
                .map_err(|e| error!("failed to send disconnect to send worker: {:?}", e));
        } else {
            warn!("send worker is already disconnected");
        }

        // disconnect the receive worker
        debug!("disconnecting recv worker");
        if self.recv_worker.take().is_some() {
            let _ = self
                .recv_tx
                .send(Action::Disconnect)
                .map_err(|e| error!("failed to send disconnect to recv worker: {:?}", e));
        } else {
            warn!("recv worker is already disconnected");
        }
    }
}

impl std::ops::Drop for Connection {
    fn drop(&mut self) {
        debug!("dropping connection");

        debug!("disconnecting");
        self.disconnect();
    }
}

/// Spawn a recieve worker thread
///
/// # Arguments
/// * `id` - The connection's unique ID
/// * `stream` - The stream to monitor for messages
/// * `msg_tx` - The sender for received messages
fn spawn_recv_worker(
    id: ConnectionId,
    mut stream: TcpStream,
    msg_tx: Sender<ConnectionOutput>,
) -> (JoinHandle<Result<()>>, Sender<Action>) {
    debug!("spawn writing worker thread");

    let (action_tx, action_rx) = channel();

    let handle = thread::spawn(move || {
        'main: loop {
            // Poll for actions from sent from the main thread
            loop {
                debug!("checking for pending messages");
                match action_rx.try_recv() {
                    Ok(action) => {
                        debug!("client thread got action: {:?}", action);
                        debug!("forward to client");

                        match action {
                            Action::Disconnect => {
                                debug!("disconnect recv thread");
                                break 'main;
                            }
                            Action::Forward(msg) => {
                                panic!(
                                    "cannot forward messages from the receiver thread ({})",
                                    msg
                                );
                            }
                        }
                    }
                    Err(e) => match e {
                        TryRecvError::Empty => {
                            debug!("no pending actions");
                            break;
                        }
                        TryRecvError::Disconnected => {
                            error!("sender disconnected, cannot receive actions");
                            return Err(Error::SenderDisconnected);
                        }
                    },
                }
            }

            debug!("read message from client");

            // Relay a message from the client back to the main thread
            Message::recv(&mut stream)
                .and_then(|msg| {
                    info!("client sent a message: {}", msg);
                    debug!("relaying back to main thread");
                    msg_tx.send((id, msg)).map_err(|e| {
                        error!("error forwarding message to server: {}", e);
                        Error::SendError
                    })
                })
                .map_err(|e| {
                    error!("error recieving message from client: {}", e);
                    e
                })?;
        }

        Ok(())
    });

    (handle, action_tx)
}

/// Spawn a worker thread which forwards outgoing messages on from the main thread
/// to the client through the given `TcpStream`
///
/// # Arguments
///
/// * `stream` - The stream to write received messages to
///
/// Returns
fn spawn_send_worker(mut stream: TcpStream) -> (JoinHandle<Result<()>>, Sender<Action>) {
    let (action_tx, action_rx) = channel();

    let handle = thread::spawn(move || {
        // Note: it is the duty of the server to forward a disconnect `Message`
        // to the client before sending the disconnect `Action` to this thread

        // repeatedly block on `send_rx` until an `Action` is received
        for action in action_rx {
            match action {
                Action::Forward(msg) => {
                    msg.write(&mut stream).map_err(|e| {
                        error!("failed to forward message: {}", e);
                        e
                    })?;
                }
                Action::Disconnect => {
                    debug!("disconnecting send thread");
                    break;
                }
            }
        }

        Ok(())
    });

    (handle, action_tx)
}

/// A collection for allocating and managing many client connections
#[derive(Debug, Default)]
pub struct ConnectionRegistry {
    next_id: ConnectionId,
    connections: HashMap<ConnectionId, Connection>,
}

impl ConnectionRegistry {
    pub fn new() -> ConnectionRegistry {
        ConnectionRegistry {
            next_id: 0,
            connections: HashMap::new(),
        }
    }

    pub fn add(&mut self, stream: TcpStream, msg_tx: Sender<ConnectionOutput>) -> ConnectionId {
        debug!("register connection");

        let id = self.next_id;
        self.next_id += 1;
        debug!("id: {}", id);

        let conn = Connection::new(id, stream, msg_tx);
        debug!("connection object created");

        // duplicate keys should be impossible as `next_id` is incremented before every insert
        if let Some(dupe) = self.connections.insert(id, conn) {
            panic!("connection {} already exists in registry: {:?}", id, dupe);
        }

        debug!("connection registered successfully");

        id
    }

    pub fn forward_to_all(&mut self, msg: Message, source: ConnectionId) -> Result<()> {
        debug!("forward to all connections: {}", msg);

        let mut dead_conns = Vec::new();

        for (&id, conn) in self.connections.iter_mut() {
            // don't send to the source connection
            if id == source {
                debug!("skip source connection {}", source);
                continue;
            }

            // try and send to the client, or mark it as dead
            debug!("forwarding to connection {}", id);
            match conn.forward(msg.clone()) {
                Ok(_) => debug!("message forwarded"),
                Err(e) => {
                    warn!("found dead client {}: {}", id, e);
                    dead_conns.push(id);
                }
            }
        }

        debug!("messages forwarded");

        debug!("clean up dead connections");

        for id in dead_conns {
            debug!("remove dead connection {}", id);
            self.disconnect(id)
                .unwrap_or_else(|_| panic!("failed to remove client: {}", id));
        }

        Ok(())
    }

    /// Remove a connection from the registry
    pub fn remove(&mut self, id: ConnectionId) -> Result<Connection> {
        debug!("remove connection {}", id);
        self.connections
            .remove(&id)
            .ok_or(Error::InvalidConnectionId(id))
    }

    /// Remove a connection from the registry and disconnect it
    pub fn disconnect(&mut self, id: ConnectionId) -> Result<()> {
        debug!("disconnect connection {}", id);
        self.remove(id).map(|mut conn| conn.disconnect())
    }

    /// Disconnect all connections
    pub fn disconnect_all(&mut self) -> Result<()> {
        debug!("disconnect all conns");

        for (id, mut conn) in self.connections.drain() {
            debug!("disconnect connection {}", id);
            conn.disconnect();
        }

        Ok(())
    }
}
