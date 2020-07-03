use std::{net::TcpStream, sync::Arc};

use parking_lot::{Mutex, RwLock};

use crate::{
    config::Config,
    view::{Message, User}
};

#[derive(Debug, Clone)]
pub struct State {
    pub config: Arc<Config>,

    pub users: Arc<RwLock<Vec<User>>>,
    pub messages: Arc<RwLock<Vec<Message>>>,

    pub input: Arc<RwLock<String>>,
    pub stream: Arc<Mutex<TcpStream>>,
}

impl State {
    pub fn new(config: Config, stream: TcpStream) -> Self {
        let nick = config.nick.clone();
        let config = Arc::new(config);

        State {
            config,

            users: Arc::new(RwLock::new(vec![nick])),
            messages: Arc::new(RwLock::new(vec![])),

            input: Arc::new(RwLock::new(String::new())),
            stream: Arc::new(Mutex::new(stream)),
        }
    }
}
