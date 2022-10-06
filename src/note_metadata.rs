use crate::file_utility::FileUtility;
use crate::note::Note;
use crate::note_property::NoteProperty;
use crate::settings::Settings;

use chrono::{Local, TimeZone};
use lazy_static::lazy_static;
use regex::Regex;
use std::io::{Error, ErrorKind};
use std::path::{Path, PathBuf};
use yaml_rust::{Yaml, YamlLoader};

lazy_static! {
    static ref NOTE_FORMAT_VALIDATOR: Regex = Regex::new(
        r##"(?xs)
        (                                                               # $1 = yaml header
            ^
            \s*
            ---[ \t]*
            \n*
            (.*?)     # $2 = yaml text
            \n*
            ---[ \t]*
            \n?
        )
        (.*)          # $3 = body of the note
        $
    "##
    )
    .unwrap();
}

pub struct NoteMetadata;

impl NoteMetadata {
    pub fn get_basic_data_of_file<P: AsRef<Path>>(file_path: P) -> Result<Note, String> {
        let note_metadata = match NoteMetadata::get_metadata_of_file(&file_path) {
            Ok(value) => value,
            Err(error) => {
                return Err(error);
            }
        };

        let mut note = Note::default();

        if let Some(note_id) =
            note_metadata[NoteProperty::NoteId.to_metadata_identifier().as_str()].as_str()
        {
            note.note_id = note_id.to_string();
        } else {
            return Err(format!("get-note-metadata: The note name couldn't be found in the metadata of the file '{}'!",
                &file_path.as_ref().to_string_lossy()));
        };

        if let Some(note_name) =
            note_metadata[NoteProperty::NoteName.to_metadata_identifier().as_str()].as_str()
        {
            note.note_name = note_name.to_string();
        } else {
            return Err(format!("get-note-metadata: The note name of couldn't be found in the metadata of the file '{}'!",
                &file_path.as_ref().to_string_lossy()));
        };

        if let Some(creation_timestamp) =
            note_metadata[NoteProperty::CreationDate.to_metadata_identifier().as_str()].as_str()
        {
            if let Ok(creation_date_time) =
                Local.datetime_from_str(&creation_timestamp, "%Y-%m-%d %H:%M:%S")
            {
                note.creation_date_time = Some(creation_date_time);
            } else {
                return Err(format!("get-note-metadata: The creation date in the metadata of the file '{}' has the wrong format!",
                    &file_path.as_ref().to_string_lossy()));
            }
        } else {
            return Err(format!("get-note-metadata: The creation date couldn't be found in the metadata of the file '{}'!",
                &file_path.as_ref().to_string_lossy()));
        };

        if let Some(os_file_name) = &file_path.as_ref().file_name() {
            if let Some(file_name) = os_file_name.to_str() {
                note.file_name = file_name.to_string();
            } else {
                return Err(format!(
                    "get-note-metadata: The file name '{}' contains illegal characters!",
                    &file_path.as_ref().to_string_lossy()
                ));
            };
        } else {
            return Err(format!(
                "get-note-metadata: Cannot get filename from the path '{}'!",
                &file_path.as_ref().to_string_lossy()
            ));
        };

        return Ok(note);
    }

    pub fn get_property_of(
        note: &Note,
        property: NoteProperty,
        settings: &Settings,
    ) -> Result<Option<String>, String> {
        let notes_dir = &settings.notes_dir;
        let absolute_note_file_path = PathBuf::from(notes_dir).join(&note.file_name);
        return NoteMetadata::get_property_of_file(&absolute_note_file_path, property);
    }

    pub fn get_tags_of(note: &Note, settings: &Settings) -> Result<Option<Vec<String>>, String> {
        let notes_dir = &settings.notes_dir;
        let absolute_note_file_path = PathBuf::from(notes_dir).join(&note.file_name);
        return NoteMetadata::get_tags_of_file(&absolute_note_file_path);
    }

    pub fn get_property_of_file<P: AsRef<Path>>(
        file_path: P,
        property: NoteProperty,
    ) -> Result<Option<String>, String> {
        let note_metadata = match NoteMetadata::get_metadata_of_file(file_path) {
            Ok(value) => value,
            Err(error) => {
                return Err(error);
            }
        };
        let property_identifier = property.to_metadata_identifier();

        let note_name = match note_metadata[property_identifier.as_str()].as_str() {
            Some(value) => Some(value.to_string()),
            None => None,
        };

        return Ok(note_name);
    }

    pub fn get_tags_of_file<P: AsRef<Path>>(file_path: P) -> Result<Option<Vec<String>>, String> {
        let note_metadata = match NoteMetadata::get_metadata_of_file(file_path) {
            Ok(value) => value,
            Err(error) => return Err(error),
        };

        let tags: Option<Vec<String>> = match note_metadata["tags"].as_vec() {
            None => None,
            Some(value) => Some(
                value
                    .iter()
                    .map(|m| m.as_str().unwrap().to_string())
                    .collect(),
            ),
        };
        return Ok(tags);
    }

    fn get_metadata_of_file<P: AsRef<Path>>(file_path: P) -> Result<Yaml, String> {
        let yaml_string = match NoteMetadata::get_yaml_header_of_file(&file_path) {
            Ok(header) => header,
            Err(error) => {
                return Err(format!(
                    "get-metadata: couldn't read header of note file '{}': {}",
                    &file_path.as_ref().to_string_lossy(),
                    error
                ));
            }
        };
        let note_metadata = match YamlLoader::load_from_str(&yaml_string) {
            Ok(yaml_vector) => yaml_vector[0].clone(),
            Err(error) => {
                return Err(format!(
                    "couldn't parse yaml header of file '{}': '{}'",
                    &file_path.as_ref().to_string_lossy(),
                    error
                ));
            }
        };

        return Ok(note_metadata);
    }

    fn get_yaml_header_of_file<P: AsRef<Path>>(file_path: P) -> Result<String, Error> {
        let note_content = match FileUtility::get_content_from_file(file_path) {
            Ok(file_content) => file_content,
            Err(error) => return Err(error),
        };

        match NOTE_FORMAT_VALIDATOR.captures(&note_content) {
            Some(note_content_match) => {
                let yaml_header = note_content_match.get(2).unwrap().as_str();
                return Ok(yaml_header.to_string());
            }
            None => {
                return Err(Error::new(ErrorKind::NotFound, "yaml header not found"));
            }
        }
    }
}
