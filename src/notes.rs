use crate::collection_tool::CollectionTool;
use crate::database::Database;
use crate::message::Message;
use crate::note_property::NoteProperty;
use crate::note_tagging::NoteTagging;
use crate::note_type::NoteType;
use crate::note::Note;
use crate::settings::Settings;

use chrono::prelude::*;
use colored::*;
use regex::Regex;
use std::collections::hash_map::RandomState;
use std::collections::hash_set::Difference;
use std::collections::HashSet;
use std::env;
use std::ffi::OsStr;
use std::fs::{ self, File };
use std::io::{ self, Error, Read, Write };
use std::path::{ Path, PathBuf };
use std::process::Command;
use yaml_rust::{ YamlLoader, Yaml };

pub struct Notes;
impl Notes {
    pub fn list(count: i32) {
        let note_ids = Database::get_all_recent_note_ids(count);

        for note_id in note_ids {
            let note = Database::get_note_where_id(&note_id).unwrap();
            println!("{} {}", note_id.yellow(), note.note_name);
        }
    }

    pub fn search(complete_search_string: &str) {
        let split_search_strings = complete_search_string.split(" && ");
        let mut search_results: HashSet<NoteTagging> = HashSet::new();
        let mut negated_search_results: HashSet<NoteTagging> = HashSet::new();

        for search_string in split_search_strings {
            search_notes_and_add_to(search_string, &mut search_results, &mut negated_search_results);
        }

        let search_results = search_results.difference(&negated_search_results);

        display_search_results_of(search_results);

        fn search_notes_and_add_to(mut search_string: &str, search_results: &mut HashSet<NoteTagging>, negated_search_results: &mut HashSet<NoteTagging>) {
            let mut individual_search_results = HashSet::new();
            let mut is_search_string_for_tag = false;
            let mut is_negated_search_string = false;

            loop {
                if &search_string[0..1] == "!" {
                    is_negated_search_string = true;
                    search_string = &search_string[1..search_string.len()];
                } else if &search_string[0..1] == "#" {
                    is_search_string_for_tag = true;
                    search_string = &search_string[1..search_string.len()];
                } else {
                    break;
                }
            }

            // Get individual search results
            if !is_search_string_for_tag {
                search_note_names(&mut individual_search_results, search_string);
            }
            search_tags(&mut individual_search_results, search_string);

            // Intersect with already existing search results
            if is_negated_search_string {
                if negated_search_results.is_empty() {
                    *negated_search_results = individual_search_results;
                }
                else {
                    *negated_search_results = CollectionTool::intersect(negated_search_results, &mut individual_search_results);
                }
            } else {
                if search_results.is_empty() {
                    *search_results = individual_search_results;
                }
                else {
                    *search_results = CollectionTool::intersect(search_results, &mut individual_search_results);
                }
            }

            fn search_note_names(individual_search_results: &mut HashSet<NoteTagging>, individual_search_string: &str) {
                let search_string_with_wildcards = format!("%{}%", individual_search_string);
                let note_ids = Database::get_note_ids_where_property_is_like(NoteProperty::NoteName, &search_string_with_wildcards);
                for note_id in note_ids {
                    individual_search_results.insert(NoteTagging::from(note_id, None));
                }
            }

            fn search_tags(individual_search_results: &mut HashSet<NoteTagging>, individual_search_string: &str) {
                let search_string_with_wildcards = format!("%{}%", individual_search_string);
                let note_tagging_results = Database::get_note_ids_with_tag_like(&search_string_with_wildcards);
                for note_tagging in note_tagging_results {
                    individual_search_results.insert(note_tagging);
                }
            }
        }

        fn display_search_results_of(search_results: Difference<NoteTagging, RandomState>) {
            for search_result in search_results {
                if let Some(note) = Database::get_note_where_id(&search_result.note_id) {
                    let tag_name = &search_result.tag_name.as_ref();

                    if tag_name.is_some() {
                        let tag_name = tag_name.unwrap();
                        println!("{} {}\t\t{}{}",
                            note.note_id.yellow(), note.note_name,
                            "#".bright_yellow(), tag_name.bright_yellow());
                    }
                    else {
                        println!("{} {}", note.note_id.yellow(), note.note_name);
                    }
                }

            }
        }
    }

    pub fn add(note_name: &str, note_type: NoteType, settings: &mut Settings) {
        let template_path = Path::new(&settings.zettelkasten_dir).join("note-template.md");

        if !template_path.exists() {
            Message::error(&format!("the note template couldn't be found at '{}'",
                template_path.to_string_lossy()));
            return;
        }

        if let Some(note) = Notes::create_note_from_template(
            note_name,
            note_type,
            &settings.notes_dir,
            template_path.as_os_str()
        ) {
            Message::info(&format!("created note: {} {}", note.note_id.yellow(), note.note_name));

            Database::insert_note(&note.note_id, &note.note_name, &note.file_name, note.creation_date_time);
            Notes::open(&note.note_id, settings);
        }
    }

