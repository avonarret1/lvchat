mod message;

use std::io::{stdout, Stdout};

use tui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    terminal::Terminal,
    widgets::{Block, Borders, List, Paragraph, Text, Widget},
};

use crate::state::State;

pub use self::message::Message;

pub type User = String;

pub struct View {
    terminal: Terminal<CrosstermBackend<Stdout>>,
}

impl View {
    pub fn update(&mut self, state: &State) {
        let _ = self.terminal.set_cursor(
            state.input.read().len() as u16,
            self.terminal
                .size()
                .map(|size| size.height - 2)
                .unwrap_or_default(),
        );

        self.terminal.clear();
    }

    pub fn render(&mut self, state: &State) {
        let user_list_items = state.users.read().iter().cloned().collect::<Vec<_>>();
        let user_list_view = List::new(user_list_items.iter().map(Text::raw));

        let message_list_items = state.messages.read().iter().cloned().collect::<Vec<_>>();
        let message_list_view = List::new(
            message_list_items
                .iter()
                .map(|message| {
                    format!(
                        "<{} [{}]> {}",
                        message.source,
                        chrono_humanize::HumanTime::from(message.ts),
                        message.text,
                    )
                })
                .map(Text::raw),
        )
        .block(Block::default().borders(Borders::LEFT));

        let message_input = state.input.read().clone();
        let message_para_input = [Text::raw(message_input)];
        let message_input_view =
            Paragraph::new(message_para_input.iter()).block(Block::default().borders(Borders::TOP));

        let _ = self.terminal.draw(|mut frame| {
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

            frame.render_widget(user_list_view, top_left);
            frame.render_widget(message_list_view, top_right);
            frame.render_widget(message_input_view, bottom.clone());
        });
    }
}

impl Default for View {
    fn default() -> Self {
        let stdout = stdout();
        let backend = CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend).unwrap();

        View { terminal }
    }
}
/*
fn create_user_list_view<'a>(state: &'a State) -> impl Widget + 'a {
    List::new(state.users.read().iter().cloned().map(Text::raw))
        .block(Block::default().borders(Borders::ALL))
}

fn create_message_list_view<'a>(state: &'a State) -> impl Widget + 'a {
    List::new(
        state
            .messages
            .read()
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
}*/
