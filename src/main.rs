mod database;
mod directory;
mod message;
mod note_property;
mod note;
mod notes;
mod settings;

use settings::Settings;
use directory::Directory;
use note_property::NoteProperty;
use notes::Notes;
use database::Database;
use message::Message;

use clap::{ Arg, App, SubCommand, AppSettings, ArgMatches, crate_name, crate_version };
use std::env;
use std::ffi::OsStr;
use std::fs;
use std::path::Path;

fn main() {
    let matches = App::new("Brain")
        .version(crate_version!())
        .about("A Zettelkasten program that helps you organize your notes.")
        .after_help(format!("Use \'{} help <subcommand>\' to view detailed information of subcommands.", crate_name!()).as_str())
        .setting(AppSettings::ArgRequiredElseHelp)
        .arg(Arg::with_name("directory")
            .help("Set zettelkasten directory that should be accessed (default is current directory)")
            .short("d")
            .long("dir")
            .takes_value(true)
        )
        .subcommand(SubCommand::with_name("init")
            .about("Initializes a new zettelkasten directory")
        )
        .subcommand(SubCommand::with_name("list")
            .about("Lists the last created notes")
            .arg(Arg::with_name("count")
                .help("The number of notes to show")
                .short("c")
                .long("count")
                .takes_value(true)
                .default_value("100")
            )
        )
        .subcommand(SubCommand::with_name("open")
            .about("Opens the specified note")
            .arg(Arg::with_name("name")
                .help("The name or ID of the note")
                .required(true)
            )
        )
        .subcommand(SubCommand::with_name("add")
            .about("Adds a new note to the zettelkasten")
            .arg(Arg::with_name("name")
                .help("The name of the new note")
                .required(true)
            )
        )
        .subcommand(SubCommand::with_name("rm")
            .about("Removes a note from the zettelkasten")
            .arg(Arg::with_name("name")
                .help("The name of the note to remove")
                .required(true)
            )
        )
        .get_matches();

    let mut settings = Settings::new();

    let notes_dir_path = Path::new(matches.value_of("directory").unwrap_or("./"));
    settings.notes_dir = notes_dir_path.as_os_str().to_os_string();
    settings.zettelkasten_dir = notes_dir_path.join(".zettelkasten").into_os_string();

    Database::set_db_path(&settings.zettelkasten_dir);

    match matches.subcommand() {
        ("init", Some(init_matches)) => exec_init_command(&init_matches, &mut settings),
        ("list", Some(list_matches)) => exec_list_command(&list_matches, &mut settings),
        ("open", Some(open_matches)) => exec_open_command(&open_matches, &mut settings),
        ("add", Some(add_matches)) => exec_add_command(&add_matches, &mut settings),
        ("rm", Some(remove_matches)) => exec_rm_command(&remove_matches, &mut settings),
        _ => (),
    }
}

fn exec_init_command (_matches: &ArgMatches, settings: &mut Settings) {
    if Directory::is_zettelkasten_dir(&settings.notes_dir, true) {
        println!("the specified directory is already a zettelkasten directory: {}",
            &settings.notes_dir.to_string_lossy());
        return;
    }
    initialize_zettelkasten(&settings.notes_dir);
}

fn initialize_zettelkasten (directory: &OsStr) {
    let notes_path = Path::new(directory);
    let zettelkasten_dir_path = notes_path.join(".zettelkasten");

    match fs::create_dir(&zettelkasten_dir_path) {
        Ok(_) => println!("initialized zettelkasten directory in '{}'", &notes_path.display()),
        Err(error) => {
            Message::error(&format!("problem creating the zettelkasten directory in '{}': {}",
                &notes_path.display(),
                error));
            return;
        }
    };

    let note_template_content = include_str!("note-template.md");
    let note_template_path = zettelkasten_dir_path.join("note-template.md");
    match fs::write(&note_template_path, note_template_content) {
        Ok(_) => {  },
        Err(error) => {
            Message::error(&format!("failed to create a note-template file in {}: {}",
                &note_template_path.to_string_lossy(), error));
            return;
        }
    };

    Database::init();
}

fn exec_list_command (matches: &ArgMatches, settings: &mut Settings) {
    if !Directory::is_zettelkasten_dir(&settings.notes_dir, false) {
        return;
    }
    Notes::list(matches.value_of("count").unwrap_or("100").parse().unwrap_or(100));
}

fn exec_open_command (matches: &ArgMatches, settings: &mut Settings) {
    if !Directory::is_zettelkasten_dir(&settings.notes_dir, false) {
        return;
    }
    let note_name = matches.value_of("name").unwrap_or_default();

    match Database::get_note_id_where(NoteProperty::NoteName, note_name) {
        Some(note_id) => Notes::open(&note_id, &settings.notes_dir),
        None => {
            Message::error(&format!("note '{}' does not exist!", note_name));
        }
    };
}

fn exec_add_command (matches: &ArgMatches, settings: &mut Settings) {
    if !Directory::is_zettelkasten_dir(&settings.notes_dir, false) {
        return;
    }
    Notes::add(matches.value_of("name").unwrap_or_default(), &settings);
}

fn exec_rm_command (matches: &ArgMatches, settings: &mut Settings) {
    if !Directory::is_zettelkasten_dir(&settings.notes_dir, false) {
        return;
    }
    Notes::remove(matches.value_of("name").unwrap_or_default(), &settings.notes_dir);
}
