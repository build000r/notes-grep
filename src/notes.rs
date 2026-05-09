use std::path::{Path, PathBuf};

use rusqlite::{Connection, OpenFlags};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::body::decode_note_body;

#[derive(Debug, Error)]
pub enum NgError {
    #[error(
        "Notes database not found at {0}. Grant Terminal/your agent Full Disk Access or pass --db PATH."
    )]
    DatabaseMissing(String),
    #[error(
        "cannot open Notes database at {path}: {source}. Grant Full Disk Access or pass --db PATH."
    )]
    DatabaseOpen {
        path: String,
        #[source]
        source: rusqlite::Error,
    },
    #[error("Notes database schema is not recognized: {0}")]
    Schema(String),
    #[error("could not locate a local data directory")]
    NoDataDir,
    #[error("failed to open note")]
    OpenFailed,
    #[error(transparent)]
    Sqlite(#[from] rusqlite::Error),
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
}

impl NgError {
    #[must_use]
    pub fn exit_code(&self) -> i32 {
        match self {
            Self::DatabaseMissing(_) | Self::DatabaseOpen { .. } => 2,
            Self::Schema(_) => 3,
            Self::OpenFailed => 4,
            Self::NoDataDir | Self::Sqlite(_) | Self::Io(_) | Self::Json(_) => 1,
        }
    }
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct StoreStats {
    pub notes: usize,
    pub folders: usize,
    pub accounts: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct NoteHit {
    pub id: String,
    pub db_id: i64,
    pub title: String,
    pub folder: Option<String>,
    pub snippet: Option<String>,
    pub modified: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct IndexedNote {
    pub id: String,
    pub db_id: i64,
    pub title: String,
    pub folder: Option<String>,
    pub snippet: Option<String>,
    pub modified: Option<String>,
    pub body: Option<String>,
}

pub struct NotesStore {
    conn: Connection,
}

pub fn default_db_path() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("~"))
        .join("Library/Group Containers/group.com.apple.notes/NoteStore.sqlite")
}

pub fn open_store(path: &Path) -> Result<NotesStore, NgError> {
    if !path.exists() {
        return Err(NgError::DatabaseMissing(path.display().to_string()));
    }
    let flags = OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_URI;
    let conn =
        Connection::open_with_flags(path, flags).map_err(|source| NgError::DatabaseOpen {
            path: path.display().to_string(),
            source,
        })?;
    verify_schema(&conn)?;
    Ok(NotesStore { conn })
}

impl NotesStore {
    pub fn stats(&self) -> Result<StoreStats, NgError> {
        Ok(StoreStats {
            notes: self.count_notes()?,
            folders: self.count_folders()?,
            accounts: self.count_accounts()?,
        })
    }

    pub fn all_indexed_notes(&self) -> Result<Vec<IndexedNote>, NgError> {
        let mut sql = base_note_query();
        sql.push_str(" ORDER BY note.ZMODIFICATIONDATE1 DESC");

        let mut stmt = self.conn.prepare(&sql)?;
        let rows = stmt.query_map([], row_to_indexed_note)?;
        rows.collect::<Result<Vec<_>, _>>().map_err(NgError::from)
    }

    pub fn search(
        &self,
        query: &str,
        folder: Option<&str>,
        limit: usize,
    ) -> Result<Vec<NoteHit>, NgError> {
        let limit = limit.clamp(1, 10_000);
        let pattern = format!("%{query}%");
        let mut sql = base_note_query();
        let mut params: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

        if !query.is_empty() {
            sql.push_str(" AND (note.ZTITLE1 LIKE ? OR COALESCE(note.ZSNIPPET, '') LIKE ?)");
            params.push(Box::new(pattern.clone()));
            params.push(Box::new(pattern));
        }
        if let Some(folder_name) = folder {
            sql.push_str(" AND folder.ZTITLE2 = ?");
            params.push(Box::new(folder_name.to_owned()));
        }
        sql.push_str(" ORDER BY note.ZMODIFICATIONDATE1 DESC LIMIT ?");
        params.push(Box::new(limit as i64));

        let mut stmt = self.conn.prepare(&sql)?;
        let params_ref: Vec<&dyn rusqlite::ToSql> = params.iter().map(|p| p.as_ref()).collect();
        let rows = stmt.query_map(params_ref.as_slice(), row_to_hit)?;
        rows.collect::<Result<Vec<_>, _>>().map_err(NgError::from)
    }

