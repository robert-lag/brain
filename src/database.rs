use crate::message::Message;
use crate::note::Note;
use crate::note_property::NoteProperty;
use crate::note_tagging::NoteTagging;

use chrono::prelude::*;
use lazy_static::lazy_static;
use rusqlite::{ Error, Connection, Statement, params, named_params };
use std::ffi::{ OsString, OsStr };
use std::path::Path;
use std::sync::Mutex;

lazy_static! {
    static ref DB_DIR_PATH: Mutex<OsString> = Mutex::default();
}

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

    pub fn set_db_path(db_file_path: &OsStr) {
        *DB_DIR_PATH.lock().unwrap() = db_file_path.to_os_string();
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

    pub fn get_random_note_id() -> Option<String> {
        let conn = Database::get_connection();

        let query_result = conn.query_row(
            "SELECT note_id
             FROM note
             ORDER BY RANDOM()
             LIMIT 1",
            named_params!{ },
            |row| row.get(0)
        ).ok();

        return query_result;
    }

    pub fn get_note_id_where(note_property: NoteProperty, value: &str) -> Option<String> {
        let conn = Database::get_connection();
        let query = format!(
            "SELECT note_id
             FROM note
             WHERE {} = :value",
             note_property.to_db_string()
        );

        let query_result = conn.query_row(
            &query,
            named_params!{
                ":value": value
            },
            |row| row.get(0)
        ).ok();

        return query_result;
    }

    pub fn get_note_ids_where_property_is_like(note_property: NoteProperty, value: &str) -> Vec<String> {
        let conn = Database::get_connection();
        let query = format!(
            "SELECT note_id
             FROM note
             WHERE {} LIKE ?;",
             note_property.to_db_string()
        );

        let select_statement = match conn.prepare(&query) {
            Ok(query_result) => query_result,
            Err(error) => {
                Message::error(&error.to_string());
                return Vec::new();
            }
        };

        let rows = Database::get_rows_of_prepared_query(select_statement, value);
        return rows;
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
                    creation_date_time: Local.datetime_from_str(&row.get::<usize, String>(3).unwrap(), "%Y-%m-%d %H:%M:%S").unwrap(),
                })
            }
        ).ok();

        return query_result;
    }

    pub fn get_all_recent_note_ids(count: i32) -> Vec<String> {
        let conn = Database::get_connection();

        let select_statement = match conn.prepare(
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

        let rows = Database::get_rows_of_prepared_query(select_statement, &count.to_string());
        return rows;
    }

    pub fn get_tags_of_note(note_id: &str) -> Vec<String> {
        let conn = Database::get_connection();

        let select_statement = match conn.prepare(
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

        let rows = Database::get_rows_of_prepared_query(select_statement, note_id);
        return rows;
    }

    pub fn get_note_ids_with_tag(tag_name: &str) -> Vec<String> {
        let conn = Database::get_connection();

        let select_statement = match conn.prepare(
            "SELECT note_id
             FROM note_tagging
             WHERE tag_name = ?;"
        ) {
            Ok(query_result) => query_result,
            Err(error) => {
                Message::error(&error.to_string());
                return Vec::new();
            }
        };

        let rows = Database::get_rows_of_prepared_query(select_statement, tag_name);
        return rows;
    }

    pub fn get_note_ids_with_tag_like(tag_name: &str) -> Vec<NoteTagging> {
        let conn = Database::get_connection();

        // Prepare statement
        let mut select_statement = match conn.prepare(
            "SELECT note_id, tag_name
             FROM note_tagging
             WHERE tag_name LIKE ?;"
        ) {
            Ok(query_result) => query_result,
            Err(error) => {
                Message::error(&error.to_string());
                return Vec::new();
            }
        };

        // Execute statement
        let rows = match select_statement.query_map(
            params![tag_name],
            |row| Ok((row.get_unwrap(0), row.get_unwrap(1)))
        ) {
            Ok(query_result) => query_result,
            Err(error) => {
                Message::error(&error.to_string());
                return Vec::new();
            }
        };

        // Convert rows to vector
        let mut row_vector = Vec::new();
        for row in rows {
            let row = row.unwrap();
            row_vector.push(NoteTagging::from(row.0, row.1));
        }
        return row_vector;
    }

    fn get_rows_of_prepared_query(mut statement: Statement, parameter: &str) -> Vec<String> {
        let rows = match statement.query_map(
            params![parameter],
            |row| row.get(0)
        ) {
            Ok(query_result) => query_result,
            Err(error) => {
                Message::error(&error.to_string());
                return Vec::new();
            }
        };

        // Convert rows to string vector
        let mut row_vector = Vec::new();
        for row in rows {
            row_vector.push(row.unwrap());
        }
        return row_vector;
    }

    pub fn update_note_name_where(new_note_name: &str, note_property: NoteProperty, value: &str) {
        let conn = Database::get_connection();

        let query = format!(
            "UPDATE note
             SET note_name = :new_note_name
             WHERE {} = :value",
             note_property.to_db_string()
        );

        match conn.execute(
            &query,
            named_params!{
                ":new_note_name": new_note_name,
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
                 WHERE tag_name = :tag_name",
                named_params!{
                    ":tag_name": tag_name
                },
                |row| row.get(0)
            ) {
                Ok(query_result) => query_result,
                Err(error) => {
                    if error == Error::QueryReturnedNoRows {
                        None
                    }
                    else {
                        Message::error(&format!("insert-tag: {}", &error.to_string()));
                        return;
                    }
                }
            };

            let tag_exists_already = tag_in_db.is_some();

            if !tag_exists_already {
                match conn.execute(
                    "INSERT INTO tag (tag_name)
                     VALUES (:tag_name)",
                    named_params!{
                         ":tag_name": tag_name
                    }
                ) {
                    Ok(_) => {  }
                    Err(error) => {
                        Message::error(&format!("insert-tag: {}", &error.to_string()));
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
                    if error == Error::QueryReturnedNoRows {
                        None
                    }
                    else {
                        Message::error(&format!("insert-tag: {}", &error.to_string()));
                        return;
                    }
                }
            };

            let note_tagging_exists_already = note_tagging_in_db.is_some();

            if !note_tagging_exists_already {
                match conn.execute(
                    "INSERT INTO note_tagging (tag_name, note_id)
                     VALUES (:tag_name, :note_id)",
                    named_params!{
                         ":tag_name": tag_name,
                         ":note_id": note_id
                    }
                ) {
                    Ok(_) => {  }
                    Err(error) => {
                        Message::error(&format!("insert-note-tagging: {}", &error.to_string()));
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
                Message::error(&format!("delete-note: {}", &error.to_string()));
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
                Message::error(&format!("delete-tag: {}", &error.to_string()));
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
                Message::error(&format!("delete-note-tagging: {}", &error.to_string()));
                return;
            }
        };
    }

    fn get_connection() -> Connection {
        let db_dir = &*DB_DIR_PATH.lock().unwrap();
        if db_dir.is_empty() {
            Message::error("the path of the database file has not been set!");
            panic!();
        }

        let conn = match Connection::open(Path::new(db_dir).join("data.db")) {
            Ok(connection) => connection,
            Err(error) => {
                Message::error(&error.to_string());
                panic!();
            }
        };
        return conn;
    }
}
