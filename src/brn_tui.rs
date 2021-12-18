use crate::database::Database;
use crate::note_property::NoteProperty;
use crate::notes::Notes;
use crate::settings::Settings;
use crate::tui_data::TuiData;

use crossterm::{
    event::{ self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode },
    execute,
    terminal::{ disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen },
};
use std::io;
use tui::{
    backend::{ Backend, CrosstermBackend },
    layout::{ Alignment, Constraint, Direction, Layout, Margin, Rect },
    style::{ Color, Modifier, Style },
    widgets::{
        Block, Borders, List, ListItem, Paragraph,
    },
    Frame, Terminal,
};

pub struct BrnTui;

impl BrnTui {
    pub fn init(settings: &mut Settings) {
        enable_raw_mode().unwrap();
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture).unwrap();
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend).unwrap();

        let mut tui_data = TuiData::default();
        let result = BrnTui::run_app(&mut terminal, &mut tui_data, settings);

        disable_raw_mode().unwrap();
        execute!(
            terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        ).unwrap();
        terminal.show_cursor().unwrap();

        if let Err(error) = result {
            println!("{:?}", error);
        }
    }

    fn run_app<B: Backend>(terminal: &mut Terminal<B>, tui_data: &mut TuiData, settings: &mut Settings) -> io::Result<()> {
        loop {
            terminal.draw(|f| BrnTui::render_ui(f, tui_data)).unwrap();

            // Detect keydown events
            if let Ok(Event::Key(key)) = event::read() {
                match key.code {
                    KeyCode::Char('q') => return Ok(()),
                    KeyCode::Char('j') => BrnTui::increment_selected_value(tui_data.note_list_data.len() - 1, tui_data, settings),
                    KeyCode::Char('k') => BrnTui::decrement_selected_value(0, tui_data, settings),
                    _ => (),
                }
            }
        }
    }

    fn render_ui<B: Backend>(f: &mut Frame<B>, tui_data: &mut TuiData) {
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
        
        BrnTui::render_note_list(f, chunks[0], tui_data);
        BrnTui::render_note_preview(f, chunks[1], tui_data);
    }

    fn render_note_list<B: Backend>(f: &mut Frame<B>, area: Rect, tui_data: &mut TuiData) {
        let selected_style = Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD);
        let normal_style = Style::default().fg(Color::White);

        // Get notes to show
        tui_data.note_list_data = Notes::get(100);
        let items: Vec<ListItem> = tui_data.note_list_data
            .iter()
            .enumerate()
            .map(|(_, m)| {
                ListItem::new(m.to_string())
            })
            .collect();

        // Render note list
        let list = List::new(items)
            .style(normal_style)
            .highlight_style(selected_style)
            .highlight_symbol("> ")
            .block(
                Block::default()
                    .title("List")
                    .borders(Borders::ALL)
            );
        f.render_stateful_widget(list, area, &mut tui_data.note_list);
    }

    fn render_note_preview<B: Backend>(f: &mut Frame<B>, area: Rect, tui_data: &mut TuiData) {
        // Render note preview
        let outer_note_block = Block::default()
                    .title("Note")
                    .borders(Borders::ALL);
        f.render_widget(outer_note_block, area);

        let inner_note_paragraph = Paragraph::new(tui_data.note_content_preview.as_str())
            .alignment(Alignment::Left);
        f.render_widget(inner_note_paragraph, area.inner(&Margin {vertical: 2, horizontal: 2}));
    }

    fn increment_selected_value(max_value: usize, tui_data: &mut TuiData, settings: &mut Settings) {
        if let Some(selected) = tui_data.note_list.selected() {
            if selected < max_value {
                tui_data.note_list.select(Some(selected + 1));
            } else {
                tui_data.note_list.select(Some(max_value))
            }
            BrnTui::show_note_content_preview(tui_data, settings);
        }
    }

    fn decrement_selected_value(min_value: usize, tui_data: &mut TuiData, settings: &mut Settings) {
        if let Some(selected) = tui_data.note_list.selected() {
            if selected > min_value {
                tui_data.note_list.select(Some(selected - 1));
            } else {
                tui_data.note_list.select(Some(min_value))
            }
            BrnTui::show_note_content_preview(tui_data, settings);
        }
    }

    fn show_note_content_preview(tui_data: &mut TuiData, settings: &mut Settings) {
        let note_name = &tui_data.note_list_data[tui_data.note_list.selected().unwrap()];
        if let Some(note_id) = Database::get_note_id_where(NoteProperty::NoteName, note_name) {
            if let Some(note_content) = Notes::get_content_of_note(&note_id, settings) {
                tui_data.note_content_preview = note_content;
            }
        }
    }
}