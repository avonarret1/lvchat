use std::{net::TcpStream, process::exit};

use lvchat_core::{Message, UserMessage};

use crate::{
    config::Config, event::Event, io::user::State as UserInputState, state::State, view::View,
};

mod config;
mod event;
mod io;
mod message;
mod state;
mod view;

fn main() {
    let config = Config::new();

    init_logger(&config);

    let stream = connect(&config);

    let state = State::new(config, stream);
    let mut view = View::default();

    let events = [
        io::user::capture(),
        io::server::capture(state.stream.clone()),
    ];

    view.clear();

    'main: loop {
        // balance load. Rendering at least each 750ms(?) or so to make responses smoother to view
        for event_rx in &events {
            while let Ok(event) = event_rx.try_recv() {
                match event {
                    Event::UserInput(input) => {
                        handle_user_input(&state, input);
                    }
                    Event::Message(message) => {
                        handle_server_message(&state, message);
                    }
                    Event::Disconnected => {
                        break 'main;
                    }
                }

                view.update(&state);
                view.render(&state);
            }
        }
    }
}

fn init_logger(config: &Config) {
    let logger = if config.debug {
        flexi_logger::Logger::with_str("lvchat_core=debug, lvchat_client=debug")
    } else if config.verbose {
        flexi_logger::Logger::with_str("lvchat_core=info, lvchat_client=info")
    } else {
        flexi_logger::Logger::with_str("lvchat_core=error, lvchat_client=error")
    };

    let logger = if let Some(ref path) = config.logs_path {
        let logger = logger.log_to_file().directory(path);

        if !config.quiet {
            logger.duplicate_to_stderr(flexi_logger::Duplicate::All)
        } else {
            logger
        }
    } else if config.quiet {
        logger.do_not_log()
    } else {
        logger
    };

    logger.start().unwrap();
}

fn connect(config: &Config) -> TcpStream {
    println!(
        "Trying to connect to remote host ({}:{})",
        config.host, config.port
    );

    match TcpStream::connect((config.host.as_str(), config.port)) {
        Ok(stream) => {
            println!("Connected.");

            stream
        }

        Err(e) => {
            eprintln!("Failed to connect to remote host: {}", e);

            exit(0);
        }
    }
}

fn handle_user_input(state: &State, input_state: UserInputState) {
    if input_state.is_sent() {
        state.input.write().clear();

        match (*input_state).as_str() {
            "/quit" => {
                let _ = Message::send(
                    &mut *state.stream.lock(),
                    UserMessage::Leave { message: None },
                );

                let _ = state.stream.lock().shutdown(std::net::Shutdown::Both);

                std::process::exit(0);
            }

            _ => {
                let _ = Message::send(
                    &mut *state.stream.lock(),
                    UserMessage::Text {
                        message: input_state.trim().to_string(),
                    },
                );

                state
                    .messages
                    .write()
                    .push(view::Message::user(&state.config.nick, input_state.trim()));
            }
        }
    } else {
        *state.input.write() = (*input_state).clone();
    }
}

fn handle_server_message(state: &State, message: Message) {
    use lvchat_core::*;

    match message {
        Message::User(_user_message) => todo!("Reject"),

        Message::Server(server_message) => match server_message {
            ServerMessage::Auth => {
                let _ = Message::send(
                    &mut *state.stream.lock(),
                    UserMessage::Auth {
                        nick: state.config.nick.clone(),
                    },
                );
            }

            ServerMessage::Notice { message } => {
                state.messages.write().push(view::Message::notice(message));
            }

            ServerMessage::Refer {
                user,
                message: user_message,
            } => match user_message {
                UserMessage::Auth { nick } => {
                    if user != nick {
                        state.messages.write().push(view::Message::notice(format!(
                            "{} changed nick to {}",
                            user, nick
                        )));

                        for user in state.users.write().iter_mut() {
                            if user == &nick {
                                *user = nick.clone();
                            }
                        }
                    } else {
                        state
                            .messages
                            .write()
                            .push(view::Message::notice(format!("User joined: {}", nick)));

                        state.users.write().push(nick);
                    }
                }

                UserMessage::Leave { message: _ } => {
                    state
                        .messages
                        .write()
                        .push(view::Message::notice(format!("User left: {}", user)));

                    let mut users = state.users.write();
                    let pos = users
                        .iter()
                        .position(|user_x| user_x == &user)
                        .expect("User position (leaving)");

                    users.remove(pos);
                }

                UserMessage::RequestUserList => {}
                UserMessage::Text { message } => {
                    state
                        .messages
                        .write()
                        .push(view::Message::user(user, message));
                }
                UserMessage::Voice { .. } => {}
            },
            ServerMessage::UserList { mut users } => {
                users.insert(0, state.config.nick.clone());

                *state.users.write() = users;
            }
        },

        Message::Error(error_message) => match error_message {
            ErrorMessage::AlreadyConnected => {
                eprintln!("Already connected. Only one client per IP address allowed.");
                std::process::exit(0);
            }
            ErrorMessage::NickNameInUse => {
                eprintln!("Someone with that nickname is already connected.");
                std::process::exit(0);
            }
        },
    }
}
