use chrono::{DateTime, Local};

pub struct Note {
    pub note_id: String,
    pub note_name: String,
    pub file_name: String,
    pub creation_date_time: Option<DateTime<Local>>,
}

impl Default for Note {
    fn default() -> Self {
        Note {
            note_id: String::default(),
            note_name: String::default(),
            file_name: String::default(),
            creation_date_time: Some(Local::now()),
        }
    }
}

impl Note {
    pub fn new(
        note_id: String,
        note_name: String,
        file_name: String,
        creation_date_time: DateTime<Local>,
    ) -> Self {
        Note {
            note_id,
            note_name,
            file_name,
            creation_date_time: Some(creation_date_time),
        }
    }
}
