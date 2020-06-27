use std::{
    io::{self, ErrorKind, Write},
    net::TcpStream,
    sync::Arc,
    thread::spawn,
};

use flume::{Receiver, Sender, TryRecvError};

use parking_lot::Mutex;

use lvchat_core::*;

use crate::{client::Client, event::Event, handler::client, state::State};

pub fn handle(state: State, client_queue_rx: Receiver<TcpStream>) {
    let (client_event_tx, client_event_rx) = flume::unbounded();

    log::trace!("Started server handling thread");

    loop {
        match client_queue_rx.try_recv() {
            Ok(mut client) => {
                handle_incoming_client(&state, client, &client_event_tx);
            }

            Err(e) => match e {
                TryRecvError::Disconnected => {
                    eprintln!("We've been disconnected. Stopping..");
                    break;
                }

                _ => (),
            },
        }

        match client_event_rx.try_recv() {
            Ok(event) => handle_event(&state, event),

            Err(e) => match e {
                TryRecvError::Disconnected => {
                    eprintln!("We've been disconnected. Stopping..");
                    break;
                }

                _ => (),
            },
        }
    }
}

fn handle_event(state: &State, event: Event) {
    match event {
        Event::Accepted(client) => {
            let notice = Message::Server(ServerMessage::Notice {
                message: format!("Welcome! Please authenticate yourself!"),
            });

            client.stream.lock().write(&notice.to_bytes());
        }
        Event::Authenticated(client) => {}
        Event::Dropped(client) => {}
    }
}

fn handle_incoming_client(state: &State, mut client_stream: TcpStream, client_tx: &Sender<Event>) {
    let addr = client_stream.peer_addr().unwrap();

    client_stream.set_nonblocking(true);

    log::info!("Processing client: {}", addr.ip());

    let client = match state.get_user_by_addr(&addr) {
        Some(client) => {
            let mut stream = client.stream.lock();

            match stream.take_error() {
                Ok(Some(ref e)) if e.kind() == ErrorKind::TimedOut => {
                    log::info!("Client ({}) timed out and has rejoined.", addr.ip());

                    *stream = client_stream;

                    drop(stream);

                    client
                }

                _ => {
                    client_stream.write(&Message::Error(ErrorMessage::AlreadyConnected).to_bytes());

                    log::info!("Client ({}) was dropped: Already joined.", addr.ip());
                    return;
                }
            }
        }

        None => {
            log::info!("New client: {}", addr.ip());

            let client = Client::new(client_stream);

            state.clients.lock().push(client.clone());

            client
        }
    };

    let state = state.clone();
    let client_tx = client_tx.clone();

    let _ = spawn(move || client::handle(state, client, client_tx));
}
