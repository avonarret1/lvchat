use lvchat_core::Message;

use crate::io::user::State as UserInputState;

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
