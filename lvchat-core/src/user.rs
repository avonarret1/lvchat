use std::net::SocketAddr;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum User {
    Ghost { addr: SocketAddr },

    Authenticated { nick: String, addr: SocketAddr },
}

impl User {
    pub fn is_ghost(&self) -> bool {
        matches!(self, User::Ghost { .. })
    }

    pub fn is_authenticated(&self) -> bool {
        matches!(self, User::Authenticated { .. })
    }
}

impl User {
    pub fn addr(&self) -> &SocketAddr {
        match self {
            Self::Ghost { ref addr } | Self::Authenticated { ref addr, .. } => addr,
        }
    }

    pub fn nick(&self) -> Option<&str> {
        match self {
            Self::Authenticated { ref nick, .. } => Some(nick.as_str()),

            _ => None,
        }
    }

    pub fn nick_unchecked(&self) -> &str {
        match self {
            Self::Authenticated { ref nick, .. } => nick.as_str(),

            _ => unreachable!(),
        }
    }
}