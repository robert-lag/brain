use crate::graph::graph_edge::GraphEdge;
use crate::graph::graph_node::GraphNode;
use crate::settings::Settings;

use std::io::{ BufReader, BufRead };
use std::fs::{ self, File };
use std::path::PathBuf;
use string_builder::Builder;
use std::env;
use std::process::Command;

pub struct Graph;
impl Graph {
    pub fn generate(settings: &mut Settings) -> Result<(), String> {
        let json_string = Graph::get_json_of_notes(settings).unwrap();

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

    fn get_json_of_notes(settings: &mut Settings) -> Result<String, String> {
        let mut vec = Vec::new();
        for i in 1..=10 {
            let id = i.to_string();
            let node = GraphNode::from(&id);
            vec.push(r#"{ "data": "#.to_string() + &serde_json::to_string(&node).unwrap() + " }");
        }
        for i in 1..=10 {
            let id = "e".to_string() + &i.to_string();
            let edge = GraphEdge::from(&id, "1", "4");
            vec.push(r#"{ "data": "#.to_string() + &serde_json::to_string(&edge).unwrap() + " }");
        }

        let json_string = "[ ".to_string() + &vec.join(", ") + " ]";
        println!("JSON: {}", json_string);
        return Ok(json_string);
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

            if line == "let elementsData;" {
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