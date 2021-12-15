use crate::notes::Notes;
use crate::message::Message;

use crossterm::{
    event::{ self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode },
    execute,
    terminal::{ disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen },
};
use std::io;
use std::time::{ Duration, Instant };
use tui::{
    backend::{ Backend, CrosstermBackend },
    layout::{ Alignment, Constraint, Direction, Layout },
    style::{ Color, Modifier, Style },
    text::{ Span, Spans, Text },
    widgets::{
        Block, BorderType, Borders, Cell, List, ListItem, ListState, Paragraph, Row, Table, Tabs,
    },
    Frame, Terminal,
};

pub struct BrnTui;

impl BrnTui {
    pub fn init() {
        enable_raw_mode().unwrap();
        let stdout = io::stdout();
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal.hide_cursor().unwrap();
        terminal.clear().unwrap();

        BrnTui::run_app(&mut terminal);
        disable_raw_mode();
        execute!(
            terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        );
        terminal.show_cursor();

    }

    fn run_app<B: Backend>(terminal: &mut Terminal<B>) {
        let mut item_list_state = ListState::default();
        item_list_state.select(Some(0));

        // UI drawing loop
        loop {
            terminal.draw(|f| BrnTui::get_ui(f, &mut item_list_state)).unwrap();

            // Detect keydown events
            if let Ok(Event::Key(key)) = event::read() {
                match key.code {
                    KeyCode::Char('q') => return,
                    KeyCode::Char('j') => {
                        if let Some(selected) = item_list_state.selected() {
                            if selected < 2 {
                                item_list_state.select(Some(selected + 1));
                            } else {
                                item_list_state.select(Some(2))
                            }
                        }
                    },
                    KeyCode::Char('k') => {
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
        }
    }

    fn get_ui<B: Backend>(f: &mut Frame<B>, item_list_state: &mut ListState) {
        let selected_style = Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD);
        let normal_style = Style::default().fg(Color::White);

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
        f.render_stateful_widget(list, chunks[0], item_list_state);

        // Render paragraph
        let para = Paragraph::new("hello world\nhello".to_string() + &item_list_state.selected().unwrap().to_string())
            .alignment(Alignment::Center)
            .block(
                Block::default()
                    .title("Note")
                    .borders(Borders::ALL)
            );
        f.render_widget(para, chunks[1]);
    }
}