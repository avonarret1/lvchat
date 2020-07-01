#![allow(unused_variables, unused_imports)]

mod config;
mod event;
mod input;
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
use crate::{config::Config, state::State, view::View, event::Event, input::user::State as UserInputState};

fn main() {
    let config = Config::new();

    init_logger(&config);

    let stream = connect(&config);

    let state = State::new(stream);
    let mut view = View::default();

    let events = &[
        input::user::capture(),
        input::server::capture(state.stream.clone())
    ];

    loop {
        for event_rx in events {
            match event_rx.try_recv() {
                Ok(event) => match event {
                    Event::UserInput(input) => {
                        handle_user_input(&state, input);
                    }
                    Event::Message(message) => {
                        handle_server_message(&state, message)
                    },
                    Event::Disconnected => {
                        break
                    }
                }

                _ => (),
            }
        }

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

            let mut data = Message::User(UserMessage::Auth {
                nick: format!("avonarret"),
            })
            .to_bytes();

            data.extend_from_slice(b"\r\n");

            let _ = stream.write(&data);

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

            let message: Message = UserMessage::Text {
                message: input,
            }.into();

            state.stream.lock().write(&message.to_bytes());
        }
    }
}

fn handle_server_message(state: &State, message: Message) {
    use lvchat_core::*;

    match message {
        Message::User(user_message) => match user_message {
            UserMessage::Auth { .. } => {}
            UserMessage::Leave { .. } => {}
            UserMessage::RequestUserList => {}
            UserMessage::Text { .. } => {}
            UserMessage::Voice { .. } => {}
        },
        Message::Server(server_message) => match server_message {
            ServerMessage::Notice { .. } => {}
            ServerMessage::Refer { .. } => {}
            ServerMessage::UserList { .. } => {}
        },
        Message::Error(error_message) => match error_message {
            ErrorMessage::AlreadyConnected => {}
            ErrorMessage::NickNameInUse => {}
        },
    }
}

