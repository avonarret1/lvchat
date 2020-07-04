use crate::client::Client;

#[derive(Debug, Clone)]
pub enum Event {
    Accepted(Client),
    Authenticated(Client),
    Dropped(Client),
}
