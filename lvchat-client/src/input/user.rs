use std::{
    io::{Read, Write},
    net::TcpStream,
    process::exit,
    sync::Arc,
    thread::{spawn, JoinHandle},
    time::Instant,
};

use flume::{bounded, Receiver};
use std::ops::Deref;

use crate::event::Event;

#[derive(Debug)]
pub enum State {
    Edit(String),
    Sent(String),
}

impl State {
    fn is_sent(&self) -> bool {
        matches!(self, Self::Sent(_))
    }
}

impl Deref for State {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        match self {
            State::Edit(data) | State::Sent(data) => {
                data.as_str()
            },
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
                        tx.send(State::Sent(input.clone()).into());

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

            tx.send(State::Edit(input.clone()).into());
        }
    });

    rx
}
