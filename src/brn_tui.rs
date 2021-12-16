use crate::notes::Notes;
use crate::message::Message;

use crossterm::{
    event::{ self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode },
    execute,
    terminal::{ disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen },
};
use std::io;
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
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture);
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend).unwrap();

        let result = BrnTui::run_app(&mut terminal);

        disable_raw_mode().unwrap();
        execute!(
            terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        );
        terminal.show_cursor().unwrap();

        if let Err(error) = result {
            println!("{:?}", error);
        }
    }

    fn run_app<B: Backend>(terminal: &mut Terminal<B>) -> io::Result<()> {
        let mut item_list_state = ListState::default();
        item_list_state.select(Some(0));

        // UI drawing loop
        loop {
            terminal.draw(|f| BrnTui::get_ui(f, &mut item_list_state)).unwrap();

            // Detect keydown events
            if let Ok(Event::Key(key)) = event::read() {
                match key.code {
                    KeyCode::Char('q') => return Ok(()),
                    KeyCode::Char('j') => BrnTui::increment_selected_value_of(&mut item_list_state, 2),
                    KeyCode::Char('k') => BrnTui::decrement_selected_value_of(&mut item_list_state, 0),
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

    fn increment_selected_value_of(list: &mut ListState, max_value: usize) {
        if let Some(selected) = list.selected() {
            if selected < max_value {
                list.select(Some(selected + 1));
            } else {
                list.select(Some(max_value))
            }
        }
    }

    fn decrement_selected_value_of(list: &mut ListState, min_value: usize) {
        if let Some(selected) = list.selected() {
            if selected > min_value {
                list.select(Some(selected - 1));
            } else {
                list.select(Some(min_value))
            }
        }
    }
}