    fn count_notes(&self) -> Result<usize, NgError> {
        self.conn
            .query_row(
                "SELECT COUNT(*) FROM ZICCLOUDSYNCINGOBJECT WHERE ZTITLE1 IS NOT NULL AND COALESCE(ZMARKEDFORDELETION, 0) != 1",
                [],
                |row| row.get::<_, i64>(0),
            )
            .map(|value| value as usize)
            .map_err(NgError::from)
    }

    fn count_accounts(&self) -> Result<usize, NgError> {
        let mut predicates = Vec::new();
        for column in ["ZNAME", "ZUSERRECORDNAME", "ZACCOUNTIDENTIFIER"] {
            if self.has_column("ZICCLOUDSYNCINGOBJECT", column)? {
                predicates.push(format!("{column} IS NOT NULL"));
            }
        }

        if predicates.is_empty() {
            return Ok(0);
        }

        let sql = format!(
            "SELECT COUNT(*) FROM ZICCLOUDSYNCINGOBJECT WHERE ({}) AND COALESCE(ZMARKEDFORDELETION, 0) != 1",
            predicates.join(" OR ")
        );
        self.conn
            .query_row(&sql, [], |row| row.get::<_, i64>(0))
            .map(|value| value as usize)
            .map_err(NgError::from)
    }

    fn count_folders(&self) -> Result<usize, NgError> {
        self.conn
            .query_row(
                "SELECT COUNT(*) FROM ZICCLOUDSYNCINGOBJECT WHERE ZTITLE2 IS NOT NULL AND COALESCE(ZMARKEDFORDELETION, 0) != 1",
                [],
                |row| row.get::<_, i64>(0),
            )
            .map(|value| value as usize)
            .map_err(NgError::from)
    }

