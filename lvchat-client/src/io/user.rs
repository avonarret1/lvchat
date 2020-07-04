use std::{
    io::Read,
    ops::{Deref, DerefMut},
    thread::spawn,
};

use flume::Receiver;

use crate::event::Event;

#[derive(Debug)]
pub enum State {
    Edit(String),
    Sent(String),
}

impl State {
    pub fn is_sent(&self) -> bool {
        matches!(self, Self::Sent(_))
    }
}

impl Deref for State {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        match self {
            State::Edit(data) | State::Sent(data) => data,
        }
    }
}

impl DerefMut for State {
    fn deref_mut(&mut self) -> &mut <Self as Deref>::Target {
        match self {
            State::Edit(ref mut data) | State::Sent(ref mut data) => data,
        }
    }
}

pub fn capture() -> Receiver<Event> {
    let (tx, rx) = flume::unbounded();

    spawn(move || {
        let mut input = String::new();

        loop {
            match std::io::stdin().bytes().take(1).last().unwrap() {
                Ok(any) => match any {
                    b'\n' => {
                        let _ = tx.send(State::Sent(input.clone()).into());

                        input.clear();

                        continue;
                    }

                    b'\r' => {}

                    // backspace
                    8 => {
                        input.pop();
                    }

                    other if !other.is_ascii_control() => {
                        input.push(other as char);
                    }

                    _ => (),
                },

                _ => (),
            }

            let _ = tx.send(State::Edit(input.clone()).into());
        }
    });

    rx
}
