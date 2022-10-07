use crate::collection_tool::CollectionTool;
use crate::database::Database;
use crate::file_utility::FileUtility;
use crate::message::Message;
use crate::note::Note;
use crate::note_metadata::NoteMetadata;
use crate::note_property::NoteProperty;
use crate::note_tagging::NoteTagging;
use crate::note_type::NoteType;
use crate::settings::Settings;

use chrono::prelude::*;
use colored::*;
use indoc::{formatdoc, indoc};
use lazy_static::lazy_static;
use regex::Regex;
use std::collections::HashSet;
use std::env;
use std::ffi::{OsStr, OsString};
use std::fs::{self, File, OpenOptions};
use std::io::{self, Error, Write};
use std::path::{Path, PathBuf};
use std::process::Command;

lazy_static! {
    static ref NOTE_LINK_VALIDATOR: Regex = Regex::new(
        r"(?x)
        \[\[
            ([a-zA-Z0-9]+?)     # $1 = Link text
        \]\]
    "
    )
    .unwrap();
    static ref NOTE_FORMAT_VALIDATOR: Regex = Regex::new(
        r##"(?xs)
        (                   # $1 = yaml header
            ^
            \s*
            ---[ \t]*
            \n*
            (.*?)           # $2 = yaml text
            \n*
            ---[ \t]*
            \n?
        )
        (.*)                # $3 = body of the note
        $
    "##
    )
    .unwrap();
    static ref NOTE_CONTENT_BACKLINK_VALIDATOR: Regex = Regex::new(
        r##"(?xs)
        (                                                   # $1 = text before backlinks
            ^
            \s*
            ---[ \t]*
            \n*
            .*?
        )
        (                                                   # $2 = backlinks category
            backlinks:\s?
            \[
            \s+
            (                                               # $3 = all backlinks
                (                                           # $4 = single backlink
                    \s?
                    \[\[[a-zA-Z0-9]+\]\]
                    [,]?
                )*
            )
            \s*
            \]
            [ \t]*
        )
        (                                                   # $5 = text after backlinks
            \n?
            .*?
            \n*
            ---[ \t]*
            \n?
            .*
            $
        )
    "##
    )
    .unwrap();
    static ref NOTE_NAME_VALIDATOR: Regex = Regex::new(
        r##"(?x)
        ^[^!?$%§&/=\{\}+*\#|~^@]*$
    "##
    )
    .unwrap();
    static ref TAG_NAME_VALIDATOR: Regex = Regex::new(
        r##"(?x)
        ^[^!?$%§&/=\{\}+*\|~^@]*$
    "##
    )
    .unwrap();
    static ref WHITESPACE_VALIDATOR: Regex = Regex::new(r"^\s*$").unwrap();
}

pub struct NoteUtility;
impl NoteUtility {
    pub fn list(count: i32) {
        let note_ids = Database::get_all_recent_note_ids(count);

        for note_id in note_ids {
            let note = Database::get_note_where_id(&note_id).unwrap();
            println!("{} {}", note_id.yellow(), note.note_name);
        }
    }

    pub fn get(count: i32) -> Vec<String> {
        let note_ids = Database::get_all_recent_note_ids(count);
        let mut note_list = Vec::new();

        for note_id in note_ids {
            let note = Database::get_note_where_id(&note_id).unwrap();
            note_list.push(note.note_name);
        }

        return note_list;
    }

    pub fn search(complete_search_string: &str) -> Vec<NoteTagging> {
        let split_search_strings = complete_search_string.split(" && ");
        let mut search_results: HashSet<NoteTagging> = HashSet::new();
        let mut negated_search_results: HashSet<NoteTagging> = HashSet::new();

        for search_string in split_search_strings {
            search_notes_and_add_to(
                search_string,
                &mut search_results,
                &mut negated_search_results,
            );
        }

        let search_results = search_results.difference(&negated_search_results);

        return search_results.cloned().collect();

        fn search_notes_and_add_to(
            mut search_string: &str,
            search_results: &mut HashSet<NoteTagging>,
            negated_search_results: &mut HashSet<NoteTagging>,
        ) {
            let mut individual_search_results = HashSet::new();
            let mut is_search_string_for_tag = false;
            let mut is_negated_search_string = false;
            let mut search_string_chars = search_string.chars();

            loop {
                let first_char = match search_string_chars.next() {
                    Some(result) => result,
                    None => break,
                };

                if first_char == '!' {
                    is_negated_search_string = true;
                    search_string = search_string_chars.as_str();
                } else if first_char == '#' {
                    is_search_string_for_tag = true;
                    search_string = search_string_chars.as_str();
                } else {
                    break;
                }
            }

            // Get individual search results
            if !is_search_string_for_tag {
                search_note_names(&mut individual_search_results, search_string);
            }
            search_tags(&mut individual_search_results, search_string);
            search_note_ids(&mut individual_search_results, search_string);

            // Intersect with already existing search results
            if is_negated_search_string {
                if negated_search_results.is_empty() {
                    *negated_search_results = individual_search_results;
                } else {
                    *negated_search_results = CollectionTool::intersect(
                        negated_search_results,
                        &mut individual_search_results,
                    );
                }
            } else {
                if search_results.is_empty() {
                    *search_results = individual_search_results;
                } else {
                    *search_results =
                        CollectionTool::intersect(search_results, &mut individual_search_results);
                }
            }

            fn search_note_names(
                individual_search_results: &mut HashSet<NoteTagging>,
                individual_search_string: &str,
            ) {
                let search_string_with_wildcards = format!("%{}%", individual_search_string);
                let note_ids = Database::get_note_ids_where_property_is_like(
                    NoteProperty::NoteName,
                    &search_string_with_wildcards,
                );
                for note_id in note_ids {
                    individual_search_results.insert(NoteTagging::from(note_id, None));
                }
            }

            fn search_note_ids(
                individual_search_results: &mut HashSet<NoteTagging>,
                individual_search_string: &str,
            ) {
                let search_string_with_wildcards = format!("%{}%", individual_search_string);
                let note_ids = Database::get_note_ids_where_property_is_like(
                    NoteProperty::NoteId,
                    &search_string_with_wildcards,
                );
                for note_id in note_ids {
                    individual_search_results.insert(NoteTagging::from(note_id, None));
                }
            }

            fn search_tags(
                individual_search_results: &mut HashSet<NoteTagging>,
                individual_search_string: &str,
            ) {
                let search_string_with_wildcards = format!("%{}%", individual_search_string);
                let note_tagging_results =
                    Database::get_note_ids_with_tag_like(&search_string_with_wildcards);
                for note_tagging in note_tagging_results {
                    individual_search_results.insert(note_tagging);
                }
            }
        }
    }

    pub fn print_search_results(search_results: Vec<NoteTagging>) {
        for search_result in search_results {
            if let Some(note) = Database::get_note_where_id(&search_result.note_id) {
                let tag_name = &search_result.tag_name.as_ref();

                if tag_name.is_some() {
                    let tag_name = tag_name.unwrap();
                    println!(
                        "{} {}\t\t{}{}",
                        note.note_id.yellow(),
                        note.note_name,
                        "#".bright_yellow(),
                        tag_name.bright_yellow()
                    );
                } else {
                    println!("{} {}", note.note_id.yellow(), note.note_name);
                }
            }
        }
    }

    pub fn add(
        note_name: &str,
        note_type: NoteType,
        settings: &mut Settings,
    ) -> Result<Option<String>, String> {
        let template_path = Path::new(&settings.zettelkasten_dir).join("note-template.md");
        if !template_path.exists() {
            return Err(format!(
                "add_note: the note template couldn't be found at '{}'",
                template_path.to_string_lossy()
            ));
        }

        if !NOTE_NAME_VALIDATOR.is_match(note_name) {
            return Err(format!(
                "add_note: the note name '{}' contains illegal characters",
                note_name
            ));
        }

        if let Some(note) = NoteUtility::create_note_from_template(
            note_name,
            note_type,
            &settings.notes_dir,
            template_path.as_os_str(),
        ) {
            if settings.print_to_stdout {
                Message::info(&format!(
                    "created note: {} {}",
                    note.note_id.yellow(),
                    note.note_name
                ));
            }

            Database::insert_note(&note);
            return Ok(Some(note.note_id));
        }

        return Ok(None);
    }

    fn create_note_from_template(
        note_name: &str,
        note_type: NoteType,
        notes_dir: &OsStr,
        template_path: &OsStr,
    ) -> Option<Note> {
        let creation_date_time = Local::now();
        let creation_timestamp = creation_date_time.format("%Y-%m-%d %H:%M:%S").to_string();
        let creation_file_timestamp = creation_date_time.format("%Y-%m-%d-%H%M%S").to_string();

        let note_type_identifier = match note_type {
            NoteType::Topic => "T",
            NoteType::Quote => "Q",
            NoteType::Journal => "J",
        };
        let note_id = format!(
            "{}{}",
            note_type_identifier,
            creation_date_time.format("%Y%m%d%H%M%S").to_string()
        );

        let file_name = format!("{}.md", &creation_file_timestamp);
        let file_path = Path::new(notes_dir).join(&file_name);

        let note_content = match FileUtility::get_content_from_file(&template_path) {
            Ok(file_content) => file_content,
            Err(error) => {
                Message::error(&format!(
                    "create_note_from_template: couldn't read template file: {}",
                    error
                ));
                return None;
            }
        };
        let note_content = note_content
            .replace("<note-id>", &note_id)
            .replace("<note-name>", &note_name)
            .replace("<creation-date>", &creation_timestamp);

        let mut new_note = match File::create(&file_path) {
            Ok(created_file) => created_file,
            Err(error) => {
                Message::error(&format!(
                    "create_note_from_template: couldn't create file: {}",
                    error
                ));
                return None;
            }
        };

        if let Err(error) = new_note.write(note_content.as_bytes()) {
            Message::warning(&format!(
                "create_note_from_template: couldn't apply template to created note: {}",
                error
            ));
        };

        return Some(Note::new(
            note_id,
            note_name.to_string(),
            file_name,
            creation_date_time,
        ));
    }

    pub fn remove(note_name: &str, notes_dir: &OsStr) -> Result<(), String> {
        let note_id = match Database::get_note_id_where(NoteProperty::NoteName, note_name) {
            Some(value) => value,
            None => {
                // try if the 'note name' is already the note id
                // (if not then the next step would cause an error which is on purpose)
                note_name.to_string()
            }
        };
        let note = match Database::get_note_where_id(&note_id) {
            Some(value) => value,
            None => {
                return Err(format!("remove_note: note couldn't be removed: the note id or note name '{}' does not exist!", note_id));
            }
        };

        let note_file_path = Path::new(notes_dir).join(note.file_name);

        NoteUtility::delete_tags_of_note(&note_id);
        Database::delete_all_links_with_note(&note_id);
        Database::delete_note(&note_id);

        match fs::remove_file(&note_file_path) {
            Ok(_) => {}
            Err(error) => {
                return Err(format!(
                    "remove_note: note file '{}' couldn't be removed: {}",
                    note_file_path.to_string_lossy(),
                    error
                ));
            }
        };

        return Ok(());
    }

    fn delete_tags_of_note(note_id: &str) {
        let note_tags = Database::get_tags_of_note(note_id);

        for note_tag in note_tags {
            Database::delete_note_tagging(note_id, &note_tag);

            let number_of_notes_with_current_tag = Database::get_note_ids_with_tag(&note_tag).len();
            if number_of_notes_with_current_tag == 0 {
                Database::delete_tag(&note_tag);
            }
        }
    }

    pub fn update_db_for_all_notes_in_project_folder(
        settings: &mut Settings,
    ) -> Result<(), String> {
        let cleared_successfully = Database::clear();
        if !cleared_successfully {
            return Err(format!("update-db: Database couldn't be cleared!"));
        }

        println!("(1/4) Create Database...");
        Database::init();
        println!("(1/4) Create Database: Done");

        println!("(2/4) Get all notes in directory...");
        let note_metadata_list = match NoteUtility::get_all_note_metadata(settings) {
            Ok(result) => result,
            Err(error) => return Err(error),
        };
        println!("(2/4) Get all notes in directory: Done");

        settings.show_interactive_dialogs = false;

        // First insert all notes before inserting tags and links
        // as they depend on notes
        let mut counter = 0;
        let mut former_percentage = 0;
        println!("(3/4) Update notes...");
        for note_metadata in &note_metadata_list {
            Database::insert_note(&note_metadata);

            counter += 1;
            let completion_percentage =
                ((counter as f32) / (note_metadata_list.len() as f32) * 100.) as usize;
            if completion_percentage - former_percentage >= 5 {
                println!("(3/4) Update notes: {}%", completion_percentage);
                former_percentage = completion_percentage;
            }
        }
        println!("(3/4) Update notes: Done");

        counter = 0;
        former_percentage = 0;
        println!("(4/4) Update note links and tags...");
        for note_metadata in &note_metadata_list {
            NoteUtility::check_links_in_note(&note_metadata, settings);

            counter += 1;
            match NoteUtility::check_metadata_of(&note_metadata, settings) {
                Ok(None) => (),
                Ok(Some(message)) => Message::warning(&message),
                Err(error) => Message::error(&format!("check_yaml_header_of: {}", error)),
            };

            let completion_percentage =
                ((counter as f32) / (note_metadata_list.len() as f32) * 100.) as usize;
            if completion_percentage - former_percentage >= 5 {
                println!(
                    "(4/4) Update note links and tags: {}%",
                    completion_percentage
                );
                former_percentage = completion_percentage;
            }
        }
        println!("(4/4) Update note links and tags: Done");

        settings.show_interactive_dialogs = true;
        return Ok(());
    }

    fn get_all_note_metadata(settings: &mut Settings) -> Result<Vec<Note>, String> {
        let directory_entries = fs::read_dir(&settings.notes_dir).unwrap();
        let mut note_metadata_list: Vec<Note> = Vec::new();

        for entry in directory_entries {
            let path = entry.unwrap().path();
            if path.is_dir() {
                continue;
            }

            let file_name = match path.file_name().unwrap().to_str() {
                Some(value) => value,
                None => {
                    return Err(format!(
                        "update-db: The file name '{}' contains illegal characters!",
                        path.to_string_lossy()
                    ))
                }
            };

            let absolute_note_file_path = PathBuf::from(&settings.notes_dir).join(&file_name);
            let _ = match NoteMetadata::get_basic_data_of_file(&absolute_note_file_path) {
                Ok(note_metadata) => {
                    note_metadata_list.push(note_metadata);
                }
                Err(error) => return Err(error),
            };
        }

        return Ok(note_metadata_list);
    }

    pub fn get_content_of_note(note_id: &str, settings: &mut Settings) -> Result<String, String> {
        let absolute_file_path = match NoteUtility::get_absolute_path_of_note(note_id, settings) {
            Ok(value) => value,
            Err(error) => return Err(error),
        };

        let note_content = match FileUtility::get_content_from_file(&absolute_file_path) {
            Ok(value) => value,
            Err(error) => {
                return Err(error.to_string());
            }
        };
        return Ok(note_content);
    }

    pub fn open_random_note(settings: &mut Settings) {
        let note_id;
        let random_notes = Database::get_random_note_ids(1);
        if random_notes.len() >= 1 {
            note_id = &random_notes[0];
        } else {
            Message::error("open_random_note: couldn't find a random note");
            return;
        }

        let note = match Database::get_note_where_id(note_id) {
            Some(value) => value,
            None => {
                Message::error(&format!(
                    "open_random_note: couldn't open note: the note id '{}' does not exist!",
                    note_id
                ));
                return;
            }
        };
        Message::info(&format!(
            "opened note:  {} {} ",
            note_id.yellow(),
            note.note_name
        ));

        if let Err(error) = NoteUtility::open(note_id, settings) {
            Message::error(&error);
        }
    }

    pub fn open(note_id: &str, settings: &mut Settings) -> Result<Option<String>, String> {
        let note = match Database::get_note_where_id(note_id) {
            Some(value) => value,
            None => {
                return Err(format!("the note id '{}' does not exist!", note_id));
            }
        };
        let editor = match env::var("EDITOR") {
            Ok(value) => value,
            Err(error) => {
                return Err(format!(
                    "couldn't read the EDITOR environment variable: '{}'",
                    error
                ));
            }
        };
        let absolute_file_path = match NoteUtility::get_absolute_path_of_note(note_id, settings) {
            Ok(value) => value,
            Err(error) => {
                return Err(format!(
                    "couldn't get the absolute path of {}: '{}'",
                    &note.file_name, error
                ));
            }
        };

        // Open the note in the editor specified by the EDITOR environment variable
        match Command::new(&editor).arg(&absolute_file_path).status() {
            Ok(_) => {}
            Err(error) => {
                return Err(format!(
                    "couldn't open the note '{}' with '{}': '{}'",
                    &editor, &note.file_name, error
                ));
            }
        };

        if let Err(error) = settings.note_history.add(note_id) {
            return Err(error);
        }

        NoteUtility::check_links_in_note(&note, settings);

        match NoteUtility::check_metadata_of(&note, settings) {
            Ok(None) => return Ok(None),
            Ok(Some(message)) => return Ok(Some(message)),
            Err(error) => return Err("check_yaml_header_of: ".to_string() + &error),
        }
    }

    fn get_absolute_path_of_note(
        note_id: &str,
        settings: &mut Settings,
    ) -> Result<OsString, String> {
        let note = match Database::get_note_where_id(note_id) {
            Some(value) => value,
            None => {
                return Err(format!("the note id '{}' does not exist!", note_id));
            }
        };

        let notes_dir = &settings.notes_dir;
        let relative_file_path = Path::new(notes_dir).join(&note.file_name);
        let absolute_file_path = match relative_file_path.canonicalize() {
            Ok(path) => path,
            Err(error) => {
                return Err(error.to_string());
            }
        };

        return Ok(absolute_file_path.as_os_str().to_os_string());
    }

    fn check_links_in_note(note: &Note, settings: &Settings) {
        if let Some(note_links) = NoteUtility::get_all_links_in_note(&note, settings) {
            if settings.backlinking_enabled {
                NoteUtility::create_backlinks_from(&note_links, &note, settings);
            }
            for note_link_id in note_links {
                if let Err(error) =
                    Database::insert_note_link_for_note(&note.note_id, &note_link_id)
                {
                    Message::warning(&error);
                }
            }
        }
    }

    fn get_all_links_in_note(note: &Note, settings: &Settings) -> Option<Vec<String>> {
        let absolute_note_file_path = PathBuf::from(&settings.notes_dir).join(&note.file_name);
        let note_content = match FileUtility::get_content_from_file(&absolute_note_file_path) {
            Ok(value) => value,
            Err(error) => {
                Message::error(&format!(
                    "get-all-links-in-note: couldn't read content of note '{} {}': {}",
                    note.note_id.yellow(),
                    note.note_name,
                    error
                ));
                return None;
            }
        };

        if let Some(note_format_match) = NOTE_FORMAT_VALIDATOR.captures(&note_content) {
            let note_body = note_format_match.get(3).unwrap().as_str();
            let mut note_links = Vec::new();
            for note_link_match in NOTE_LINK_VALIDATOR.captures_iter(note_body) {
                let linked_note_id = note_link_match.get(1).unwrap().as_str();
                note_links.push(linked_note_id.to_string());
            }
            return Some(note_links);
        } else {
            Message::warning(&format!("get-all-links-in-note: couldn't search for links in '{} {}': note does not have the correct format",
                note.note_id.yellow(),
                note.note_name));
            Message::display_correct_note_format();
            return None;
        }
    }

    fn create_backlinks_from(note_links: &Vec<String>, source_note: &Note, settings: &Settings) {
        for linked_note_id in note_links {
            if let Some(linked_note) = Database::get_note_where_id(&linked_note_id) {
                NoteUtility::add_backlink_to(&linked_note, &source_note.note_id, settings)
            }
        }
    }

    fn add_backlink_to(note: &Note, backlink_id: &str, settings: &Settings) {
        let absolute_note_file_path = PathBuf::from(&settings.notes_dir).join(&note.file_name);
        let note_content = match FileUtility::get_content_from_file(&absolute_note_file_path) {
            Ok(value) => value,
            Err(error) => {
                Message::error(&format!(
                    "add_backlink: couldn't read note file '{}': {}",
                    &absolute_note_file_path.to_string_lossy(),
                    error
                ));
                return;
            }
        };

        if let Some(note_content_match) = NOTE_CONTENT_BACKLINK_VALIDATOR.captures(&note_content) {
            let text_before_backlinks = note_content_match.get(1).unwrap().as_str().to_string();
            let backlinks = note_content_match.get(3).unwrap().as_str();
            let text_after_backlinks = note_content_match.get(5).unwrap().as_str();

            if backlink_exists_in_string(backlink_id, backlinks) {
                return;
            }

            let backlinks_string = create_backlinks_string_from(backlinks, backlink_id);
            let new_note_content = text_before_backlinks + &backlinks_string + text_after_backlinks;

            if let Err(error) = NoteUtility::replace_content_of_file(
                &absolute_note_file_path,
                new_note_content.as_bytes(),
            ) {
                Message::error(&format!(
                    "add_backlink: couldn't change contents of note '{} {}': {}",
                    note.note_id.yellow(),
                    note.note_name,
                    error
                ));
            };
        } else {
            Message::error(&format!(
                "couldn't add backlink to '{} {}': note does not have the correct format",
                note.note_id.yellow(),
                note.note_name
            ));
            return;
        }

        fn create_backlinks_string_from(
            existing_backlinks_list: &str,
            new_backlink_id: &str,
        ) -> String {
            let mut new_backlinks_list = String::from(existing_backlinks_list);

            if new_backlinks_list.len() > 0 {
                new_backlinks_list.push_str(", ");
            }
            new_backlinks_list.push_str(&format!("[[{}]]", new_backlink_id));

            let backlinks_string = format!("backlinks: [ {} ]", new_backlinks_list);
            return backlinks_string;
        }

        fn backlink_exists_in_string(backlink_id: &str, existing_backlinks_string: &str) -> bool {
            for existing_backlink in NOTE_LINK_VALIDATOR.captures_iter(existing_backlinks_string) {
                let existing_backlink_id = existing_backlink.get(1).unwrap().as_str();
                if existing_backlink_id == backlink_id {
                    return true;
                }
            }

            return false;
        }
    }

    fn replace_content_of_file<P: AsRef<Path>>(
        path: P,
        new_file_content: &[u8],
    ) -> Result<(), Error> {
        let mut note_file = match OpenOptions::new().write(true).truncate(true).open(path) {
            Ok(opened_file) => opened_file,
            Err(error) => return Err(error),
        };
        if let Err(error) = note_file.write(new_file_content) {
            return Err(error);
        }

        return Ok(());
    }

    fn show_open_file_dialog_for(note_id: &str, settings: &mut Settings) {
        if !settings.show_interactive_dialogs {
            return;
        }

        print!("Do you want to open the file again? [Y/n] ");
        io::stdout().flush().unwrap();

        let mut open_file_again = String::new();
        match io::stdin().read_line(&mut open_file_again) {
            Ok(_) => {}
            Err(error) => {
                Message::error(&format!(
                    "show_open_file_dialog: couldn't read user input: {}",
                    error
                ));
                return;
            }
        }

        if !(open_file_again.trim().to_lowercase() == "n") {
            if let Err(error) = NoteUtility::open(note_id, settings) {
                Message::error(&error);
            }
        }
    }

    pub fn get_note_history(settings: &Settings) -> Vec<Note> {
        let mut note_list: Vec<Note> = Vec::new();

        for note_id in settings.note_history.list() {
            if let Some(note) = Database::get_note_where_id(&note_id) {
                note_list.push(note);
            }
        }

        return note_list;
    }

    pub fn print_note_list(note_list: Vec<Note>) {
        for note in note_list {
            println!("{} {}", note.note_id.yellow(), note.note_name);
        }
    }

    pub fn print_note_name_of(note_id: &str) {
        if let Some(note) = Database::get_note_where_id(note_id) {
            println!("{}", note.note_name);
        } else {
            Message::error(&format!("the note id {} doesn't exist", note_id.yellow()));
        }
    }

    pub fn print_file_name_of(note_id: &str) {
        if let Some(note) = Database::get_note_where_id(note_id) {
            println!("{}", note.file_name);
        } else {
            Message::error(&format!("the note id {} doesn't exist", note_id.yellow()));
        }
    }

    fn check_metadata_of(note: &Note, settings: &mut Settings) -> Result<Option<String>, String> {
        let mut warning_message = String::new();
        let note_name = match NoteMetadata::get_property_of(note, NoteProperty::NoteName, settings)
        {
            Ok(value) => value,
            Err(error) => return Err(error),
        };
        let tags = match NoteMetadata::get_tags_of(note, settings) {
            Ok(value) => value,
            Err(error) => return Err(error),
        };

        match NoteUtility::check_metadata_name_of(
            &note.note_id,
            note_name,
            &note.note_name,
            settings,
        ) {
            Err(error) => return Err(error),
            Ok(Some(warning)) => warning_message = warning,
            Ok(None) => (),
        }

        match NoteUtility::check_metadata_tags_of(&note.note_id, tags, settings) {
            Err(error) => return Err(error),
            Ok(Some(warning)) => warning_message = warning,
            Ok(None) => (),
        }

        if warning_message == "" {
            return Ok(None);
        } else {
            return Ok(Some(warning_message));
        }
    }

    fn check_metadata_name_of(
        note_id: &str,
        note_name: Option<String>,
        original_note_name: &str,
        settings: &mut Settings,
    ) -> Result<Option<String>, String> {
        match note_name {
            Some(new_note_name) => {
                if WHITESPACE_VALIDATOR.is_match(&new_note_name) {
                    if settings.print_to_stdout {
                        show_missing_note_name_warning_for(note_id, settings);
                    }
                    return Err(format!("the note '{}' doesn't have a name! please add a value after the 'name' property to the yaml header!",
                                    note_id));
                } else if new_note_name != original_note_name {
                    Database::update_note_name_where(&new_note_name, NoteProperty::NoteId, note_id);
                }
            }
            None => {
                if settings.print_to_stdout {
                    show_missing_note_name_warning_for(note_id, settings);
                }
                return Err(format!("the note '{}' doesn't have a name! please add a value after the 'name' property to the yaml header!",
                                note_id));
            }
        }

        return Ok(None);

        fn show_missing_note_name_warning_for(note_id: &str, settings: &mut Settings) {
            Message::error("this note doesn't have a name! please add a value after the 'name' property to the yaml header!");
            Message::example(indoc! {r#"
                ---

                name: "note name"

                ---
            "#});
            NoteUtility::show_open_file_dialog_for(note_id, settings);
        }
    }

    fn check_metadata_tags_of(
        note_id: &str,
        tags: Option<Vec<String>>,
        settings: &mut Settings,
    ) -> Result<Option<String>, String> {
        NoteUtility::delete_tags_of_note(note_id);

        match tags {
            Some(tags) => {
                if tags.len() == 0 {
                    if settings.print_to_stdout {
                        show_missing_tags_warning_for(note_id, settings);
                    } else {
                        return Ok(Some(format!("the note '{}' doesn't have any tags! It will be difficult to find again!", note_id)));
                    }
                }

                for tag in tags.iter() {
                    if TAG_NAME_VALIDATOR.is_match(tag) {
                        Database::insert_tag_for_note(tag, note_id);
                    } else {
                        return Err(format!(
                            "check_tags: the tag name '{}' contains illegal characters",
                            tag
                        ));
                    }
                }
            }
            None => {
                if settings.print_to_stdout {
                    show_missing_tags_warning_for(note_id, settings);
                } else {
                    return Ok(Some(format!(
                        "the note '{}' doesn't have any tags! It will be difficult to find again!",
                        note_id
                    )));
                }
            }
        }

        return Ok(None);

        fn show_missing_tags_warning_for(note_id: &str, settings: &mut Settings) {
            Message::warning(&formatdoc! {"
                the note '{}' doesn't have any tags! It will be difficult to find again!
                please add a few appropriate tags!
            ", &note_id});
            Message::example(indoc! {r##"
                ---

                tags: [ first-tag, #second-tag, third-tag ]

                ---
            "##});

            NoteUtility::show_open_file_dialog_for(note_id, settings);
        }
    }
}
