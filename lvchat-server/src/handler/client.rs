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
    log::info!("[Client: {}] Connected.", client);

    sender.send(Event::Accepted(client.clone()));

    'main: loop {
        if let Some(mut client_stream) = client.stream.try_lock() {
            match client_stream.read(&mut buffer) {
                Ok(0) => (),

                Ok(size) => {
                    data.extend_from_slice(&buffer[0..size]);
                }

                Err(e) => match e.kind() {
                    ErrorKind::ConnectionReset | ErrorKind::TimedOut => {
                        log::warn!("[Client: {}] Disconnected forcefully. Reason: {}", client, e);

                        break 'main;
                    }

                    ErrorKind::WouldBlock => (),

                    _ => {
                        log::warn!("[Client: {}] Error was not processed: {}", client, e);
                    }
                },
            }
        }

        if data.is_empty() {
            yield_now();
        }

        if let Some(eol) = data.windows(2).position(|w| w == b"\r\n") {
            let raw = data.drain(..eol+2).collect::<Vec<_>>();
            let message = Message::from_bytes(&raw[..eol]);

            if let Some(message) = message {
                log::info!("[Client: {}] Received message: {:#?}", client, message);

                handle_message(&state, &client, message, sender.clone());

            // let _ = sender.send(Event::Message(stream.clone(), message));
            } else {
                log::warn!("Received invalid message. Skipping");
                yield_now();
            }
        }
    }

    sender.send(Event::Dropped(client));

    log::trace!("Stopping client handler thread");
}

fn broadcast_user_message(state: &State, client: &Client, message: &UserMessage) {
    let refer = Message::Server(ServerMessage::Refer {
        user: client.user.read().nick_unchecked().to_owned(),
        message: message.clone(),
    });

    for client in get_all_clients_with_exception(&state, &[client]) {
        let _ = client.stream.lock().write(&refer.to_bytes());
    }
}

fn handle_message(state: &State, client: &Client, message: Message, sender: Sender<Event>) {
    if client.user.read().is_ghost() {
        match &message {
            Message::User(message) => {
                match message {
                    UserMessage::Auth { nick } => {
                        if let Some(other_user) = state.get_client_by_name(&nick) {
                            client
                                .stream
                                .lock()
                                .write(&Message::Error(ErrorMessage::NickNameInUse).to_bytes());
                        } else {
                            log::info!("[Client: {}] Now authenticated as {}", client, nick);

                            let user = client.user.read().clone();

                            *client.user.write() = lvchat_core::User::Authenticated {
                                nick: nick.clone(),
                                addr: *user.addr(),
                            };

                            sender.send(Event::Authenticated(client.clone()));
                        }
                    }

                    _ => {
                        log::info!(
                            "[Client: {}] Sent message without being authenticated: {:#?}",
                            client,
                            message,
                        );
                    }
                }

                broadcast_user_message(state, client, message);
            }

            _ => {
                log::warn!("[Client: {}] Invalid message received: {:#?}", client, message);
                log::warn!("[Client: {}] Skipping.", client);
            }
        }
    } else {
        match message {
            Message::User(message) => {
                let mut broadcast = true;

                match &message {
                    UserMessage::Auth { nick } => {
                        if state.get_client_by_name(&nick).is_some() {
                            client
                                .stream
                                .lock()
                                .write(&Message::Error(ErrorMessage::NickNameInUse).to_bytes());
                        } else {
                            log::info!(
                                "[Client: {}] Changing nick to {}",
                                client,
                                nick
                            );

                            let mut user = client.user.write();

                            *user = User::Authenticated {
                                addr: user.addr().clone(),
                                nick: nick.clone(),
                            };
                        }
                    }

                    UserMessage::Leave { message } => {
                        log::info!(
                            "[Client: {}] Is leaving ({:?})",
                            client,
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
                        let users = get_all_clients_with_exception(&state, &[client])
                            .into_iter()
                            .filter_map(|client| client.user.read().nick().map(ToOwned::to_owned))
                            .collect::<Vec<_>>();

                        let response = Message::Server(ServerMessage::UserList { users });
                        client.stream.lock().write(&response.to_bytes());

                        broadcast = false;
                    }

                    UserMessage::Text { message } => {

                    }

                    UserMessage::Voice { stream: _ } => {}
                }

                if broadcast {
                    broadcast_user_message(state, client, &message);
                }
            }

            _ => {
                log::warn!("[Client: {}] Invalid message received: {:#?}", client, message);
                log::warn!("[Client: {}] Skipping.", client);
            }
        }
    }
}

fn get_all_clients_with_exception(state: &State, excpetions: &[&Client]) -> Vec<Client> {
    let mut clients = vec![];

    for client in state.clients.lock().iter() {
        if excpetions.contains(&client) {
            continue;
        }

        clients.push(client.clone());
    }

    clients
}
