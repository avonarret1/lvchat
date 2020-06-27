use std::{
    net::{SocketAddr, TcpListener},
    sync::Arc,
    thread,
};

use parking_lot::Mutex;

use crate::{client::Client, config::Config};

#[derive(Debug, Clone)]
pub struct State {
    pub config: Arc<Config>,
    pub clients: Arc<Mutex<Vec<Client>>>,
}

impl State {
    pub fn new(config: Config) -> Self {
        State {
            config: Arc::new(config),
            clients: Arc::new(Mutex::new(vec![])),
        }
    }
}

impl State {
    pub fn get_user_by_addr(&self, addr: &SocketAddr) -> Option<Client> {
        for user in self.clients.lock().iter() {
            if user.user.read().addr().ip() == addr.ip() {
                return Some(user.clone());
            }
        }

        None
    }

    pub fn get_user_by_name(&self, name: &str) -> Option<Client> {
        for user in self.clients.lock().iter() {
            if user.user.read().nick() == Some(name) {
                return Some(user.clone());
            }
        }

        None
    }
}