    fn has_column(&self, table: &str, column: &str) -> Result<bool, NgError> {
        let mut stmt = self.conn.prepare(&format!("PRAGMA table_info({table})"))?;
        let rows = stmt.query_map([], |row| row.get::<_, String>(1))?;
        for row in rows {
            if row? == column {
                return Ok(true);
            }
        }
        Ok(false)
    }
}

pub fn search_indexed_notes(
    notes: &[IndexedNote],
    query: &str,
    folder: Option<&str>,
    limit: usize,
) -> Vec<NoteHit> {
    let limit = limit.clamp(1, 10_000);
    notes
        .iter()
        .filter(|note| folder.is_none_or(|folder_name| note.folder.as_deref() == Some(folder_name)))
        .filter(|note| indexed_note_matches(note, query))
        .take(limit)
        .map(|note| note.to_hit(query))
        .collect()
}

impl IndexedNote {
    fn to_hit(&self, query: &str) -> NoteHit {
        NoteHit {
            id: self.id.clone(),
            db_id: self.db_id,
            title: self.title.clone(),
            folder: self.folder.clone(),
            snippet: best_snippet(self, query),
            modified: self.modified.clone(),
        }
    }
}

fn verify_schema(conn: &Connection) -> Result<(), NgError> {
    let has_object_table: bool = conn.query_row(
        "SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE type='table' AND name='ZICCLOUDSYNCINGOBJECT')",
        [],
        |row| row.get::<_, i64>(0),
    )? != 0;
    if !has_object_table {
        return Err(NgError::Schema("missing ZICCLOUDSYNCINGOBJECT".to_owned()));
    }
    Ok(())
}

fn base_note_query() -> String {
    r#"
        SELECT
            'x-coredata://' || COALESCE(zmd.Z_UUID, '') || '/ICNote/p' || note.Z_PK AS id,
            note.Z_PK AS db_id,
            note.ZTITLE1 AS title,
            folder.ZTITLE2 AS folder,
            note.ZSNIPPET AS snippet,
            datetime(note.ZMODIFICATIONDATE1 + 978307200, 'unixepoch') AS modified,
            data.ZDATA AS body_data
        FROM ZICCLOUDSYNCINGOBJECT AS note
        LEFT JOIN ZICCLOUDSYNCINGOBJECT AS folder ON note.ZFOLDER = folder.Z_PK
        LEFT JOIN ZICNOTEDATA AS data ON note.ZNOTEDATA = data.Z_PK
        LEFT JOIN Z_METADATA AS zmd ON 1=1
        WHERE note.ZTITLE1 IS NOT NULL
          AND COALESCE(note.ZMARKEDFORDELETION, 0) != 1
    "#
    .to_owned()
}

fn row_to_hit(row: &rusqlite::Row<'_>) -> rusqlite::Result<NoteHit> {
    Ok(NoteHit {
        id: row.get(0)?,
        db_id: row.get(1)?,
        title: normalized_string(row, 2)?,
        folder: normalized_optional_string(row, 3)?,
        snippet: normalized_optional_string(row, 4)?,
        modified: row.get(5)?,
    })
}

fn row_to_indexed_note(row: &rusqlite::Row<'_>) -> rusqlite::Result<IndexedNote> {
    let body_data: Option<Vec<u8>> = row.get(6)?;
    let body = body_data
        .as_deref()
        .and_then(|data| decode_note_body(data).ok().flatten());

    Ok(IndexedNote {
        id: row.get(0)?,
        db_id: row.get(1)?,
        title: normalized_string(row, 2)?,
        folder: normalized_optional_string(row, 3)?,
        snippet: normalized_optional_string(row, 4)?,
        modified: row.get(5)?,
        body,
    })
}

fn normalized_string(row: &rusqlite::Row<'_>, idx: usize) -> rusqlite::Result<String> {
    let value: String = row.get(idx)?;
    Ok(normalize_line_separators(&value))
}

fn normalized_optional_string(
    row: &rusqlite::Row<'_>,
    idx: usize,
) -> rusqlite::Result<Option<String>> {
    let value: Option<String> = row.get(idx)?;
    Ok(value.map(|text| normalize_line_separators(&text)))
}

fn indexed_note_matches(note: &IndexedNote, query: &str) -> bool {
    query.is_empty()
        || contains_case_insensitive(&note.title, query)
        || note
            .snippet
            .as_deref()
            .is_some_and(|snippet| contains_case_insensitive(snippet, query))
        || note
            .body
            .as_deref()
            .is_some_and(|body| contains_case_insensitive(body, query))
}

fn best_snippet(note: &IndexedNote, query: &str) -> Option<String> {
    if query.is_empty()
        || note
            .snippet
            .as_deref()
            .is_some_and(|snippet| contains_case_insensitive(snippet, query))
    {
        return note.snippet.clone();
    }

    note.body
        .as_deref()
        .and_then(|body| {
            body.lines()
                .find(|line| contains_case_insensitive(line, query))
        })
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(ToOwned::to_owned)
        .or_else(|| note.snippet.clone())
}

fn contains_case_insensitive(haystack: &str, needle: &str) -> bool {
    haystack.to_lowercase().contains(&needle.to_lowercase())
}

fn normalize_line_separators(text: &str) -> String {
    text.chars()
        .map(|ch| match ch {
            '\u{0085}' | '\u{2028}' | '\u{2029}' => '\n',
            _ => ch,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use flate2::Compression;
    use flate2::write::GzEncoder;
    use std::io::Write;

    fn fixture_store() -> NotesStore {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            r#"
            CREATE TABLE Z_METADATA (Z_UUID TEXT);
            INSERT INTO Z_METADATA VALUES ('FIXTURE-UUID');
            CREATE TABLE ZICCLOUDSYNCINGOBJECT (
                Z_PK INTEGER PRIMARY KEY,
                Z_ENT INTEGER,
                ZTITLE1 TEXT,
                ZTITLE2 TEXT,
                ZSNIPPET TEXT,
                ZFOLDER INTEGER,
                ZNOTEDATA INTEGER,
                ZMODIFICATIONDATE1 REAL,
                ZMARKEDFORDELETION INTEGER,
                ZNAME TEXT,
                ZUSERRECORDNAME TEXT,
                ZACCOUNTIDENTIFIER TEXT
            );
            CREATE TABLE ZICNOTEDATA (
                Z_PK INTEGER PRIMARY KEY,
                ZDATA BLOB
            );
            INSERT INTO ZICCLOUDSYNCINGOBJECT (Z_PK, Z_ENT, ZTITLE2, ZMARKEDFORDELETION) VALUES (10, 15, 'Work', 0);
            INSERT INTO ZICCLOUDSYNCINGOBJECT (Z_PK, Z_ENT, ZTITLE2, ZMARKEDFORDELETION) VALUES (11, 15, 'Personal', 0);
            INSERT INTO ZICCLOUDSYNCINGOBJECT (Z_PK, Z_ENT, ZNAME, ZUSERRECORDNAME, ZMARKEDFORDELETION) VALUES (12, 13, 'iCloud', '_fixture', 0);
            INSERT INTO ZICCLOUDSYNCINGOBJECT (Z_PK, Z_ENT, ZTITLE1, ZSNIPPET, ZFOLDER, ZNOTEDATA, ZMODIFICATIONDATE1, ZMARKEDFORDELETION)
                VALUES (1, 12, 'Stripe refund', 'Refund follow-up and receipt', 10, 101, 800000000, 0);
            INSERT INTO ZICCLOUDSYNCINGOBJECT (Z_PK, Z_ENT, ZTITLE1, ZSNIPPET, ZFOLDER, ZNOTEDATA, ZMODIFICATIONDATE1, ZMARKEDFORDELETION)
                VALUES (2, 12, 'Garden list', 'Tomatoes and irrigation', 11, 102, 700000000, 0);
            "#,
        )
        .unwrap();
        conn.execute(
            "INSERT INTO ZICNOTEDATA (Z_PK, ZDATA) VALUES (?, ?)",
            (
                101,
                fixture_body_blob("The body-only alpha phrase is not in metadata."),
            ),
        )
        .unwrap();
        conn.execute(
            "INSERT INTO ZICNOTEDATA (Z_PK, ZDATA) VALUES (?, ?)",
            (
                102,
                fixture_body_blob("The body-only beta phrase is not in metadata."),
            ),
        )
        .unwrap();
        NotesStore { conn }
    }

    #[test]
    fn search_matches_title_and_snippet() {
        let store = fixture_store();
        let hits = store.search("refund", None, 10).unwrap();
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].title, "Stripe refund");
        assert_eq!(hits[0].folder.as_deref(), Some("Work"));
    }

