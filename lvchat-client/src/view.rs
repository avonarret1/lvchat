use std::io::{stdout, Stdout};

use tui::{
    layout::{Constraint, Direction, Layout, Rect},
    terminal::Terminal,
    widgets::{Block, Borders, List, Paragraph, Text},
};

pub use crate::message::Message;
use crate::state::State;

pub type User = String;

pub struct View {
    //#[cfg(target_os = "windows")]
    terminal: Terminal<tui::backend::CrosstermBackend<Stdout>>,
    //#[cfg(not(target_os = "windows"))]
    //terminal: Terminal<tui::backend::TermionBackend<Stdout>>,
}

impl View {
    pub fn clear(&mut self) {
        let _ = self.terminal.clear();
    }

    pub fn update(&mut self, state: &State) {}

    pub fn render(&mut self, state: &State) {
        let user_list_items = state.users.read().iter().cloned().collect::<Vec<_>>();
        let user_list_view = List::new(user_list_items.iter().map(Text::raw));

        let message_list_items = state.messages.read().iter().cloned().collect::<Vec<_>>();
        let message_list_view = List::new(
            message_list_items
                .iter()
                .map(ToString::to_string)
                .map(Text::raw),
        )
        .block(Block::default().borders(Borders::LEFT));

        let message_input = state.input.read().clone();
        let message_para_input = [Text::raw(message_input)];
        let message_input_view =
            Paragraph::new(message_para_input.iter()).block(Block::default().borders(Borders::TOP));

        let _ = self.terminal.draw(move |mut frame| {
            let (bottom, top) = {
                let mut layout = Layout::default()
                    .constraints([Constraint::Min(0), Constraint::Max(3)])
                    .direction(Direction::Vertical)
                    .split(Rect {
                        x: 0,
                        y: 0,
                        width: frame.size().width,
                        height: frame.size().height,
                    });

                (layout.pop().unwrap(), layout.pop().unwrap())
            };

            let (top_right, top_left) = {
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

        let _ = self.terminal.set_cursor(
            state.input.read().len() as u16,
            self.terminal
                .size()
                .map(|size| size.height - 2)
                .unwrap_or_default(),
        );
    }
}

impl Default for View {
    fn default() -> Self {
        let backend = {
            let stdout = stdout();

            //#[cfg(target_os = "windows")]
            //{
            tui::backend::CrosstermBackend::new(stdout)
            //}
            //#[cfg(not(target_os = "windows"))]
            //{
            //    tui::backend::TermionBackend::new(stdout)
            //}
        };

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
