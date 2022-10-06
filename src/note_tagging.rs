use std::hash::{Hash, Hasher};

#[derive(Clone)]
pub struct NoteTagging {
    pub note_id: String,
    pub tag_name: Option<String>,
}

impl NoteTagging {
    pub fn from(note_id: String, tag_name: Option<String>) -> Self {
        NoteTagging {
            note_id,
            tag_name,
        }
    }
}

impl PartialEq for NoteTagging {
    fn eq(&self, other: &Self) -> bool {
        self.note_id == other.note_id
    }
}

impl Eq for NoteTagging {}

impl Hash for NoteTagging {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.note_id.hash(state);
    }
}
