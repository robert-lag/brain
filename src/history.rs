use crate::message::Message;

use std::collections::VecDeque;
use std::fs::{ self, File };
use std::io::{ Write, Read };
use std::path::PathBuf;

pub struct History {
    pub note_history_capacity: usize,
    note_history: VecDeque<String>,
    history_file_path: PathBuf,
}

impl History {
    pub fn new() -> History {
        return History {
            note_history: VecDeque::new(),
            note_history_capacity: 30,
            history_file_path: PathBuf::default(),
        };
    }

    pub fn init(&mut self, file_path: PathBuf) {
        self.history_file_path = file_path;
        self.load();
    }

    pub fn add(&mut self, note_id: &str) {
        self.note_history.push_front(note_id.to_string());
        if self.note_history.len() > self.note_history_capacity {
            self.note_history.pop_back();
        }
        self.save();
    }

    fn save(&self) {
        let history_file_path_prefix = self.history_file_path.parent().unwrap();
        if let Err(error) = fs::create_dir_all(history_file_path_prefix) {
            Message::error(&format!("save_note_history: couldn't create file path '{}': {}",
                history_file_path_prefix.to_string_lossy(),
                error));
            return;
        }

        let mut history_file = match File::create(&self.history_file_path) {
            Ok(value) => value,
            Err(error) => {
                Message::error(&format!("save_note_history: couldn't access history file at '{}': {}",
                    self.history_file_path.to_string_lossy(),
                    error));
                return;
            }
        };

        let mut is_first_entry = true;
        for note_id in self.note_history.iter() {
            let history_entry; 
            if is_first_entry {
                history_entry = note_id.to_string();
                is_first_entry = false;
            } else {
                history_entry = format!("\n{}", note_id);
            }

            if let Err(error) = history_file.write(&history_entry.as_bytes()) {
                Message::warning(&format!("save_note_history: couldn't write note id {} to history file: {}",
                    note_id,
                    error));
            }
        }
    }

    fn load(&mut self) {
        let history_file_path_prefix = self.history_file_path.parent().unwrap();
        if let Err(error) = fs::create_dir_all(history_file_path_prefix) {
            Message::error(&format!("save_note_history: couldn't create file path '{}': {}",
                history_file_path_prefix.to_string_lossy(),
                error));
            return;
        }

        let mut history_file = match File::open(&self.history_file_path) {
            Ok(value) => value,
            Err(error) => {
                Message::warning(&format!("load_note_history: couldn't access history file at '{}': {}",
                    self.history_file_path.to_string_lossy(),
                    error));
                return;
            }
        };

        let mut note_history_string = String::new();
        if let Err(error) = history_file.read_to_string(&mut note_history_string) {
            Message::warning(&format!("load_note_history: couldn't load note history: {}",
                error));
        }

        for note_id in note_history_string.split('\n') {
            self.note_history.push_back(note_id.to_string());
        }
    }

    pub fn list(&self) -> Vec<String> {
        return Vec::from(self.note_history.clone());
    }
}