    fn create_note_from_template(note_name: &str, note_type: NoteType, notes_dir: &OsStr, template_path: &OsStr) -> Option<Note> {
        let creation_date_time = Local::now();
        let creation_timestamp = creation_date_time.format("%Y-%m-%d %H:%M:%S").to_string();
        let creation_file_timestamp = creation_date_time.format("%Y-%m-%d-%H%M%S").to_string();

        let note_type_identifier = match note_type {
            NoteType::Topic => "T",
            NoteType::Quote => "Q",
            NoteType::Journal => "J",
        };
        let note_id = format!("{}{}", note_type_identifier, creation_date_time.format("%Y%m%d%H%M%S").to_string());

        let file_name = format!("{}.md", &creation_file_timestamp);
        let file_path = Path::new(notes_dir).join(&file_name);

        let note_content = match Notes::get_content_from_file(&template_path) {
            Ok(file_content) => file_content,
            Err(error) => {
                Message::error(&format!("couldn't read template file: {}", error));
                return None;
            }
        };
        let note_content = note_content
            .replace("<note-name>", &note_name)
            .replace("<creation-date>", &creation_timestamp);

        let mut new_note = match File::create(&file_path) {
            Ok(created_file) => created_file,
            Err(error) => {
                Message::error(&format!("couldn't create file: {}", error));
                return None;
            }
        };
        if let Err(error) = new_note.write(note_content.as_bytes()) {
            Message::warning(&format!("couldn't apply template to created note: {}", error));
        };

        return Some(Note::new(note_id, note_name.to_string(), file_name, creation_date_time));
    }

    pub fn remove(note_name: &str, notes_dir: &OsStr) {
        let note_id = match Database::get_note_id_where(NoteProperty::NoteName, note_name) {
            Some(value) => value,
            None => {
                // Message::error(&format!("note couldn't be removed: the note '{}' does not exist!", note_name));
                // return;
                note_name.to_string()
            }
        };
        let note = match Database::get_note_where_id(&note_id) {
            Some(value) => value,
            None => {
                Message::error(&format!("note couldn't be removed: the note id or note name '{}' does not exist!", note_id));
                return;
            }
        };

        let note_file_path = Path::new(notes_dir).join(note.file_name);
        let note_tags = Database::get_tags_of_note(&note_id);

        for note_tag in note_tags {
            Database::delete_note_tagging(&note_id, &note_tag);

            let number_of_notes_with_current_tag = Database::get_note_ids_with_tag(&note_tag).len();
            if number_of_notes_with_current_tag == 0 {
                Database::delete_tag(&note_tag);
            }
        }

        Database::delete_note(&note_id);

        match fs::remove_file(&note_file_path){
            Ok(_) => {  },
            Err(error) => {
                Message::error(&format!("note file '{}' couldn't be removed: {}", note_file_path.to_string_lossy(), error));
                return;
            }
        };
    }
    
    pub fn open_random_note(settings: &mut Settings) {
        let note_id = match Database::get_random_note_id() {
            Some(value) => value,
            None => {
                Message::error("couldn't find a random note");
                return;
            }
        };
        
        let note = match Database::get_note_where_id(&note_id) {
            Some(value) => value,
            None => {
                Message::error(&format!("couldn't open note: the note id '{}' does not exist!", note_id));
                return;
            }
        };
        Message::info(&format!("opened note:  {} {} ", note_id.yellow(), note.note_name));

        Notes::open(&note_id, settings);
    }

    pub fn open(note_id: &str, settings: &mut Settings) {
        let note = match Database::get_note_where_id(note_id) {
            Some(value) => value,
            None => {
                Message::error(&format!("couldn't open note: the note id '{}' does not exist!", note_id));
                return;
            }
        };

        let notes_dir = &settings.notes_dir;
        let relative_file_path = Path::new(notes_dir).join(&note.file_name);

        let editor = match env::var("EDITOR") {
            Ok(value) => value,
            Err(error) => {
                Message::error(&format!("couldn't read the EDITOR environment variable: '{}'", error));
                return;
            }
        };

        let absolute_file_path = match relative_file_path.canonicalize() {
            Ok(path) => path,
            Err(error) => {
                Message::error(&format!("couldn't get the absolute path of {}: '{}'",
                    &note.file_name,
                    error));
                return;
            }
        };

        // Open the note in the editor specified by the EDITOR environment variable
        match Command::new(&editor).arg(&absolute_file_path).status() {
            Ok(_) => {  },
            Err(error) => {
                Message::error(&format!("couldn't open the note '{}': '{}'", &note.file_name, error));
                return;
            }
        };
        settings.add_to_note_history(note_id);

        Notes::check_yaml_header_of(&note, settings);
    }

