#[path = "database.rs"]
mod database;
#[path = "message.rs"]
mod message;

use database::Database;
use message::Message;

use chrono::prelude::*;
use colored::*;
use regex::Regex;
use std::env;
use std::fs::{ self, File };
use std::io::{ self, Error, Read, Write };
use std::path::Path;
use std::process::Command;
use yaml_rust::{ YamlLoader, Yaml };

pub struct Notes;
impl Notes {
    pub fn list(count: i32) {
        let note_names = Database::get_all_recent_note_names(count);

        for note_name in note_names {
            let note_id = Database::get_note_id_where(&format!("note_name = '{}'", note_name)).unwrap();
            println!("{} {}", note_id.yellow(), note_name);
        }

        // let entries = fs::read_dir("./").unwrap();
        // for entry in entries {
        //     let entry = entry.unwrap();
        //     print_if_file(&entry);
        // }
        //
        // fn print_if_file(entry: &DirEntry) {
        //     let path = entry.path();
        //     if path.is_file() {
        //         let file_stem = path.file_stem().unwrap().to_str().expect("The filename contains illegal characters!");
        //         if !file_stem.starts_with(".") {
        //             println!("-- {}", file_stem);
        //         }
        //     }
        // }
    }

    pub fn add(note_name: &str, notes_dir: &str) {
        let notes_dir_path = Path::new(notes_dir);
        let template_path = notes_dir_path.join(".zettelkasten").join("note-template.md");

        if !template_path.exists() {
            println!("The note template couldn't be found at {:?}", template_path);
            return;
        }

        let (note_id, file_name, creation_date_time) = Notes::create_note_from_template(note_name, notes_dir, &template_path.to_str().unwrap());

        if (note_id != None) && (file_name != None) && (creation_date_time != None) {
            let note_id = note_id.unwrap();
            let file_name = file_name.unwrap();
            let creation_date_time = creation_date_time.unwrap();

            Database::insert_note(&note_id, note_name, &file_name, creation_date_time);
            Notes::open(&note_id);
        }
    }

    fn create_note_from_template(note_name: &str, notes_dir: &str, template_path: &str) -> (Option<String>, Option<String>, Option<DateTime<Local>>) {
        let creation_date_time = Local::now();
        let creation_timestamp = creation_date_time.format("%Y-%m-%d %H:%M:%S").to_string();
        let creation_file_timestamp = creation_date_time.format("%Y-%m-%d-%H%M%S").to_string();
        let note_id = creation_date_time.format("%Y%m%d%H%M%S").to_string();

        let file_name = format!("{}.md", &creation_file_timestamp);
        let file_path = Path::new(notes_dir).join(&file_name);

        let note_content = match Notes::get_content_from_file(&template_path) {
            Ok(file_content) => file_content,
            Err(error) => {
                Message::error(&format!("couldn't read template file: '{}'", error));
                return (None, None, None);
            }
        };
        let note_content = note_content
            .replace("<note-name>", &note_name)
            .replace("<creation-date>", &creation_timestamp);

        let mut new_note = match File::create(&file_path) {
            Ok(created_file) => created_file,
            Err(error) => {
                Message::error(&format!("couldn't create file: '{}'", error));
                return (None, None, None);
            }
        };
        match new_note.write(note_content.as_bytes()) {
            Ok(_) => {  },
            Err(error) => {
                Message::warning(&format!("couldn't apply template to created note: '{}'", error));
            }
        };

        return (Some(note_id), Some(file_name), Some(creation_date_time));
    }

