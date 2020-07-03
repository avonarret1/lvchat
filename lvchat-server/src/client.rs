use std::{net::TcpStream, sync::Arc};

use parking_lot::{Mutex, RwLock};

use lvchat_core::User;

#[derive(Debug, Clone)]
pub struct Client {
    pub stream: Arc<Mutex<TcpStream>>,
    pub user: Arc<RwLock<User>>,
}

impl Client {
    pub fn new(stream: TcpStream) -> Self {
        let addr = stream.peer_addr().unwrap();

        Client {
            stream: Arc::new(Mutex::new(stream)),
            user: Arc::new(RwLock::new(User::Ghost { addr })),
        }
    }
}

impl PartialEq<Self> for Client {
    fn eq(&self, other: &Client) -> bool {
        *self.user.read() == *other.user.read()
    }
}

impl Eq for Client {}

impl std::fmt::Display for Client {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.user.read())
    }
}
