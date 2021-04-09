use chrono::prelude::*;
use mysql::*;
use mysql::prelude::*;

pub struct Database;
impl Database {
    pub fn insert_note(note_id: &str, note_name: &str, file_name: &str, creation_date_time: DateTime<Local>) {
        let creation_timestamp = creation_date_time.format("%Y-%m-%d %H:%M:%S").to_string();

        let mut conn = Database::get_connection();
        conn.exec_drop(
            r"INSERT INTO note (note_id, note_name, file_name, creation_date)
              VALUES (:note_id, :note_name, :file_name, :creation_timestamp)",
            (note_id, note_name, file_name, creation_timestamp)
        ).unwrap();
    }

    pub fn get_note_id_where(where_statement: &str) -> Option<String> {
        let mut conn = Database::get_connection();

        let query = format!(r"SELECT note_id
              FROM note
              WHERE {}",
              where_statement
        );

        let query_result = conn.query_first(&query).unwrap();
        return query_result
    }

    pub fn get_note_name_where(where_statement: &str) -> Option<String> {
        let mut conn = Database::get_connection();

        let query = format!(r"SELECT note_name
              FROM note
              WHERE {}",
              where_statement
        );

        let query_result = conn.query_first(&query).unwrap();

        return query_result;
    }

    pub fn get_file_name_where(where_statement: &str) -> Option<String> {
        let mut conn = Database::get_connection();

        let query = format!(r"SELECT file_name
              FROM note
              WHERE {}",
              where_statement
        );

        let query_result = conn.query_first(&query).unwrap();

        return query_result;
    }

    pub fn get_all_recent_note_names(count: i32) -> Vec<String> {
        let mut conn = Database::get_connection();

        let query = format!(r"SELECT note_name
              FROM note
              ORDER BY creation_date DESC
              LIMIT {};",
              count
        );

        let query_result = conn.query(&query).unwrap();

        return query_result;
    }

    pub fn get_tags_of_note(note_id: &str) -> Vec<String> {
        let mut conn = Database::get_connection();

        let query = format!(r"SELECT tag_name
              FROM note_tagging
              WHERE note_id = '{}'",
              note_id
        );

        let query_result = conn.query(&query).unwrap();

        return query_result;
    }

    pub fn get_note_ids_with_tag(tag_name: &str) -> Vec<String> {
        let mut conn = Database::get_connection();

        let query = format!(r"SELECT note_id
              FROM note_tagging
              WHERE tag_name = '{}'",
              tag_name
        );

        let query_result = conn.query(&query).unwrap();

        return query_result;
    }

    pub fn update_note_name_where(new_note_name: &str, where_statement: &str) {
        let mut conn = Database::get_connection();

        let query = format!(
            r"UPDATE note
              SET note_name = '{}'
              WHERE {};",
            new_note_name, where_statement
        );

        conn.exec_drop(query, ()).unwrap();
    }

    pub fn insert_tag_for_note(tag_name: &str, note_id: &str) {
        let mut conn = Database::get_connection();

        // Insert tag
        let get_tag_entry = format!(r"SELECT tag_name
              FROM tag
              WHERE tag_name = '{}'",
              tag_name
        );
        match conn.query_first::<String, &str>(&get_tag_entry).unwrap() {
            Some(_) => {
                // Tag entry already exists
            },
            None => {
                conn.exec_drop(
                    r"INSERT INTO tag (tag_name)
                      VALUES (:tag_name)",
                      (tag_name, )
                ).unwrap();
            }
        };

        // Insert connection between note and tag
        let get_note_tagging_entry = format!(r"SELECT tag_name
              FROM note_tagging
              WHERE tag_name = '{}' AND note_id = '{}'",
              tag_name, note_id
        );
        match conn.query_first::<String, &str>(&get_note_tagging_entry).unwrap() {
            Some(_) => {
                // Note tagging entry already exists
            },
            None => {
                conn.exec_drop(
                    r"INSERT INTO note_tagging (tag_name, note_id)
                      VALUES (:tag_name, :note_id)",
                    (tag_name, note_id)
                ).unwrap();
            }
        };
    }

    pub fn delete_note(note_id: &str) {
        let mut conn = Database::get_connection();

        let query = format!(
            r"DELETE FROM note
              WHERE note_id = '{}';",
            note_id
        );

        conn.exec_drop(query, ()).unwrap();
    }

    pub fn delete_tag(tag_name: &str) {
        let mut conn = Database::get_connection();

        let query = format!(
            r"DELETE FROM tag
              WHERE tag_name = '{}';",
            tag_name
        );

        conn.exec_drop(query, ()).unwrap();
    }

    pub fn delete_note_tagging(note_id: &str, tag_name: &str) {
        let mut conn = Database::get_connection();

        let query = format!(
            r"DELETE FROM note_tagging
              WHERE note_id = '{}' AND tag_name = '{}';",
            note_id, tag_name
        );

        conn.exec_drop(query, ()).unwrap();
    }

    fn get_connection() -> PooledConn  {
        let url = "mysql://root:password@localhost:3306/brain";

        let pool = Pool::new(url).unwrap();
        let conn = pool.get_conn().unwrap();

        return conn;
    }
}
