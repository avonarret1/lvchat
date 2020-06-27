#![allow(unused_variables, unused_imports)]
mod config;

use std::{
    io::{Read, Write},
    net::TcpStream,
    time::Instant,
};

use tui::backend::{Backend, CrosstermBackend};
use tui::layout::{Constraint, Direction, Layout, Rect};
use tui::widgets::Paragraph;
use tui::{
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, List, Text, Widget},
    Frame, Terminal,
};

use lvchat_core::{Message as CoreMessage, UserMessage};

use crate::config::Config;
use std::process::exit;

#[derive(Debug, Clone)]
pub struct Message {
    ts: chrono::DateTime<chrono::Utc>,
    source: String,
    text: String,
}

#[derive(Debug, Clone)]
pub struct State {
    users: Vec<String>,
    messages: Vec<Message>,

    input: String,
}

impl std::default::Default for State {
    fn default() -> Self {
        Self {
            users: vec![],
            messages: vec![],

            input: String::new(),
        }
    }
}

impl State {
    pub fn users(&self) -> &Vec<String> {
        &self.users
    }

    pub fn users_mut(&mut self) -> &mut Vec<String> {
        &mut self.users
    }

    pub fn messages(&self) -> &Vec<Message> {
        &self.messages
    }

    pub fn messages_mut(&mut self) -> &mut Vec<Message> {
        &mut self.messages
    }
}

impl Message {
    pub fn new<S, T>(source: S, text: T) -> Self
    where
        S: AsRef<str>,
        T: AsRef<str>,
    {
        Self {
            ts: chrono::Utc::now(),
            source: source.as_ref().to_string(),
            text: text.as_ref().to_string(),
        }
    }
}

impl From<(&str, &str)> for Message {
    fn from(data: (&str, &str)) -> Self {
        Self::new(data.0, data.1)
    }
}

fn main() {
    let config = Config::init();
    let mut state = State::default();

    // SAMPLE
    *state.users_mut() = vec![
        format!("Ein User"),
        format!("Noch ein User"),
        format!("Ein dritter User"),
    ];

    *state.messages_mut() = vec![
        ("Ein User", "Hallo!").into(),
        ("Noch ein User", "Halt's Maul!").into(),
    ];
    // /SAMPLE

    let mut stream = connect(&config);

    init_logger(&config);
    let rx = init_input_handler();
    let mut terminal = init_terminal();

    loop {
        if let Ok(any) = rx.try_recv() {
            match any {
                b'\n' => {}

                b'\r' => {
                    state.input.clear();
                }

                // backspace
                8 => {
                    state.input.pop();
                }

                other if !other.is_ascii_control() => {
                    state.input.push(other as char);
                }

                _ => (),
            }
        }

        let _ = terminal.draw(|mut frame| render(&state, frame));
        let _ = terminal.set_cursor(
            state.input.len() as u16,
            terminal
                .size()
                .map(|size| size.height - 2)
                .unwrap_or_default(),
        );
    }
}

pub fn create_user_list_view<'a>(state: &'a State) -> impl Widget + 'a {
    List::new(state.users.iter().cloned().map(Text::raw))
        .block(Block::default().borders(Borders::ALL))
}

pub fn create_message_list_view<'a>(state: &'a State) -> impl Widget + 'a {
    List::new(
        state
            .messages
            .iter()
            .map(|message| {
                format!(
                    "<{}> {} | {}",
                    chrono_humanize::HumanTime::from(message.ts),
                    message.source,
                    message.text
                )
            })
            .map(Text::raw)
            .collect::<Vec<_>>()
            .into_iter(),
    )
    .block(Block::default().borders(Borders::ALL))
}

pub fn render<B: Backend>(state: &State, mut frame: Frame<B>) {
    let user_list = create_user_list_view(&state);
    let message_list = create_message_list_view(&state);

    let message_input_text = [Text::raw(&state.input)];
    let message_input = Paragraph::new(message_input_text.iter());

    let (bottom, top) = {
        let mut layout = Layout::default()
            .constraints([Constraint::Min(0), Constraint::Max(2)])
            .direction(Direction::Vertical)
            .split(Rect {
                x: 0,
                y: 0,
                width: frame.size().width,
                height: frame.size().height,
            });

        (layout.pop().unwrap(), layout.pop().unwrap())
    };

    let (top_left, top_right) = {
        let mut layout = Layout::default()
            .constraints([Constraint::Ratio(1, 6), Constraint::Ratio(5, 6)])
            .direction(Direction::Horizontal)
            .split(top);

        (layout.pop().unwrap(), layout.pop().unwrap())
    };

    frame.render_widget(message_list, top_left);
    frame.render_widget(user_list, top_right);
    frame.render_widget(message_input, bottom.clone());
}

fn init_terminal() -> Terminal<CrosstermBackend<std::io::Stdout>> {
    let stdout = std::io::stdout();
    let backend = tui::backend::CrosstermBackend::new(stdout);
    let mut terminal = tui::Terminal::new(backend).unwrap();
    let _ = terminal.clear();

    terminal
}

fn init_input_handler() -> flume::Receiver<u8> {
    let (tx, rx) = flume::bounded(1);

    let _input_handler = std::thread::spawn(move || loop {
        match std::io::stdin().bytes().take(1).last().unwrap() {
            Ok(any) => {
                let _ = tx.send(any);
            }
            _ => (),
        }

        std::thread::yield_now();
    });

    rx
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
    println!(
        "Trying to connect to remote host ({}:{})",
        config.host, config.port
    );

    match TcpStream::connect((config.host.as_str(), config.port)) {
        Ok(mut stream) => {
            println!("Connected.");

            let mut data = CoreMessage::User(UserMessage::Auth {
                nick: format!("avonarret"),
            })
            .to_bytes();

            data.extend_from_slice(b"\r\n");

            let _ = stream.write(&data);

            println!("Disconnected.");

            return stream;
        }

        Err(e) => {
            eprintln!("Failed to connect to remote host: {}", e);

            exit(0);
        }
    }
}
