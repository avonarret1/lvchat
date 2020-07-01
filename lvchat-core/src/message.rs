use serde::{Deserialize, Serialize};

/// Enumeration of the network protocol lvchat is using
#[repr(C)]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Message {
    /// User to server
    User(User),

    /// Server to user
    Server(Server),

    /// Client to server errors
    Error(Error),
}

#[repr(C)]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum User {
    Auth { nick: String },
    Leave { message: Option<String> },

    RequestUserList,

    Text { message: String },

    Voice { stream: Vec<u8> },
}

#[repr(C)]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Server {
    // Shutdown { message: Option<String> },

    // MessageOfTheDay { message: Option<String> },
    Notice { message: String },

    Refer { user: String, message: User },

    UserList { users: Vec<String> },
}

#[repr(C)]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Error {
    /// Client is already connected.
    AlreadyConnected,

    /// Requested nick is already in use
    NickNameInUse,
}

impl Message {
    pub fn to_bytes(&self) -> Vec<u8> {
        bincode2::serialize(&self).unwrap()
    }

    pub fn from_bytes(data: &[u8]) -> Option<Self> {
        bincode2::deserialize(data).ok()
    }
}

impl From<User> for Message {
    fn from(user: User) -> Self {
        Self::User(user)
    }
}

impl From<Server> for Message {
    fn from(server: Server) -> Self {
        Self::Server(server)
    }
}

impl From<Error> for Message {
    fn from(error: Error) -> Self {
        Self::Error(error)
    }
}

#[test]
fn bin_conv() {
    let origin = Message::User(User::Leave { message: None });
    let origin_encoded = bincode2::serialize(&origin).unwrap();
    let deserialized: Message = bincode2::deserialize(&origin_encoded).unwrap();

    assert_eq!(origin, deserialized);
}
