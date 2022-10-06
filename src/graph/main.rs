use crate::database::Database;
use crate::graph::graph_edge::GraphEdge;
use crate::graph::graph_node::GraphNode;
use crate::message::Message;
use crate::settings::Settings;

use lazy_static::lazy_static;
use regex::Regex;
use std::collections::HashMap;
use std::env;
use std::fs::{self, File};
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::process::Command;
use string_builder::Builder;

lazy_static! {
    static ref JSON_VARIABLE_VALIDATOR: Regex =
        Regex::new(r#"^let elementsData( = [- A-Za-z0-9"'()<>\[\]\{\}.,´`:/\\?!]+)?;$"#).unwrap();
}

pub struct Graph;
impl Graph {
    pub fn generate(settings: &mut Settings) -> Result<(), String> {
        let json_string = Graph::get_json_of_notes().unwrap();

        let zettelkasten_dir = &settings.zettelkasten_dir;
        let graph_generator_file_path = PathBuf::from(zettelkasten_dir).join("graph.js");
        let file = match File::open(&graph_generator_file_path) {
            Ok(opened_file) => opened_file,
            Err(error) => {
                return Err(format!(
                    "failed to open file {}: {}",
                    &graph_generator_file_path.to_string_lossy(),
                    error
                ))
            }
        };

        let new_content = match Graph::insert_json_in_file_content(file, &json_string) {
            Ok(result) => result,
            Err(error) => return Err(error),
        };

        match fs::write(&graph_generator_file_path, new_content) {
            Ok(_) => {}
            Err(error) => {
                return Err(format!(
                    "failed to create file {}: {}",
                    &graph_generator_file_path.to_string_lossy(),
                    error
                ))
            }
        };

        Ok(())
    }

    fn get_json_of_notes() -> Result<String, String> {
        let mut graph_vector = Vec::new();
        let mut number_of_neighbors: HashMap<String, usize> = HashMap::new();

        // Edges
        let all_note_links = Database::get_all_note_links();
        for note_link in all_note_links {
            let edge_id = format!(
                "{}->{}",
                &note_link.source_note_id, &note_link.target_note_id
            );
            let edge = GraphEdge::from(
                &edge_id,
                &note_link.source_note_id,
                &note_link.target_note_id,
            );
            graph_vector
                .push(r#"{ "data":"#.to_string() + &serde_json::to_string(&edge).unwrap() + " }");

            *number_of_neighbors
                .entry(note_link.source_note_id)
                .or_default() += 1;
            *number_of_neighbors
                .entry(note_link.target_note_id)
                .or_default() += 1;
        }

        // Nodes
        let all_note_ids = Database::get_all_note_ids();
        for note_id in all_note_ids {
            if let Some(note) = Database::get_note_where_id(&note_id) {
                let node_weight = number_of_neighbors.get(&note_id).unwrap_or(&0);

                let node = GraphNode::from(&note_id, &note.note_name, *node_weight);
                graph_vector.push(
                    r#"{ "data": "#.to_string() + &serde_json::to_string(&node).unwrap() + " }",
                );
            } else {
                Message::warning(&format!("note '{}' not found. skipped", &note_id));
            }
        }

        let graph_json = "[ ".to_string() + &graph_vector.join(", ") + " ]";
        Ok(graph_json)
    }

    fn insert_json_in_file_content(file: File, json_string: &str) -> Result<String, String> {
        let reader = BufReader::new(file);

        let mut new_content_builder = Builder::default();
        let mut first_iteration = true;
        let mut found_json_variable = false;
        for line in reader.lines() {
            let line = line.unwrap();
            if !first_iteration {
                new_content_builder.append("\n");
            }

            if JSON_VARIABLE_VALIDATOR.is_match(&line) {
                new_content_builder
                    .append("let elementsData = JSON.parse(`".to_string() + json_string + "`);");
                found_json_variable = true;
            } else {
                new_content_builder.append(line);
            }

            first_iteration = false;
        }

        if found_json_variable {
            Ok(new_content_builder.string().unwrap())
        } else {
            Err("generate-graph: couldn't insert json in file".to_string())
        }
    }

    pub fn show(settings: &mut Settings) -> Result<(), String> {
        let zettelkasten_dir = &settings.zettelkasten_dir;
        let browser = match env::var("BROWSER") {
            Ok(value) => value,
            Err(error) => {
                return Err(format!(
                    "couldn't read the BROWSER environment variable: '{}'",
                    error
                ));
            }
        };

        let graph_file_path = PathBuf::from(zettelkasten_dir).join("index.html");
        match Command::new(&browser).arg(&graph_file_path).status() {
            Ok(_) => {}
            Err(error) => {
                return Err(format!(
                    "couldn't open the graph '{}' with '{}': '{}'",
                    &graph_file_path.to_string_lossy(),
                    &browser,
                    error
                ));
            }
        };
        Ok(())
    }
}
