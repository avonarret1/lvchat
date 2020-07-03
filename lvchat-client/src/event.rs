use crate::{io::user::State as UserInputState, state::State};
use flume::{Receiver, TryRecvError};
use lvchat_core::message::{Error, Server, User};
use lvchat_core::Message;

#[derive(Debug)]
pub enum Event {
    UserInput(UserInputState),
    Message(Message),
    Disconnected,
}

impl From<Message> for Event {
    fn from(message: Message) -> Self {
        Self::Message(message)
    }
}

impl From<UserInputState> for Event {
    fn from(input: UserInputState) -> Self {
        Self::UserInput(input)
    }
}
