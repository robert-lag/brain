use crate::database::Database;
use crate::graph::graph_edge::GraphEdge;
use crate::graph::graph_node::GraphNode;
use crate::settings::Settings;

use lazy_static::lazy_static;
use regex::Regex;
use std::io::{ BufReader, BufRead };
use std::fs::{ self, File };
use std::path::PathBuf;
use string_builder::Builder;
use std::env;
use std::process::Command;

lazy_static! {
    static ref JSON_VARIABLE_VALIDATOR: Regex = Regex::new(r#"^let elementsData( = [- A-Za-z0-9"'()<>\[\]\{\}.,´`:]+)?;$"#).unwrap();
}

pub struct Graph;
impl Graph {
    pub fn generate(settings: &mut Settings) -> Result<(), String> {
        let json_string = Graph::get_json_of_notes().unwrap();

        let zettelkasten_dir = &settings.zettelkasten_dir;
        let graph_generator_file_path = PathBuf::from(zettelkasten_dir).join("graph.js");
        let file = match File::open(&graph_generator_file_path) {
            Ok(opened_file) => opened_file,
            Err(error) => return Err(format!("failed to open file {}: {}",
                                    &graph_generator_file_path.to_string_lossy(),
                                    error))
        };

        let new_content = Graph::insert_json_in_file_content(file, &json_string);
        match fs::write(&graph_generator_file_path, new_content) {
            Ok(_) => {  },
            Err(error) => return Err(format!("failed to create file {}: {}",
                                    &graph_generator_file_path.to_string_lossy(),
                                    error))
        };

        return Ok(());
    }

    fn get_json_of_notes() -> Result<String, String> {
        let mut graph_vector = Vec::new();
        let all_note_ids = Database::get_all_note_ids();
        for note_id in all_note_ids {
            if let Some(note) = Database::get_note_where_id(&note_id) {
                let node = GraphNode::from(&note_id, &note.note_name);
                graph_vector.push(r#"{ "data": "#.to_string() + &serde_json::to_string(&node).unwrap() + " }");
            } else {
                println!("failed");
            }
        }

        let all_note_links = Database::get_all_note_links();
        for note_link in all_note_links {
            let edge_id = format!("{}->{}", &note_link.source_note_id, &note_link.target_note_id);
            let edge = GraphEdge::from(&edge_id, &note_link.source_note_id, &note_link.target_note_id);
            graph_vector.push(r#"{ "data":"#.to_string() + &serde_json::to_string(&edge).unwrap() + " }");
        }

        let graph_json = "[ ".to_string() + &graph_vector.join(", ") + " ]";
        return Ok(graph_json);
    }

    fn insert_json_in_file_content(file: File, json_string: &str) -> String {
        let reader = BufReader::new(file);

        let mut new_content_builder = Builder::default();
        let mut first_iteration = true;
        for line in reader.lines() {
            let line = line.unwrap();
            if !first_iteration {
                new_content_builder.append("\n");
            }

            if JSON_VARIABLE_VALIDATOR.is_match(&line) {
                new_content_builder.append("let elementsData = JSON.parse(`".to_string() + json_string + "`);");
            } else {
                new_content_builder.append(line);
            }

            first_iteration = false;
        }

        return new_content_builder.string().unwrap();
    }

    pub fn show(settings: &mut Settings) -> Result<(), String> {
        let zettelkasten_dir = &settings.zettelkasten_dir;
        let browser = match env::var("BROWSER") {
            Ok(value) => value,
            Err(error) => {
                return Err(format!("couldn't read the BROWSER environment variable: '{}'", error));
            }
        };

        let graph_file_path = PathBuf::from(zettelkasten_dir).join("index.html");
        match Command::new(&browser).arg(&graph_file_path).status() {
            Ok(_) => {  },
            Err(error) => {
                return Err(format!("couldn't open the graph '{}' with '{}': '{}'",
                    &graph_file_path.to_string_lossy(),
                    &browser,
                    error));
            }
        };
        return Ok(());
    }
}