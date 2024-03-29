mod brn_tui;
mod collection_tool;
mod database;
mod directory;
mod file_utility;
mod graph;
mod history;
mod message;
mod note;
mod note_link;
mod note_metadata;
mod note_property;
mod note_tagging;
mod note_type;
mod note_utility;
mod settings;

use brn_tui::main::BrnTui;
use database::Database;
use directory::Directory;
use graph::main::Graph;
use message::Message;
use note_property::NoteProperty;
use note_type::NoteType;
use note_utility::NoteUtility;
use settings::Settings;

use clap::{crate_name, crate_version, App, AppSettings, Arg, ArgMatches, SubCommand};
use include_dir::{include_dir, Dir};
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
        .arg(Arg::with_name("no-backlinking")
            .help("Don't use automatic backlinking of notes")
            .short("b")
            .long("no-backlinking")
        )
        .subcommand(SubCommand::with_name("init")
            .about("Initializes a new zettelkasten directory")
        )
        .subcommand(SubCommand::with_name("tui")
            .about("Shows an interactive TUI interface")
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
        .subcommand(SubCommand::with_name("search")
            .about("Searches notes with the specified search string")
            .arg(Arg::with_name("search-string")
                .help("The search string that is used to find notes")
                .required(true)
            )
        )
        .subcommand(SubCommand::with_name("random")
            .about("Opens a random note")
        )
        .subcommand(SubCommand::with_name("history")
            .about("Shows a history of recently visited notes")
        )
        .subcommand(SubCommand::with_name("add")
            .about("Adds a new note to the zettelkasten")
            .arg(Arg::with_name("name")
                .help("The name of the new note")
                .required(true)
            )
            .arg(Arg::with_name("topic")
                .help("Marks the note as a topic (default)")
                .short("t")
                .long("topic")
            )
            .arg(Arg::with_name("quote")
                .help("Marks the note as a quote")
                .short("q")
                .long("quote")
            )
            .arg(Arg::with_name("journal")
                .help("Marks the note as a journal note")
                .short("j")
                .long("journal")
            )
        )
        .subcommand(SubCommand::with_name("rm")
            .about("Removes a note from the zettelkasten")
            .arg(Arg::with_name("name")
                .help("The name or ID of the note to remove")
                .required(true)
            )
        )
        .subcommand(SubCommand::with_name("update-db")
            .about("Updates the database entries for all notes in the zettelkasten directory")
        )
        .subcommand(SubCommand::with_name("get-name")
            .about("Returns the name of the note with the specified id from the zettelkasten")
            .arg(Arg::with_name("id")
                .help("The ID of the note")
                .required(true)
            )
        )
        .subcommand(SubCommand::with_name("get-file-name")
            .about("Returns the name of the note file with the specified id from the zettelkasten")
            .arg(Arg::with_name("id")
                .help("The ID of the note")
                .required(true)
            )
        )
        .subcommand(SubCommand::with_name("graph")
            .about("Shows a graph virtualization of the zettelkasten in the browser.")
        )
        .get_matches();

    let notes_dir = matches
        .value_of_os("directory")
        .unwrap_or(OsStr::new("./"))
        .to_os_string();
    let notes_dir_path = Path::new(&notes_dir);
    let zettelkasten_dir = notes_dir_path.join(".zettelkasten").into_os_string();
    let mut settings = Settings::init(notes_dir, zettelkasten_dir);

    Database::set_db_path(&settings.zettelkasten_dir);

    if matches.is_present("no-backlinking") {
        settings.backlinking_enabled = false;
    } else {
        settings.backlinking_enabled = true;
    }

    match matches.subcommand() {
        ("init", Some(init_matches)) => exec_init_command(&init_matches, &mut settings),
        ("tui", Some(tui_matches)) => exec_tui_command(&tui_matches, &mut settings),
        ("list", Some(list_matches)) => exec_list_command(&list_matches, &mut settings),
        ("open", Some(open_matches)) => exec_open_command(&open_matches, &mut settings),
        ("search", Some(search_matches)) => exec_search_command(&search_matches, &mut settings),
        ("random", Some(random_matches)) => exec_random_command(&random_matches, &mut settings),
        ("history", Some(history_matches)) => exec_history_command(&history_matches, &mut settings),
        ("add", Some(add_matches)) => exec_add_command(&add_matches, &mut settings),
        ("rm", Some(remove_matches)) => exec_rm_command(&remove_matches, &mut settings),
        ("update-db", Some(update_db_matches)) => {
            exec_update_db_command(&update_db_matches, &mut settings)
        }
        ("get-name", Some(get_name_matches)) => {
            exec_get_name_command(&get_name_matches, &mut settings)
        }
        ("get-file-name", Some(get_file_name_matches)) => {
            exec_get_file_name_command(&get_file_name_matches, &mut settings)
        }
        ("graph", Some(graph_matches)) => exec_graph_command(&graph_matches, &mut settings),
        _ => (),
    }
}

