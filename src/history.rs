use std::collections::VecDeque;
use std::ffi::OsStr;
use std::fs::File;
use std::io::{Read, Write};
use std::path::PathBuf;

pub struct History {
    pub note_history_capacity: usize,
    note_history: VecDeque<String>,
    history_file_path: PathBuf,
}

impl History {
    pub fn new() -> History {
        History {
            note_history: VecDeque::new(),
            note_history_capacity: 100,
            history_file_path: PathBuf::default(),
        }
    }

    pub fn init(&mut self, file_path: &OsStr) -> Result<(), String> {
        self.history_file_path = PathBuf::from(file_path).join("history");
        self.load()
    }

    pub fn add(&mut self, note_id: &str) -> Result<(), String> {
        if self.note_history.contains(&note_id.to_string()) {
            self.note_history.retain(|x| x != &note_id.to_string());
        } else if self.note_history.len() >= self.note_history_capacity {
            self.note_history.pop_back();
        }

        self.note_history.push_front(note_id.to_string());

        self.save()
    }

    fn save(&self) -> Result<(), String> {
        let mut history_file = match self.create_history_file() {
            Ok(value) => value,
            Err(error) => return Err(error),
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

            if let Err(error) = history_file.write(history_entry.as_bytes()) {
                return Err(format!(
                    "save_note_history: couldn't write note id {} to history file: {}",
                    note_id, error
                ));
            }
        }

        Ok(())
    }

    fn load(&mut self) -> Result<(), String> {
        let mut history_file = match File::open(&self.history_file_path) {
            Ok(value) => value,
            Err(_) => {
                return match self.create_history_file() {
                    Ok(_) => Ok(()),
                    Err(error) => Err(error),
                }
            }
        };

        let mut note_history_string = String::new();
        if let Err(error) = history_file.read_to_string(&mut note_history_string) {
            return Err(format!(
                "load_note_history: couldn't load note history: {}",
                error
            ));
        }

        self.note_history.clear();
        for note_id in note_history_string.split('\n') {
            self.note_history.push_back(note_id.to_string());
        }

        Ok(())
    }

    fn create_history_file(&self) -> Result<File, String> {
        let history_file = match File::create(&self.history_file_path) {
            Ok(value) => value,
            Err(error) => {
                return Err(format!(
                    "save_note_history: couldn't access history file at '{}': {}",
                    self.history_file_path.to_string_lossy(),
                    error
                ));
            }
        };

        Ok(history_file)
    }

    pub fn list(&self) -> Vec<String> {
        Vec::from(self.note_history.clone())
    }
}
