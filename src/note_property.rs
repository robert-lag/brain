use std::fmt;

#[derive(Debug)]
pub enum NoteProperty {
    NoteId,
    NoteName,
    CreationDate,
    // FileName,
}

impl NoteProperty {
    pub fn to_db_string(&self) -> String {
        match self {
            NoteProperty::NoteId => "note_id".to_string(),
            NoteProperty::NoteName => "note_name".to_string(),
            NoteProperty::CreationDate => "creation_date".to_string(),
            // NoteProperty::FileName => "file_name".to_string(),
        }
    }

    pub fn to_metadata_identifier(&self) -> String {
        match self {
            NoteProperty::NoteId => "id".to_string(),
            NoteProperty::NoteName => "name".to_string(),
            NoteProperty::CreationDate => "date".to_string(),
            // NoteProperty::FileName => "".to_string(),
        }
    }
}

impl fmt::Display for NoteProperty {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}
