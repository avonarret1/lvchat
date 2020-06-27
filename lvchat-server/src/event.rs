use crate::client::Client;
use lvchat_core::Message;
use parking_lot::Mutex;
use std::{net::TcpStream, sync::Arc};

#[derive(Debug, Clone)]
pub enum Event {
    Accepted(Client),
    Authenticated(Client),
    Dropped(Client),
}