    fn check_yaml_header_of(note: &Note, settings: &mut Settings) {
        let notes_dir = &settings.notes_dir;
        let absolute_note_file_path = PathBuf::from(notes_dir).join(&note.file_name);
        let yaml_header = match Notes::get_yaml_header_of(absolute_note_file_path.as_os_str()) {
            Ok(header) => header,
            Err(error) => {
                Message::error(&format!("couldn't read note file: '{}'", error));
                Notes::show_open_file_dialog_for(&note.note_id, settings);
                return;
            }
        };
        let yaml_files = match YamlLoader::load_from_str(&yaml_header) {
            Ok(yaml_vector) => yaml_vector,
            Err(error) => {
                Message::error(&format!("couldn't parse yaml header: '{}'", error));
                Notes::show_open_file_dialog_for(&note.note_id, settings);
                return;
            }
        };
        let note_metadata = &yaml_files[0];

        let is_check_complete = check_metadata_name_of(&note.note_id, note_metadata, &note.note_name, settings);
        if !is_check_complete { return; }

        let is_check_complete = check_metadata_tags_of(&note.note_id, note_metadata, settings);
        if !is_check_complete { return; }

        fn check_metadata_name_of(note_id: &str, note_metadata: &Yaml, original_note_name: &str, settings: &mut Settings) -> bool {
            match note_metadata["name"].as_str() {
                Some(new_note_name) => {
                    let whitespace_validator = Regex::new(r"^\s*$").unwrap();

                    if whitespace_validator.is_match(new_note_name) {
                        Message::error("this note doesn't have a name! please add a value after the 'name' property to the yaml header!");
                        Message::example("---\nname: \"note name\"\n---");
                        Notes::show_open_file_dialog_for(note_id, settings);
                        return false;
                    } else if new_note_name != original_note_name {
                        Database::update_note_name_where(new_note_name, NoteProperty::NoteId, note_id);
                    }
                }
                None => {
                    Message::error("this note doesn't have a name! please add a value after the 'name' property to the yaml header!");
                    Message::example("---\nname: \"note name\"\n---");
                    Notes::show_open_file_dialog_for(note_id, settings);
                    return false;
                }
            }

            return true;
        }

        fn check_metadata_tags_of(note_id: &str, note_metadata: &Yaml, settings: &mut Settings) -> bool {
            match note_metadata["tags"].as_vec() {
                Some(tags) => {
                    for tag in tags.iter() {
                        let tag = tag.as_str().unwrap();
                        Database::insert_tag_for_note(tag, note_id);
                    }
                }
                None => {
                    println!("{} this note doesn't have any tags! It will be difficult to find again!", "warning:".bold().yellow());
                    println!("Please add a few appropriate tags!");
                    println!("\n{}\n---\ntags: \"[ first-tag, second-tag, third-tag ]\"\n---\n", "example:".bold().yellow());
                    Notes::show_open_file_dialog_for(note_id, settings);
                    return false;
                }
            }

            return true;
        }
    }

    fn show_open_file_dialog_for(note_id: &str, settings: &mut Settings) {
        print!("Do you want to open the file again? [Y/n] ");
        io::stdout().flush().unwrap();

        let mut open_file_again = String::new();
        match io::stdin().read_line(&mut open_file_again) {
            Ok(_) => { },
            Err(error) => Message::error(&error.to_string())
        }

        if !(open_file_again.trim().to_lowercase() == "n") {
            Notes::open(note_id, settings);
        }
    }

    fn get_yaml_header_of(file_path: &OsStr) -> Result<String, Error> {
        let note_content = match Notes::get_content_from_file(file_path) {
            Ok(file_content) => file_content,
            Err(error) => return Err(error)
        };

        let yaml_start_index = note_content.find("---\n").unwrap();
        let yaml_end_index = note_content[yaml_start_index+3..].find("---").unwrap();
        let yaml_header = &note_content[yaml_start_index+3..yaml_end_index+3];

        return Ok(yaml_header.to_string());
    }

    fn get_content_from_file(path: &OsStr) -> Result<String, Error> {
        let mut file = match File::open(path) {
            Ok(opened_file) => opened_file,
            Err(error) => return Err(error)
        };
        let mut file_content = String::new();

        match file.read_to_string(&mut file_content) {
            Ok(_) => { },
            Err(error) => return Err(error)
        };

        return Ok(file_content);
    }
    
    pub fn print_note_history(settings: &Settings) {
        for note_id in settings.get_note_history_iterator() {
            if let Some(note) = Database::get_note_where_id(&note_id) {
                println!("{} {}", note_id.yellow(), note.note_name);
            }
        }
    }
}
