use std::hash::{Hash, Hasher};

pub struct Note {
    pub note_id: String,
    pub note_name: String,
    pub file_name: String,
    pub creation_date: String
}

impl PartialEq for Note {
    fn eq(&self, other: &Self) -> bool {
        self.note_id == other.note_id
    }
}

impl Eq for Note {}

impl Hash for Note {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.note_id.hash(state);
    }
}