fn exec_init_command(_matches: &ArgMatches, settings: &mut Settings) {
    if Directory::is_zettelkasten_dir(&settings.notes_dir, true) {
        Message::info(&format!(
            "the specified directory is already a zettelkasten directory: {}",
            &settings.notes_dir.to_string_lossy()
        ));
        return;
    }

    initialize_zettelkasten(&settings.notes_dir);
    if let Err(error) = settings
        .note_history
        .init(&settings.zettelkasten_dir.as_os_str())
    {
        Message::info(&("initializing history: ".to_string() + &error));
    }
}

fn initialize_zettelkasten(directory: &OsStr) {
    let notes_path = Path::new(directory);
    let zettelkasten_dir_path = notes_path.join(".zettelkasten");

    match fs::create_dir(&zettelkasten_dir_path) {
        Ok(_) => Message::info(&format!(
            "initialized zettelkasten directory in '{}'",
            &notes_path.display()
        )),
        Err(error) => {
            Message::error(&format!(
                "problem creating the zettelkasten directory in '{}': {}",
                &notes_path.display(),
                error
            ));
            return;
        }
    };

    let zettelkasten_dir_files = include_dir!("src/for_zettelkasten_dir");
    copy_directory_to_zettelkasten_dir(&zettelkasten_dir_files, &zettelkasten_dir_path, false);

    Database::init();
}

fn copy_directory_to_zettelkasten_dir(
    directory: &Dir,
    zettelkasten_dir_path: &Path,
    create_directory: bool,
) {
    if create_directory {
        if let Err(error) = fs::create_dir(&zettelkasten_dir_path.join(directory.path())) {
            Message::error(&format!(
                "problem creating the directory '{}': {}",
                &zettelkasten_dir_path.display(),
                error
            ));
            return;
        }
    }

    for subdirectory in directory.dirs() {
        copy_directory_to_zettelkasten_dir(&subdirectory, zettelkasten_dir_path, true);
    }

    for file in directory.files() {
        let new_file_path = zettelkasten_dir_path.join(&file.path());
        let file_content = file.contents_utf8().unwrap();
        match fs::write(&new_file_path, file_content) {
            Ok(_) => {}
            Err(error) => {
                Message::error(&format!(
                    "failed to create file in {}: {}",
                    &new_file_path.to_string_lossy(),
                    error
                ));
                return;
            }
        };
    }
}

fn exec_tui_command(_matches: &ArgMatches, settings: &mut Settings) {
    if !Directory::is_zettelkasten_dir(&settings.notes_dir, true) {
        return;
    }
    settings.print_to_stdout = false;
    BrnTui::init(settings);
}

fn exec_list_command(matches: &ArgMatches, settings: &mut Settings) {
    if !Directory::is_zettelkasten_dir(&settings.notes_dir, false) {
        return;
    }
    NoteUtility::list(
        matches
            .value_of("count")
            .unwrap_or("100")
            .parse()
            .unwrap_or(100),
    );
}

