#![allow(unused_variables, unused_imports)]

mod config;
mod event;
mod io;
mod state;
mod view;

use std::{
    io::{Read, Write},
    net::TcpStream,
    process::exit,
    sync::Arc,
    thread::spawn,
    time::Instant,
};

use parking_lot::Mutex;
use lvchat_core::{Message, UserMessage};
use crate::{config::Config, state::State, view::View, event::Event, io::user::State as UserInputState};

fn main() {
    let config = Config::new();

    init_logger(&config);

    let stream = connect(&config);

    let state = State::new(&config.nick, stream);
    let mut view = View::default();

    let events = [
        io::user::capture(),
        io::server::capture(state.stream.clone())
    ];

    view.clear();

    'main: loop {
        // balance load. Rendering at least each 750ms(?) or so to make responses smoother to view
        'events: for event_rx in &events {
            match event_rx.try_recv() {
                Ok(event) => match event {
                    Event::UserInput(input) => {
                        handle_user_input(&state, input);
                    }
                    Event::Message(message) => {
                        handle_server_message(&state, message);
                    },
                    Event::Disconnected => {
                        break 'main;
                    }
                }

                _ => (),
            }
        }

        view.update(&state);
        view.render(&state);
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
    } else {
        if config.quiet {
            logger.do_not_log()
        } else {
            logger
        }
    };

    logger.start().unwrap();
}

fn connect(config: &Config) -> TcpStream {
    use lvchat_core::UserMessage;

    println!(
        "Trying to connect to remote host ({}:{})",
        config.host, config.port
    );

    match TcpStream::connect((config.host.as_str(), config.port)) {
        Ok(mut stream) => {
            println!("Connected.");

            Message::send(&mut stream, UserMessage::Auth {
                nick: config.nick.clone()
            });

            return stream;
        }

        Err(e) => {
            eprintln!("Failed to connect to remote host: {}", e);

            exit(0);
        }
    }
}


fn handle_user_input(state: &State, input_state: UserInputState) {
    match input_state {
        UserInputState::Edit(input) => {
            *state.input.write() = input;
        }
        UserInputState::Sent(input) => {
            *state.input.write() = input.clone();

            Message::send(&mut *state.stream.lock(), UserMessage::Text {
                message: input.trim().to_string(),
            });

            state.messages.write().push(view::Message::user(&state.nick, input.trim()));

            state.input.write().clear();
        }
    }
}

fn handle_server_message(state: &State, message: Message) {
    use lvchat_core::*;

    match message {
        Message::User(user_message) => {
            todo!("Reject")
        }

        Message::Server(server_message) => match server_message {
            ServerMessage::Notice { .. } => {}
            ServerMessage::Refer { user, message: user_message } => {
                match user_message {
                    UserMessage::Auth { nick } => {
                        if user != nick {
                            state.messages.write().push(view::Message::notice(format!("{} changed nick to {}", user, nick)));

                            for user in state.users.write().iter_mut() {
                                if user == &nick {
                                    *user = nick.clone();
                                }
                            }
                        } else {
                            state.messages.write().push(view::Message::notice(format!("User joined: {}", nick)));

                            state.users.write().push(nick);
                        }
                    }

                    UserMessage::Leave { message } => {
                        state.messages.write().push(view::Message::notice(format!("User left: {}", user)));
                    }

                    UserMessage::RequestUserList => {}
                    UserMessage::Text { message } => {
                        state.messages.write().push(view::Message::user(user, message));
                    }
                    UserMessage::Voice { .. } => {}
                }
            }
            ServerMessage::UserList { mut users } => {
                users.insert(0, state.nick.clone());

                *state.users.write() = users;
            }
        }

        Message::Error(error_message) => match error_message {
            ErrorMessage::AlreadyConnected => {}
            ErrorMessage::NickNameInUse => {}
        }
    }
}

