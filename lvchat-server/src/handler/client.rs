use std::{
    io::{ErrorKind, Read, Write},
    net::TcpStream,
    sync::Arc,
    thread::yield_now,
};

use flume::Sender;

use parking_lot::Mutex;

use lvchat_core::*;

use crate::{client::Client, event::Event, state::State};

pub fn handle(state: State, client: Client, sender: Sender<Event>) {
    let mut data = vec![];
    let mut buffer = [0u8; 1024];

    log::trace!("Started client handler thread");
    log::info!("Client connected.");

    sender.send(Event::Accepted(client.clone()));

    loop {
        if let Some(mut client_stream) = client.stream.try_lock() {
            match client_stream.read(&mut buffer) {
                Ok(0) => yield_now(),

                Ok(_) => {
                    data.extend_from_slice(&buffer);
                }

                Err(e) => match e.kind() {
                    ErrorKind::ConnectionReset | ErrorKind::TimedOut => {
                        log::warn!("Client disconnected. Reason: {}", e);

                        //let _ = sender.send(Event::Quit(stream.clone()));

                        break;
                    }

                    ErrorKind::WouldBlock => (),

                    _ => {
                        log::warn!("Error was not processed: {}", e);
                    }
                },
            }
        }

        if data.is_empty() {
            yield_now();
        }

        if let Some(eol) = data.windows(2).position(|w| w == b"\r\n") {
            let raw = data.drain(..eol).collect::<Vec<_>>();
            let message = Message::from_bytes(&raw);
            let _ = data.drain(..2);

            if let Some(message) = message {
                log::info!("Received message: {:#?}", message);

                handle_message(&state, &client, message, sender.clone());

            // let _ = sender.send(Event::Message(stream.clone(), message));
            } else {
                log::warn!("Received invalid message. Skipping");
                yield_now();
            }
        }
    }

    sender.send(Event::Dropped(client));

    log::info!("Client disconnected");
}

fn handle_message(state: &State, client: &Client, message: Message, sender: Sender<Event>) {
    if client.user.read().is_ghost() {
        match message {
            Message::User(message) => match message {
                UserMessage::Auth { nick } => {
                    if let Some(other_user) = state.get_client_by_name(&nick) {
                        client
                            .stream
                            .lock()
                            .write(&Message::Error(ErrorMessage::NickNameInUse).to_bytes());
                    } else {
                        log::info!("User is now authenticated as {}", nick);

                        let user = client.user.read().clone();

                        *client.user.write() = lvchat_core::User::Authenticated {
                            nick,
                            addr: *user.addr(),
                        };

                        sender.send(Event::Authenticated(client.clone()));
                    }
                }

                _ => {
                    log::info!(
                        "User ({}) send message without being authenticated: {:#?}",
                        client.user.read().addr(),
                        message,
                    );
                }
            },

            _ => {
                log::warn!("Invalid message received: {:#?}", message);
                log::warn!("Skipping.");
            }
        }
    } else {
        match message {
            Message::User(message) => match message {
                UserMessage::Auth { nick } => {
                    if state.get_client_by_name(&nick).is_some() {
                        client
                            .stream
                            .lock()
                            .write(&Message::Error(ErrorMessage::NickNameInUse).to_bytes());
                    } else {
                        let mut user = client.user.write();

                        log::info!(
                            "User {} ({}) changed nick to {}",
                            user.nick_unchecked(),
                            user.addr().ip(),
                            nick
                        );

                        *user = User::Authenticated {
                            addr: user.addr().clone(),
                            nick,
                        };
                    }
                }

                UserMessage::Leave { message } => {
                    log::info!(
                        "User {} is leaving ({:?})",
                        client.user.read().nick_unchecked(),
                        message
                    );

                    let mut clients = state.clients.lock();
                    let client_pos = clients
                        .iter()
                        .position(|client_x| client == client_x)
                        .expect("Client in list");

                    clients.remove(client_pos);
                }

                UserMessage::RequestUserList => {
                    let current_user = client.user.read();
                    let mut user_list = vec![];

                    for other_client in state.clients.lock().iter() {
                        let other_user = other_client.user.read();

                        if other_user.is_authenticated() && current_user.nick() != other_user.nick()
                        {
                            user_list.push(other_user.nick_unchecked().to_string());
                        }
                    }

                    drop(current_user);

                    let response = Message::Server(ServerMessage::UserList { users: user_list });
                    client.stream.lock().write(&response.to_bytes());
                }

                UserMessage::Text { message: _ } => {}
                UserMessage::Voice { stream: _ } => {}
            },

            _ => {
                log::warn!("Invalid message received: {:#?}", message);
                log::warn!("Skipping.");
            }
        }
    }
}