    #[test]
    fn search_filters_folder() {
        let store = fixture_store();
        let hits = store.search("and", Some("Personal"), 10).unwrap();
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].title, "Garden list");
    }

    #[test]
    fn stats_count_notes_and_folders() {
        let store = fixture_store();
        let stats = store.stats().unwrap();
        assert_eq!(stats.notes, 2);
        assert_eq!(stats.folders, 2);
        assert_eq!(stats.accounts, 1);
    }

    #[test]
    fn all_indexed_notes_extracts_body_text() {
        let store = fixture_store();
        let notes = store.all_indexed_notes().unwrap();
        let stripe = notes
            .iter()
            .find(|note| note.title == "Stripe refund")
            .expect("stripe note");
        assert!(
            stripe
                .body
                .as_deref()
                .unwrap_or_default()
                .contains("body-only alpha phrase")
        );
    }

    #[test]
    fn search_indexed_notes_matches_body_and_filters_folder() {
        let store = fixture_store();
        let notes = store.all_indexed_notes().unwrap();
        let hits = search_indexed_notes(&notes, "body-only", Some("Work"), 10);
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].title, "Stripe refund");
        assert!(
            hits[0]
                .snippet
                .as_deref()
                .unwrap_or_default()
                .contains("body-only alpha phrase")
        );
    }

    fn fixture_body_blob(text: &str) -> Vec<u8> {
        let mut message = Vec::new();
        push_len_field(&mut message, 1, text.as_bytes());

        let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(&message).unwrap();
        encoder.finish().unwrap()
    }

    fn push_len_field(message: &mut Vec<u8>, field: u64, bytes: &[u8]) {
        push_varint(message, (field << 3) | 2);
        push_varint(message, bytes.len() as u64);
        message.extend_from_slice(bytes);
    }

    fn push_varint(message: &mut Vec<u8>, mut value: u64) {
        while value >= 0x80 {
            message.push((value as u8 & 0x7f) | 0x80);
            value >>= 7;
        }
        message.push(value as u8);
    }
}
