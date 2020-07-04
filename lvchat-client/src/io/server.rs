use std::{
    io::{ErrorKind, Read},
    net::TcpStream,
    sync::Arc,
    thread::{spawn, yield_now},
};

use flume::Receiver;
use parking_lot::Mutex;

use lvchat_core::Message;

use crate::event::Event;

pub fn capture(stream: Arc<Mutex<TcpStream>>) -> Receiver<Event> {
    let (tx, rx) = flume::unbounded();

    let _ = stream.lock().set_nonblocking(true);

    spawn(move || {
        let mut data = String::new();

        loop {
            if let Some(mut stream) = stream.try_lock() {
                match stream.read_to_string(&mut data) {
                    Ok(_) => {}
                    Err(e) => match e.kind() {
                        ErrorKind::ConnectionAborted
                        | ErrorKind::ConnectionReset
                        | ErrorKind::TimedOut => {
                            let _ = tx.send(Event::Disconnected);
                            return;
                        }
                        _ => (),
                    },
                }
            }

            if let Some(line) = data.lines().next().map(ToString::to_string) {
                match Message::from_bytes(&line.bytes().collect::<Vec<_>>()) {
                    Some(message) => {
                        let _ = tx.send(message.into());

                        data.drain(..line.len());
                        data = data.trim().to_owned();
                    }
                    None => yield_now(),
                }
            } else {
                yield_now()
            }
        }
    });

    rx
}