fn exec_open_command(matches: &ArgMatches, settings: &mut Settings) {
    if !Directory::is_zettelkasten_dir(&settings.notes_dir, false) {
        return;
    }
    let note_name = matches.value_of("name").unwrap_or_default();

    let result;
    match Database::get_note_id_where(NoteProperty::NoteName, note_name) {
        Some(note_id) => {
            result = NoteUtility::open(&note_id, settings);
        }
        None => {
            // Maybe the note id was given instead of the name
            let note_id = note_name;
            result = NoteUtility::open(&note_id, settings);
        }
    };

    match result {
        Ok(None) => (),
        Ok(Some(message)) => Message::warning(&message),
        Err(error) => Message::error(&error),
    }
}

fn exec_search_command(matches: &ArgMatches, settings: &mut Settings) {
    if !Directory::is_zettelkasten_dir(&settings.notes_dir, false) {
        return;
    }
    let search_string = matches.value_of("search-string").unwrap_or_default();

    let search_results = NoteUtility::search(search_string);
    NoteUtility::print_search_results(search_results)
}

fn exec_random_command(_matches: &ArgMatches, settings: &mut Settings) {
    if !Directory::is_zettelkasten_dir(&settings.notes_dir, false) {
        return;
    }

    NoteUtility::open_random_note(settings);
}

fn exec_history_command(_matches: &ArgMatches, settings: &mut Settings) {
    if !Directory::is_zettelkasten_dir(&settings.notes_dir, false) {
        return;
    }

    let note_history = NoteUtility::get_note_history(settings);
    NoteUtility::print_note_list(note_history);
}

fn exec_add_command(matches: &ArgMatches, settings: &mut Settings) {
    if !Directory::is_zettelkasten_dir(&settings.notes_dir, false) {
        return;
    }

    let note_name = matches.value_of("name").unwrap_or_default();

    let note_type;
    if matches.is_present("quote") {
        note_type = NoteType::Quote;
    } else if matches.is_present("journal") {
        note_type = NoteType::Journal;
    } else {
        note_type = NoteType::Topic;
    }

    match NoteUtility::add(note_name, note_type, settings) {
        Ok(None) => (),
        Ok(Some(note_id)) => match NoteUtility::open(&note_id, settings) {
            Ok(None) => (),
            Ok(Some(message)) => Message::warning(&message),
            Err(error) => Message::error(&error),
        },
        Err(error) => Message::error(&error),
    }
}

fn exec_rm_command(matches: &ArgMatches, settings: &mut Settings) {
    if !Directory::is_zettelkasten_dir(&settings.notes_dir, false) {
        return;
    }

    let note_name = matches.value_of("name").unwrap_or_default();

    if let Err(error) = NoteUtility::remove(note_name, &settings.notes_dir) {
        Message::error(&error);
    }
}

fn exec_update_db_command(_matches: &ArgMatches, settings: &mut Settings) {
    if !Directory::is_zettelkasten_dir(&settings.notes_dir, false) {
        return;
    }

    if let Err(error) = NoteUtility::update_db_for_all_notes_in_project_folder(settings) {
        Message::error(&error);
    }
}

fn exec_get_name_command(matches: &ArgMatches, settings: &mut Settings) {
    if !Directory::is_zettelkasten_dir(&settings.notes_dir, false) {
        return;
    }

    let note_id = matches.value_of("id").unwrap_or_default();
    NoteUtility::print_note_name_of(note_id);
}

fn exec_get_file_name_command(matches: &ArgMatches, settings: &mut Settings) {
    if !Directory::is_zettelkasten_dir(&settings.notes_dir, false) {
        return;
    }

    let note_id = matches.value_of("id").unwrap_or_default();
    NoteUtility::print_file_name_of(note_id);
}

fn exec_graph_command(_matches: &ArgMatches, settings: &mut Settings) {
    if !Directory::is_zettelkasten_dir(&settings.notes_dir, false) {
        return;
    }

    if let Err(error) = Graph::generate(settings) {
        Message::error(&error);
    } else {
        if let Err(error) = Graph::show(settings) {
            Message::error(&error);
        }
    }
}
