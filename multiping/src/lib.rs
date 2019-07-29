//! multiping module

#[macro_use]
extern crate log;

mod client;
mod error;
mod message;
mod server;

pub use client::Client;
pub use error::{Error, Result};
pub use message::Message;
pub use server::Server;

#[cfg(test)]
mod tests {
    
}
