use crate::brn_tui::tui_data::TuiData;
use crate::brn_tui::input_mode::InputMode;
use crate::database::Database;
use crate::note_property::NoteProperty;
use crate::note_type::NoteType;
use crate::notes::Notes;
use crate::settings::Settings;

use clipboard::{ ClipboardProvider, ClipboardContext };
use crossterm::{
    event::{ self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode },
    execute,
    terminal::{ disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen },
};
use std::io;
use std::io::Write;
use regex::Regex;
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

    fn run_app<B: Backend + Write>(terminal: &mut Terminal<B>, tui_data: &mut TuiData, settings: &mut Settings) -> io::Result<()> {
        BrnTui::show_note_content_preview(tui_data, settings);
        loop {
            terminal.draw(|f| BrnTui::render_ui(f, tui_data)).unwrap();

            // Detect keydown events
            if let Ok(Event::Key(key)) = event::read() {
                match tui_data.input_mode {
                    InputMode::Normal => match key.code {
                        KeyCode::Char('q') => return Ok(()),
                        KeyCode::Char('j') | KeyCode::Down
                            => BrnTui::increment_selected_value(tui_data, settings),
                        KeyCode::Char('k') | KeyCode::Up
                            => BrnTui::decrement_selected_value(tui_data, settings),
                        KeyCode::Char('l') | KeyCode::Enter
                            => BrnTui::open_selected_note(terminal, tui_data, settings),
                        KeyCode::Char('y')
                            => BrnTui::copy_selected_note_as_link(tui_data),
                        KeyCode::Char('a') => {
                            tui_data.edit_text.set_pre_text("Name: ");
                            tui_data.input_mode = InputMode::Add;
                        },
                        KeyCode::Char('d') | KeyCode::Esc => {
                            let note_list = Notes::get(100);
                            tui_data.note_list.replace_items_with(note_list);
                            tui_data.note_list.select(Some(0));
                            tui_data.note_list_title = String::from("List");
                        },
                        KeyCode::Char('h') => {
                            let note_history = Notes::get_note_history(settings);
                            tui_data.note_list.replace_items_with(note_history.iter().map(|m| m.note_name.clone()).collect());
                            tui_data.note_list.select(Some(0));
                            tui_data.note_list_title = String::from("History");
                        },
                        KeyCode::Char('r') => {
                            tui_data.edit_text.set_pre_text("Remove currently selected note [y|N]: ");
                            tui_data.input_mode = InputMode::Remove;
                        },
                        KeyCode::Char('/')
                            => tui_data.input_mode = InputMode::Search,
                        _ => (),
                    },
                    InputMode::Add => match key.code {
                        KeyCode::Esc => tui_data.input_mode = InputMode::Normal,
                        KeyCode::Enter => {
                            if tui_data.note_name_cache.is_empty() {
                                tui_data.note_name_cache = tui_data.edit_text.get_content_text();
                                tui_data.edit_text.set_pre_text("Type (T|q|j): ");
                            } else {
                                BrnTui::add_note(terminal, tui_data, settings);
                                tui_data.input_mode = InputMode::Normal;
                                tui_data.edit_text.clear();
                            }
                        }
                        KeyCode::Char(c) => {
                            tui_data.edit_text.push(c);
                        },
                        KeyCode::Backspace => {
                            tui_data.edit_text.pop();
                        },
                        _ => (),
                    },
                    InputMode::Remove => match key.code {
                        KeyCode::Esc => tui_data.input_mode = InputMode::Normal,
                        KeyCode::Enter => {
                            let confirmation_validator = Regex::new("^[Yy]$").unwrap();
                            if confirmation_validator.is_match(&tui_data.edit_text.get_content_text()) {
                                if let Some(selected_note) = tui_data.note_list.selected_item() {
                                    if let Err(error) = Notes::remove(selected_note, &settings.notes_dir) {
                                        tui_data.message = error;
                                    }
                                }
                            }
                            tui_data.input_mode = InputMode::Normal;
                        }
                        KeyCode::Char(c) => {
                            tui_data.edit_text.push(c);
                        },
                        KeyCode::Backspace => {
                            tui_data.edit_text.pop();
                        },
                        _ => (),
                    },
                    InputMode::Search => match key.code {
                        KeyCode::Esc => tui_data.input_mode = InputMode::Normal,
                        KeyCode::Enter => {
                            tui_data.input_mode = InputMode::Normal;
                            BrnTui::execute_search(tui_data, settings);
                            tui_data.note_list_title = tui_data.search_text.get_displayed_text();
                        }
                        KeyCode::Char(c) => {
                            tui_data.search_text.push(c);
                        },
                        KeyCode::Backspace => {
                            tui_data.search_text.pop();
                        },
                        _ => (),
                    },
                }
            }
        }
    }

    fn render_ui<B: Backend>(f: &mut Frame<B>, tui_data: &mut TuiData) {
        let vertical_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(
                [
                    Constraint::Min(0),
                    Constraint::Length(1),
                ].as_ref()
            )
            .split(f.size().inner(&Margin {vertical: 1, horizontal: 1}));

        let horizontal_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .margin(0)
            .constraints(
                [
                    Constraint::Percentage(35),
                    Constraint::Min(0),
                ].as_ref()
            )
            .split(vertical_chunks[0]);
        
        BrnTui::render_note_list(f, horizontal_chunks[0], tui_data);
        BrnTui::render_note_preview(f, horizontal_chunks[1], tui_data);
        BrnTui::render_message_block(f, vertical_chunks[1].inner(&Margin {vertical: 0, horizontal: 1}), tui_data);
    }

    fn render_note_list<B: Backend>(f: &mut Frame<B>, area: Rect, tui_data: &mut TuiData) {
        let selected_style = Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD);
        let normal_style = Style::default().fg(Color::White);

        // Get notes to show
        let items: Vec<ListItem> = tui_data.note_list.get_items()
            .iter()
            .map(|m| {
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
                    .title(tui_data.note_list_title.clone())
                    .borders(Borders::ALL)
            );
        f.render_stateful_widget(list, area, tui_data.note_list.get_state());
    }

    fn render_note_preview<B: Backend>(f: &mut Frame<B>, area: Rect, tui_data: &mut TuiData) {
        let title = match tui_data.note_list.selected_item() {
            Some(value) => value,
            None => "Note preview",
        };

        // Render note preview
        let outer_note_block = Block::default()
                    .title(title)
                    .borders(Borders::ALL);
        f.render_widget(outer_note_block, area);

        let inner_note_paragraph = Paragraph::new(tui_data.note_content_preview.as_str())
            .alignment(Alignment::Left);
        f.render_widget(inner_note_paragraph, area.inner(&Margin {vertical: 2, horizontal: 2}));
    }

    fn render_message_block<B: Backend>(f: &mut Frame<B>, area: Rect, tui_data: &mut TuiData) {
        let message_paragraph;
        match tui_data.input_mode {
            InputMode::Normal => {
                message_paragraph = Paragraph::new(tui_data.message.as_str())
                    .alignment(Alignment::Left)
                    .style(Style::default().fg(Color::LightRed));
            },
            InputMode::Add | InputMode::Remove => {
                message_paragraph = Paragraph::new(tui_data.edit_text.get_displayed_text())
                    .alignment(Alignment::Left)
                    .style(Style::default());
            },
            InputMode::Search => {
                message_paragraph = Paragraph::new(tui_data.search_text.get_displayed_text())
                    .alignment(Alignment::Left)
                    .style(Style::default());
            },
        }
        f.render_widget(message_paragraph, area);
    }

    fn increment_selected_value(tui_data: &mut TuiData, settings: &mut Settings) {
        tui_data.note_list.next();
        BrnTui::show_note_content_preview(tui_data, settings);
    }

    fn decrement_selected_value(tui_data: &mut TuiData, settings: &mut Settings) {
        tui_data.note_list.previous();
        BrnTui::show_note_content_preview(tui_data, settings);
    }

    fn show_note_content_preview(tui_data: &mut TuiData, settings: &mut Settings) {
        if let Some(selected_note_name) = &tui_data.note_list.selected_item() {
            if let Some(note_id) = Database::get_note_id_where(NoteProperty::NoteName, selected_note_name) {
                match Notes::get_content_of_note(&note_id, settings) {
                    Ok(note_content) => {
                        tui_data.note_content_preview = note_content;
                    }
                    Err(error) => {
                        tui_data.message = error;
                    }
                }
            }
        }
    }

    fn open_selected_note<B: Backend + Write>(terminal: &mut Terminal<B>, tui_data: &mut TuiData, settings: &mut Settings) {
        if let Some(selected_note_name) = tui_data.note_list.selected_item() {
            if let Some(note_id) = Database::get_note_id_where(NoteProperty::NoteName, selected_note_name) {
                BrnTui::open_note(&note_id, terminal, tui_data, settings);
            };
        };
    }

    fn copy_selected_note_as_link(tui_data: &mut TuiData) {
        let mut clipboard_context: ClipboardContext = ClipboardProvider::new().unwrap();
        if let Some(selected_note_name) = tui_data.note_list.selected_item() {
            if let Some(note_id) = Database::get_note_id_where(NoteProperty::NoteName, selected_note_name) {
                if let Err(boxed_error) = clipboard_context.set_contents(format!("[[{}]] ", note_id)) {
                    tui_data.message = (*boxed_error).to_string();
                }
            };
        };
    }

    fn execute_search(tui_data: &mut TuiData, settings: &mut Settings) {
        let search_results = Notes::search(&tui_data.search_text.get_content_text())
            .iter()
            .map(|m| {
                match Database::get_note_where_id(&m.note_id) {
                    Some(note) => note.note_name,
                    None => String::new(),
                }
            })
            .collect();
        tui_data.note_list.replace_items_with(search_results);
        tui_data.note_list.select(Some(0));
        BrnTui::show_note_content_preview(tui_data, settings);
    }

    fn add_note<B: Backend + Write>(terminal: &mut Terminal<B>, tui_data: &mut TuiData, settings: &mut Settings) {
        let note_type = match tui_data.edit_text.get_content_text().as_str() {
            "Q" | "q" => NoteType::Quote,
            "J" | "j" => NoteType::Journal,
            "T" | "t" | _ => NoteType::Topic,
        };

        match Notes::add(tui_data.note_name_cache.as_str(), note_type, settings) {
            Ok(None) => (),
            Ok(Some(note_id)) => {
                BrnTui::open_note(&note_id, terminal, tui_data, settings);
            },
            Err(error) => tui_data.message = "ERROR: ".to_string() + &error,
        };
    }

    fn open_note<B: Backend + Write>(note_id: &str, terminal: &mut Terminal<B>, tui_data: &mut TuiData, settings: &mut Settings) {
        execute!(
            terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        ).unwrap();

        match Notes::open(&note_id, settings) {
            Ok(None) => (),
            Ok(Some(message)) => tui_data.message = "INFO: ".to_string() + &message,
            Err(message) => tui_data.message = "ERROR: ".to_string() + &message,
        }

        // Force full redraw in the terminal
        terminal.clear().unwrap();

        execute!(
            terminal.backend_mut(),
            EnterAlternateScreen,
            EnableMouseCapture
        ).unwrap();
    }
}