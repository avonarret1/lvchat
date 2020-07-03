use std::{
    thread::{spawn, yield_now},
    sync::Arc,
    net::TcpStream,
    io::{Read, ErrorKind},
};
use parking_lot::{Mutex, RawMutex};
use flume::Receiver;
use lvchat_core::Message;
use crate::{
    config::Config,
    event::Event,
};

pub fn capture(stream: Arc<Mutex<TcpStream>>) -> Receiver<Event> {
    let (tx, rx) = flume::unbounded();

    stream.lock().set_nonblocking(true);

    spawn(move || {
        let mut data = String::new();

        loop {
            if let Some(mut stream) = stream.try_lock() {
                match stream.read_to_string(&mut data) {
                    Ok(len) => {}
                    Err(e) => match e.kind() {
                        ErrorKind::ConnectionAborted | ErrorKind::ConnectionReset | ErrorKind::TimedOut => {
                            tx.send(Event::Disconnected);
                            return;
                        }
                        _ => (),
                    },
                }
            }

            if let Some(line) = data.lines().next() {
                match Message::from_bytes(&line.bytes().collect::<Vec<_>>()) {
                    Some(message) => {
                        tx.send(message.into());

                        data.drain(..line.len());
                        data = data.trim().to_owned();
                    }
                    None => {
                        yield_now()
                    }
                }
            } else {
                yield_now()
            }
        }
    });

    rx
}

