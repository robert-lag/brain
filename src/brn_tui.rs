use crate::notes::Notes;
use crate::message::Message;

use std::io;
use std::time::{ Duration, Instant };
use termion::raw::IntoRawMode;
use termion::event::{ Key };
use termion::input::{ MouseTerminal, TermRead };
use tui::Terminal;
use tui::backend::TermionBackend;
use tui::layout::{ Layout, Constraint, Direction, Alignment };
use tui::style::{ Color, Modifier, Style };
use tui::widgets::{ List, ListItem, ListState, Block, Borders, Paragraph };

pub struct BrnTui;

impl BrnTui {
    pub fn init() {
        let stdin = termion::async_stdin();
        let mut keys_stdin = stdin.keys();
        let raw_stdout = io::stdout().into_raw_mode().unwrap();
        let mouse_term_stdout = MouseTerminal::from(raw_stdout);
        let backend = TermionBackend::new(mouse_term_stdout);
        let mut terminal = Terminal::new(backend).unwrap();

        let tick_rate = Duration::from_millis(250);

        terminal.hide_cursor().unwrap();
        terminal.clear().unwrap();

        let selected_style = Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD);
        let normal_style = Style::default().fg(Color::White);

        let mut item_list_state = ListState::default();
        item_list_state.select(Some(0));

        // UI drawing loop
        loop {
            terminal.draw(|f| {
                let chunks = Layout::default()
                    .direction(Direction::Horizontal)
                    .margin(1)
                    .constraints(
                        [
                            Constraint::Percentage(20),
                            Constraint::Min(0)
                        ].as_ref()
                    )
                    .split(f.size());

                // Render item list
                let items = [ListItem::new("Item 1"), ListItem::new("Item 2"), ListItem::new("Item 3")];
                //let items = Notes::list(100);
                let list = List::new(items)
                    .highlight_style(selected_style)
                    .highlight_symbol("> ")
                    .block(
                        Block::default()
                            .title("List")
                            .borders(Borders::ALL)
                    );
                // f.render_widget(list, chunks[0]);
                f.render_stateful_widget(list, chunks[0], &mut item_list_state);

                // Render paragraph
                let para = Paragraph::new("hello world\nhello".to_string() + &item_list_state.selected().unwrap().to_string())
                    .alignment(Alignment::Center)
                    .block(
                        Block::default()
                            .title("Note")
                            .borders(Borders::ALL)
                    );
                f.render_widget(para, chunks[1]);
            }).unwrap();

            // Detect keydown events
            for c in keys_stdin.next() {
                match c.unwrap() {
                    Key::Char('q') => return,
                    Key::Char('j') => {
                        if let Some(selected) = item_list_state.selected() {
                            if selected < 2 {
                                item_list_state.select(Some(selected + 1));
                            } else {
                                item_list_state.select(Some(2))
                            }
                        }
                    },
                    Key::Char('k') => {
                        if let Some(selected) = item_list_state.selected() {
                            if selected > 0 {
                                item_list_state.select(Some(selected - 1));
                            } else {
                                item_list_state.select(Some(0))
                            }
                        }
                    },
                    _ => (),
                }
            }

            // Apply tick rate to reduce cpu usage

        }
    }
}