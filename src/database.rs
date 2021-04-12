use crate::message::Message;
use crate::note::Note;
use crate::note_property::NoteProperty;

use chrono::prelude::*;
use rusqlite::{ Connection, params, named_params };
use std::path::Path;

pub struct Database;
impl Database {
    pub fn init() {
        let conn = Database::get_connection();

        conn.execute_batch("
            BEGIN TRANSACTION;

            CREATE TABLE IF NOT EXISTS note (
                note_id varchar(20) NOT NULL,
                note_name varchar(255) NOT NULL,
                file_name varchar(50) NOT NULL,
                creation_date datetime NOT NULL,
                PRIMARY KEY (note_id)
            );

            CREATE TABLE IF NOT EXISTS tag (
                tag_name varchar(200) NOT NULL,
                PRIMARY KEY (tag_name)
            );

            CREATE TABLE IF NOT EXISTS note_tagging (
                note_id varchar(20) NOT NULL,
                tag_name varchar(200) NOT NULL,
                PRIMARY KEY (note_id, tag_name),
                FOREIGN KEY (note_id)
                    REFERENCES note (note_id),
                FOREIGN KEY (tag_name)
                    REFERENCES tag (tag_name)
            );

            COMMIT TRANSACTION;
        ").unwrap();
    }

    pub fn insert_note(note_id: &str, note_name: &str, file_name: &str, creation_date_time: DateTime<Local>) {
        let creation_timestamp = creation_date_time.format("%Y-%m-%d %H:%M:%S").to_string();

        let conn = Database::get_connection();
        conn.execute(
            "INSERT INTO note (note_id, note_name, file_name, creation_date)
             VALUES (:note_id, :note_name, :file_name, :creation_timestamp)",
            named_params!{
                ":note_id": note_id,
                ":note_name": note_name,
                ":file_name": file_name,
                ":creation_timestamp": creation_timestamp
            }
        ).unwrap();
    }

    pub fn get_note_id_where(note_property: NoteProperty, value: &str) -> Option<String> {
        let conn = Database::get_connection();
        println!("{}", note_property.to_db_string());

        let query_result = conn.query_row(
            "SELECT note_id
             FROM note
             WHERE note_name = :value",
            named_params!{
                ":value": value
            },
            |row| row.get(0)
        ).ok();

        return query_result;
    }

    pub fn get_note_where_id(note_id: &str) -> Option<Note> {
        let conn = Database::get_connection();

        let query_result = conn.query_row(
            "SELECT note_id, note_name, file_name, creation_date
             FROM note
             WHERE note_id = :note_id;",
            named_params!{
                ":note_id": note_id
            },
            |row| {
                Ok(Note {
                    note_id: row.get(0).unwrap(),
                    note_name: row.get(1).unwrap(),
                    file_name: row.get(2).unwrap(),
                    creation_date: row.get(3).unwrap(),
                })
            }
        ).ok();

        return query_result;
    }

    pub fn get_all_recent_note_ids(count: i32) -> Vec<String> {
        let conn = Database::get_connection();

        let mut select_statement = match conn.prepare(
            "SELECT note_id
             FROM note
             ORDER BY creation_date DESC
             LIMIT ?;"
        ) {
            Ok(query_result) => query_result,
            Err(error) => {
                Message::error(&error.to_string());
                return Vec::new();
            }
        };

        let mut note_names = Vec::new();
        let rows = match select_statement.query_map(
            params![count],
            |row| row.get(0)
        ) {
            Ok(query_result) => query_result,
            Err(error) => {
                Message::error(&error.to_string());
                return Vec::new();
            }
        };

        for note_name in rows {
            note_names.push(note_name.unwrap());
        }
        return note_names;
    }

    pub fn get_tags_of_note(note_id: &str) -> Vec<String> {
        let conn = Database::get_connection();

        let mut select_statement = match conn.prepare(
            "SELECT tag_name
             FROM note_tagging
             WHERE note_id = ?;"
        ) {
            Ok(query_result) => query_result,
            Err(error) => {
                Message::error(&error.to_string());
                return Vec::new();
            }
        };

        let mut tag_names = Vec::new();
        let rows = match select_statement.query_map(
            params![note_id],
            |row| row.get(0)
        ) {
            Ok(query_result) => query_result,
            Err(error) => {
                Message::error(&error.to_string());
                return Vec::new();
            }
        };

        for tag_name in rows {
            tag_names.push(tag_name.unwrap());
        }
        return tag_names;
    }

    pub fn get_note_ids_with_tag(tag_name: &str) -> Vec<String> {
        let conn = Database::get_connection();

        let mut select_statement = match conn.prepare(
            "SELECT note_id
             FROM note_tagging
             WHERE tag_name = '?';"
        ) {
            Ok(query_result) => query_result,
            Err(error) => {
                Message::error(&error.to_string());
                return Vec::new();
            }
        };

        let mut note_ids = Vec::new();
        let rows = match select_statement.query_map(
            params![tag_name],
            |row| row.get(0)
        ) {
            Ok(query_result) => query_result,
            Err(error) => {
                Message::error(&error.to_string());
                return Vec::new();
            }
        };

        for note_id in rows {
            note_ids.push(note_id.unwrap());
        }
        return note_ids;
    }

    pub fn update_note_name_where(new_note_name: &str, note_property: NoteProperty, value: &str) {
        let conn = Database::get_connection();

        match conn.execute(
            "UPDATE note
             SET note_name = ':new_note_name'
             WHERE :note_property = :value;",
            named_params!{
                ":new_note_name": new_note_name,
                ":note_property": note_property.to_db_string(),
                ":value": value
            }
        ) {
            Ok(_) => {  }
            Err(error) => {
                Message::error(&error.to_string());
                return;
            }
        };
    }

    pub fn insert_tag_for_note(tag_name: &str, note_id: &str) {
        let conn = Database::get_connection();

        insert_tag(&conn, tag_name);
        insert_note_tagging(&conn, note_id, tag_name);

        fn insert_tag(conn: &Connection, tag_name: &str) {
            let tag_in_db: Option<String> = match conn.query_row(
                "SELECT tag_name
                 FROM tag
                 WHERE tag_name = ':tag_name';",
                named_params!{
                    ":tag_name": tag_name
                },
                |row| row.get(0)
            ) {
                Ok(query_result) => query_result,
                Err(error) => {
                    Message::error(&error.to_string());
                    return;
                }
            };

            let tag_exists_already = tag_in_db.is_some();

            if !tag_exists_already {
                match conn.execute(
                    "INSERT INTO tag (tag_name)
                     VALUES (:tag_name)",
                    named_params!{
                         ":tag:name": tag_name
                    }
                ) {
                    Ok(_) => {  }
                    Err(error) => {
                        Message::error(&error.to_string());
                        return;
                    }
                };
            }
        }

        fn insert_note_tagging(conn: &Connection, note_id: &str, tag_name: &str) {
            let note_tagging_in_db: Option<String> = match conn.query_row(
                "SELECT tag_name
                 FROM note_tagging
                 WHERE tag_name = :tag_name AND note_id = :note_id",
                named_params!{
                    ":tag_name": tag_name,
                    ":note_id": note_id
                },
                |row| row.get(0)
            ) {
                Ok(query_result) => query_result,
                Err(error) => {
                    Message::error(&error.to_string());
                    return;
                }
            };

            let note_tagging_exists_already = note_tagging_in_db.is_some();

            if !note_tagging_exists_already {
                match conn.execute(
                    "INSERT INTO note_tagging (tag_name, note_id)
                     VALUES (:tag_name, :note_id)",
                    named_params!{
                         ":tag:name": tag_name,
                         ":note_id": note_id
                    }
                ) {
                    Ok(_) => {  }
                    Err(error) => {
                        Message::error(&error.to_string());
                        return;
                    }
                };
            };
        }
    }

    pub fn delete_note(note_id: &str) {
        let conn = Database::get_connection();

        match conn.execute(
            "DELETE FROM note
             WHERE note_id = :note_id",
            named_params!{
                ":note_id": note_id
            }
        ) {
            Ok(_) => {  }
            Err(error) => {
                Message::error(&error.to_string());
                return;
            }
        };
    }

    pub fn delete_tag(tag_name: &str) {
        let conn = Database::get_connection();

        match conn.execute(
            "DELETE FROM tag
             WHERE tag_name = :tag_name",
            named_params!{
                ":tag_name": tag_name
            }
        ) {
            Ok(_) => {  }
            Err(error) => {
                Message::error(&error.to_string());
                return;
            }
        };
    }

    pub fn delete_note_tagging(note_id: &str, tag_name: &str) {
        let conn = Database::get_connection();

        match conn.execute(
            "DELETE FROM note_tagging
             WHERE note_id = :note_id AND tag_name = :tag_name",
            named_params!{
                ":note_id": note_id,
                ":tag_name": tag_name
            }
        ) {
            Ok(_) => {  }
            Err(error) => {
                Message::error(&error.to_string());
                return;
            }
        };
    }

    fn get_connection() -> Connection {
        let conn = match Connection::open(Path::new(".zettelkasten").join("data.db")) {
            Ok(connection) => connection,
            Err(error) => {
                Message::error(&error.to_string());
                panic!();
            }
        };
        return conn;
    }
}