    pub fn remove(note_name: &str, notes_dir: &str) {
        let note_id = match Database::get_note_id_where(&format!("note_name = '{}'", note_name)) {
            Some(value) => value,
            None => {
                Message::error(&format!("the note '{}' does note exist!", note_name));
                return;
            }
        };

        let note_file_name = match Database::get_file_name_where(&format!("note_id = '{}'", note_id)) {
            Some(value) => value,
            None => {
                Message::error(&format!("the note id '{}' does note exist!", note_id));
                return;
            }
        };
        let note_file_path = Path::new(notes_dir).join(note_file_name);

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
                Message::error(&format!("note file '{}' couldn't be removed: '{}'", note_file_path.to_str().unwrap(), error));
                return;
            }
        };
    }

    pub fn open(note_id: &str) {
        let file_name = match Database::get_file_name_where(&format!("note_id = '{}'", note_id)) {
            Some(value) => value,
            None => {
                Message::error(&format!("the note id '{}' does not exist!", note_id));
                return;
            }
        };
        let relative_file_path = Path::new(&file_name);
        let original_note_name = match Database::get_note_name_where(&format!("note_id = '{}'", note_id)) {
            Some(value) => value,
            None => {
                Message::error(&format!("the note id '{}' does not exist!", note_id));
                return;
            }
        };

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
                Message::error(&format!("couldn't get the absolute path of {}: '{}'", &file_name, error));
                return;
            }
        };

        // Open the note in the editor specified by the EDITOR environment variable
        match Command::new(&editor).arg(&absolute_file_path).status() {
            Ok(_) => {  },
            Err(error) => {
                Message::error(&format!("couldn't open the note '{}': '{}'", &file_name, error));
                return;
            }
        };

        Notes::check_yaml_header_of(&file_name, note_id, &original_note_name);
    }

    fn check_yaml_header_of(file_name: &str, note_id: &str, original_note_name: &str) {
        let yaml_header = match Notes::get_yaml_header_of(file_name) {
            Ok(header) => header,
            Err(error) => {
                Message::error(&format!("couldn't read note file: '{}'", error));
                Notes::show_open_file_dialog_for(note_id);
                return;
            }
        };
        let yaml_files = match YamlLoader::load_from_str(&yaml_header) {
            Ok(yaml_vector) => yaml_vector,
            Err(error) => {
                Message::error(&format!("couldn't parse yaml header: '{}'", error));
                Notes::show_open_file_dialog_for(note_id);
                return;
            }
        };
        let note_metadata = &yaml_files[0];

        let is_check_complete = check_metadata_name_of(note_id, note_metadata, original_note_name);
        if !is_check_complete { return; }

        let is_check_complete = check_metadata_tags_of(note_id, note_metadata);
        if !is_check_complete { return; }

        fn check_metadata_name_of(note_id: &str, note_metadata: &Yaml, original_note_name: &str) -> bool {
            match note_metadata["name"].as_str() {
                Some(new_note_name) => {
                    let whitespace_validator = Regex::new(r"^\s*$").unwrap();

                    if whitespace_validator.is_match(new_note_name) {
                        Message::error("this note doesn't have a name! please add a value after the 'name' property to the yaml header!");
                        Message::example("---\nname: \"note name\"\n---");
                        Notes::show_open_file_dialog_for(note_id);
                        return false;
                    } else if new_note_name != original_note_name {
                        Database::update_note_name_where(new_note_name, &format!("note_id = '{}'", note_id));
                    }
                }
                None => {
                    Message::error("this note doesn't have a name! please add a value after the 'name' property to the yaml header!");
                    Message::example("---\nname: \"note name\"\n---");
                    Notes::show_open_file_dialog_for(note_id);
                    return false;
                }
            }

            return true;
        }

        fn check_metadata_tags_of(note_id: &str, note_metadata: &Yaml) -> bool {
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
                    Notes::show_open_file_dialog_for(note_id);
                    return false;
                }
            }

            return true;
        }
    }

    fn show_open_file_dialog_for(note_id: &str) {
        print!("Do you want to open the file again? [Y/n] ");
        io::stdout().flush().unwrap();

        let mut open_file_again = String::new();
        match io::stdin().read_line(&mut open_file_again) {
            Ok(_) => { },
            Err(error) => Message::error(&error.to_string())
        }

        if !(open_file_again.trim().to_lowercase() == "n") {
            Notes::open(note_id);
        }
    }

    fn get_yaml_header_of(file_path: &str) -> Result<String, Error> {
        let note_content = match Notes::get_content_from_file(file_path) {
            Ok(file_content) => file_content,
            Err(error) => return Err(error)
        };

        let yaml_start_index = note_content.find("---\n").unwrap();
        let yaml_end_index = note_content[yaml_start_index+3..].find("---").unwrap();
        let yaml_header = &note_content[yaml_start_index+3..yaml_end_index+3];

        return Ok(yaml_header.to_string());
    }

    fn get_content_from_file(path: &str) -> Result<String, Error> {
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
}
