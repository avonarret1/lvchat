use std::{net::TcpStream, sync::Arc};

use parking_lot::{Mutex, RwLock};

use crate::view::{Message, User};

#[derive(Debug, Clone)]
pub struct State {
    pub users: Arc<RwLock<Vec<User>>>,
    pub messages: Arc<RwLock<Vec<Message>>>,

    pub input: Arc<RwLock<String>>,
    pub stream: Arc<Mutex<TcpStream>>,
}

impl State {
    pub fn new(stream: TcpStream) -> Self {
        State {
            users: Arc::new(RwLock::new(vec![])),
            messages: Arc::new(RwLock::new(vec![])),

            input: Arc::new(RwLock::new(String::new())),
            stream: Arc::new(Mutex::new(stream)),
        }
    }
}