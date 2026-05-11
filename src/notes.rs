use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

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
    #[error("{0}")]
    Command(String),
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
            Self::NoDataDir | Self::Command(_) | Self::Sqlite(_) | Self::Io(_) | Self::Json(_) => 1,
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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub folder_path: Option<String>,
    pub snippet: Option<String>,
    pub modified: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct IndexedNote {
    pub id: String,
    pub db_id: i64,
    pub title: String,
    pub folder: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub folder_path: Option<String>,
    pub snippet: Option<String>,
    pub modified: Option<String>,
    pub body: Option<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct FolderEntry {
    pub id: i64,
    pub name: String,
    pub parent_id: Option<i64>,
    pub account_id: Option<i64>,
    pub account: Option<String>,
    pub path: String,
    pub account_path: String,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct FolderMovePlan {
    pub source_id: i64,
    pub source_path: String,
    pub target_parent_id: Option<i64>,
    pub target_account_id: Option<i64>,
    pub target_path: String,
    pub new_name: String,
    pub descendant_folders: usize,
    pub notes: usize,
    pub will_change: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct NoteMovePlan {
    pub note_id: String,
    pub note_db_id: i64,
    pub note_title: String,
    pub source_folder_id: i64,
    pub source_folder_path: String,
    pub target_folder_id: i64,
    pub target_folder_path: String,
    pub will_change: bool,
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

pub fn open_store_for_writing(path: &Path) -> Result<NotesStore, NgError> {
    if !path.exists() {
        return Err(NgError::DatabaseMissing(path.display().to_string()));
    }
    let flags = OpenFlags::SQLITE_OPEN_READ_WRITE | OpenFlags::SQLITE_OPEN_URI;
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
        let folder_paths = self.folder_path_map()?;

        let mut stmt = self.conn.prepare(&sql)?;
        let rows = stmt.query_map([], row_to_indexed_note)?;
        let mut notes = rows.collect::<Result<Vec<_>, _>>()?;
        for note in &mut notes {
            note.note.folder_path = note
                .folder_id
                .and_then(|folder_id| folder_paths.get(&folder_id).cloned());
        }
        Ok(notes.into_iter().map(|note| note.note).collect())
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
        sql.push_str(" ORDER BY note.ZMODIFICATIONDATE1 DESC");
        if folder.is_none() {
            sql.push_str(" LIMIT ?");
            params.push(Box::new(limit as i64));
        }

        let folder_paths = self.folder_path_map()?;
        let mut stmt = self.conn.prepare(&sql)?;
        let params_ref: Vec<&dyn rusqlite::ToSql> = params.iter().map(|p| p.as_ref()).collect();
        let rows = stmt.query_map(params_ref.as_slice(), row_to_hit)?;
        let mut hits = rows.collect::<Result<Vec<_>, _>>()?;
        for hit in &mut hits {
            hit.note.folder_path = hit
                .folder_id
                .and_then(|folder_id| folder_paths.get(&folder_id).cloned());
        }
        let hits = hits
            .into_iter()
            .map(|hit| hit.note)
            .filter(|hit| folder.is_none_or(|folder_name| note_hit_in_folder(hit, folder_name)))
            .take(limit)
            .collect();
        Ok(hits)
    }

    pub fn folders(&self) -> Result<Vec<FolderEntry>, NgError> {
        self.ensure_folder_read_schema()?;
        let raw = self.raw_folders()?;
        Ok(build_folder_entries(&raw))
    }

    pub fn plan_folder_move(
        &self,
        source_path: &str,
        target_path: &str,
    ) -> Result<FolderMovePlan, NgError> {
        self.ensure_folder_read_schema()?;
        let raw = self.raw_folders()?;
        let folders = build_folder_entries(&raw);
        let source = resolve_folder(&folders, source_path)?;
        let target = resolve_target(&folders, &source, target_path)?;

        if source.account_id != target.account_id {
            return Err(NgError::Command(format!(
                "cross-account folder moves are not supported: '{}' -> '{}'",
                source.account_path, target.path
            )));
        }

        if target.parent_id == Some(source.id)
            || parent_chain_contains(&raw, target.parent_id, source.id)
        {
            return Err(NgError::Command(format!(
                "cannot move '{}' into itself or one of its descendants",
                source.account_path
            )));
        }

        if sibling_exists(
            &folders,
            source.id,
            target.account_id,
            target.parent_id,
            &target.name,
        ) {
            return Err(NgError::Command(format!(
                "target sibling already exists: '{}'",
                target.path
            )));
        }

        let descendant_ids = descendant_folder_ids(&raw, source.id);
        let mut note_folder_ids = descendant_ids.clone();
        note_folder_ids.push(source.id);
        let notes = self.count_notes_in_folders(&note_folder_ids)?;
        let will_change = source.parent_id != target.parent_id
            || source.account_id != target.account_id
            || source.name != target.name;

        Ok(FolderMovePlan {
            source_id: source.id,
            source_path: source.account_path,
            target_parent_id: target.parent_id,
            target_account_id: target.account_id,
            target_path: target.path,
            new_name: target.name,
            descendant_folders: descendant_ids.len(),
            notes,
            will_change,
        })
    }

    pub fn apply_folder_move(&mut self, plan: &FolderMovePlan) -> Result<(), NgError> {
        if !plan.will_change {
            return Ok(());
        }

        self.ensure_folder_write_schema()?;
        let now = apple_reference_now();
        let tx = self.conn.transaction()?;
        let updated = tx.execute(
            r#"
            UPDATE ZICCLOUDSYNCINGOBJECT
            SET ZTITLE2 = ?1,
                ZPARENT = ?2,
                ZACCOUNT8 = ?3,
                ZPARENTMODIFICATIONDATE = ?4,
                Z_OPT = COALESCE(Z_OPT, 0) + 1
            WHERE Z_PK = ?5
              AND ZTITLE2 IS NOT NULL
              AND COALESCE(ZMARKEDFORDELETION, 0) != 1
            "#,
            (
                &plan.new_name,
                plan.target_parent_id,
                plan.target_account_id,
                now,
                plan.source_id,
            ),
        )?;
        if updated != 1 {
            return Err(NgError::Command(format!(
                "folder move expected to update 1 row, updated {updated}"
            )));
        }
        tx.commit()?;
        Ok(())
    }

    pub fn plan_note_move(
        &self,
        note_id: &str,
        target_folder_path: &str,
    ) -> Result<NoteMovePlan, NgError> {
        self.ensure_note_move_read_schema()?;
        self.ensure_folder_read_schema()?;

        let raw = self.raw_folders()?;
        let folders = build_folder_entries(&raw);
        let folders_by_id = folders
            .iter()
            .map(|folder| (folder.id, folder))
            .collect::<HashMap<_, _>>();
        let note = self.resolve_active_note(note_id)?;
        let source_folder = folders_by_id.get(&note.folder_id).ok_or_else(|| {
            NgError::Command(format!(
                "active source folder not found for note '{}'",
                note.id
            ))
        })?;
        let target_folder = resolve_folder(&folders, target_folder_path)?;

        if source_folder.account_id != target_folder.account_id {
            return Err(NgError::Command(format!(
                "cross-account note moves are not supported: '{}' -> '{}'",
                source_folder.account_path, target_folder.account_path
            )));
        }

        Ok(NoteMovePlan {
            note_id: note.id,
            note_db_id: note.db_id,
            note_title: note.title,
            source_folder_id: source_folder.id,
            source_folder_path: source_folder.account_path.clone(),
            target_folder_id: target_folder.id,
            target_folder_path: target_folder.account_path,
            will_change: source_folder.id != target_folder.id,
        })
    }

    pub fn apply_note_move(&mut self, plan: &NoteMovePlan) -> Result<(), NgError> {
        if !plan.will_change {
            return Ok(());
        }

        self.ensure_note_move_write_schema()?;
        let now = apple_reference_now();
        let tx = self.conn.transaction()?;
        let updated = tx.execute(
            r#"
            UPDATE ZICCLOUDSYNCINGOBJECT
            SET ZFOLDER = ?1,
                ZFOLDERMODIFICATIONDATE = ?2,
                Z_OPT = COALESCE(Z_OPT, 0) + 1
            WHERE Z_PK = ?3
              AND ZTITLE1 IS NOT NULL
              AND ZFOLDER = ?4
              AND COALESCE(ZMARKEDFORDELETION, 0) != 1
              AND EXISTS (
                  SELECT 1
                  FROM ZICCLOUDSYNCINGOBJECT AS source_folder
                  JOIN ZICCLOUDSYNCINGOBJECT AS target_folder ON target_folder.Z_PK = ?1
                  WHERE source_folder.Z_PK = ?4
                    AND source_folder.ZTITLE2 IS NOT NULL
                    AND target_folder.ZTITLE2 IS NOT NULL
                    AND COALESCE(source_folder.ZMARKEDFORDELETION, 0) != 1
                    AND COALESCE(target_folder.ZMARKEDFORDELETION, 0) != 1
                    AND COALESCE(source_folder.ZACCOUNT8, -1) = COALESCE(target_folder.ZACCOUNT8, -1)
              )
            "#,
            (
                plan.target_folder_id,
                now,
                plan.note_db_id,
                plan.source_folder_id,
            ),
        )?;
        if updated != 1 {
            return Err(NgError::Command(format!(
                "note move expected to update 1 row, updated {updated}"
            )));
        }
        tx.commit()?;
        Ok(())
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

    fn ensure_folder_read_schema(&self) -> Result<(), NgError> {
        for column in [
            "Z_PK",
            "ZTITLE2",
            "ZPARENT",
            "ZACCOUNT8",
            "ZMARKEDFORDELETION",
        ] {
            if !self.has_column("ZICCLOUDSYNCINGOBJECT", column)? {
                return Err(NgError::Schema(format!(
                    "missing folder column ZICCLOUDSYNCINGOBJECT.{column}"
                )));
            }
        }
        Ok(())
    }

    fn ensure_folder_write_schema(&self) -> Result<(), NgError> {
        self.ensure_folder_read_schema()?;
        for column in ["Z_OPT", "ZPARENTMODIFICATIONDATE"] {
            if !self.has_column("ZICCLOUDSYNCINGOBJECT", column)? {
                return Err(NgError::Schema(format!(
                    "missing folder column ZICCLOUDSYNCINGOBJECT.{column}"
                )));
            }
        }
        Ok(())
    }

    fn ensure_note_move_read_schema(&self) -> Result<(), NgError> {
        for column in ["Z_PK", "ZTITLE1", "ZFOLDER", "ZMARKEDFORDELETION"] {
            if !self.has_column("ZICCLOUDSYNCINGOBJECT", column)? {
                return Err(NgError::Schema(format!(
                    "missing note column ZICCLOUDSYNCINGOBJECT.{column}"
                )));
            }
        }
        Ok(())
    }

    fn ensure_note_move_write_schema(&self) -> Result<(), NgError> {
        self.ensure_note_move_read_schema()?;
        for column in ["Z_OPT", "ZFOLDERMODIFICATIONDATE"] {
            if !self.has_column("ZICCLOUDSYNCINGOBJECT", column)? {
                return Err(NgError::Schema(format!(
                    "missing note column ZICCLOUDSYNCINGOBJECT.{column}"
                )));
            }
        }
        Ok(())
    }

    fn raw_folders(&self) -> Result<HashMap<i64, RawFolder>, NgError> {
        let mut stmt = self.conn.prepare(
            r#"
            SELECT
                folder.Z_PK,
                folder.ZTITLE2,
                folder.ZPARENT,
                folder.ZACCOUNT8,
                account.ZNAME
            FROM ZICCLOUDSYNCINGOBJECT AS folder
            LEFT JOIN ZICCLOUDSYNCINGOBJECT AS account ON folder.ZACCOUNT8 = account.Z_PK
            WHERE folder.ZTITLE2 IS NOT NULL
              AND COALESCE(folder.ZMARKEDFORDELETION, 0) != 1
            "#,
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(RawFolder {
                id: row.get(0)?,
                name: normalized_string(row, 1)?,
                parent_id: row.get(2)?,
                account_id: row.get(3)?,
                account: normalized_optional_string(row, 4)?,
            })
        })?;

        let mut folders = HashMap::new();
        for row in rows {
            let folder = row?;
            folders.insert(folder.id, folder);
        }
        Ok(folders)
    }

    fn folder_path_map(&self) -> Result<HashMap<i64, String>, NgError> {
        if !self.has_column("ZICCLOUDSYNCINGOBJECT", "ZPARENT")?
            || !self.has_column("ZICCLOUDSYNCINGOBJECT", "ZACCOUNT8")?
        {
            return Ok(HashMap::new());
        }
        let raw = self.raw_folders()?;
        Ok(build_folder_entries(&raw)
            .into_iter()
            .map(|folder| (folder.id, folder.path))
            .collect())
    }

    fn count_notes_in_folders(&self, folder_ids: &[i64]) -> Result<usize, NgError> {
        if folder_ids.is_empty() {
            return Ok(0);
        }

        let placeholders = std::iter::repeat_n("?", folder_ids.len())
            .collect::<Vec<_>>()
            .join(", ");
        let sql = format!(
            "SELECT COUNT(*) FROM ZICCLOUDSYNCINGOBJECT WHERE ZTITLE1 IS NOT NULL AND COALESCE(ZMARKEDFORDELETION, 0) != 1 AND ZFOLDER IN ({placeholders})"
        );
        let params = rusqlite::params_from_iter(folder_ids.iter());
        self.conn
            .query_row(&sql, params, |row| row.get::<_, i64>(0))
            .map(|value| value as usize)
            .map_err(NgError::from)
    }

    fn resolve_active_note(&self, note_id: &str) -> Result<ResolvedNote, NgError> {
        let trimmed = note_id.trim();
        if trimmed.is_empty() {
            return Err(NgError::Command("note ID cannot be empty".to_owned()));
        }

        let db_id = if let Some(db_id) = parse_coredata_note_db_id(trimmed) {
            db_id
        } else {
            parse_numeric_note_db_id(trimmed)?
        };

        let Some(note) = self.active_note_by_db_id(db_id)? else {
            return Err(NgError::Command(format!("note not found: '{trimmed}'")));
        };

        if trimmed.starts_with("x-coredata://") && note.id != trimmed {
            return Err(NgError::Command(format!("note not found: '{trimmed}'")));
        }

        Ok(note)
    }

    fn active_note_by_db_id(&self, db_id: i64) -> Result<Option<ResolvedNote>, NgError> {
        let mut stmt = self.conn.prepare(
            r#"
            SELECT
                'x-coredata://' || COALESCE((SELECT Z_UUID FROM Z_METADATA LIMIT 1), '') || '/ICNote/p' || note.Z_PK AS id,
                note.Z_PK AS db_id,
                note.ZTITLE1 AS title,
                note.ZFOLDER AS folder_id
            FROM ZICCLOUDSYNCINGOBJECT AS note
            WHERE note.Z_PK = ?1
              AND note.ZTITLE1 IS NOT NULL
              AND COALESCE(note.ZMARKEDFORDELETION, 0) != 1
            "#,
        )?;
        let mut rows = stmt.query([db_id])?;
        let Some(row) = rows.next()? else {
            return Ok(None);
        };
        Ok(Some(ResolvedNote {
            id: row.get(0)?,
            db_id: row.get(1)?,
            title: normalized_string(row, 2)?,
            folder_id: row.get(3)?,
        }))
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
        .filter(|note| folder.is_none_or(|folder_name| indexed_note_in_folder(note, folder_name)))
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
            folder_path: self.folder_path.clone(),
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
    // Z_UUID is read via a scalar subquery (not a cross-join) so a Z_METADATA
    // table containing zero or more than one row still yields exactly one
    // result row per note. Matches the pattern used in active_note_by_db_id.
    r#"
        SELECT
            'x-coredata://' || COALESCE((SELECT Z_UUID FROM Z_METADATA LIMIT 1), '') || '/ICNote/p' || note.Z_PK AS id,
            note.Z_PK AS db_id,
            note.ZTITLE1 AS title,
            folder.Z_PK AS folder_id,
            folder.ZTITLE2 AS folder,
            note.ZSNIPPET AS snippet,
            datetime(note.ZMODIFICATIONDATE1 + 978307200, 'unixepoch') AS modified,
            data.ZDATA AS body_data
        FROM ZICCLOUDSYNCINGOBJECT AS note
        LEFT JOIN ZICCLOUDSYNCINGOBJECT AS folder ON note.ZFOLDER = folder.Z_PK
        LEFT JOIN ZICNOTEDATA AS data ON note.ZNOTEDATA = data.Z_PK
        WHERE note.ZTITLE1 IS NOT NULL
          AND COALESCE(note.ZMARKEDFORDELETION, 0) != 1
    "#
    .to_owned()
}

struct RawNoteHit {
    note: NoteHit,
    folder_id: Option<i64>,
}

struct RawIndexedNote {
    note: IndexedNote,
    folder_id: Option<i64>,
}

fn row_to_hit(row: &rusqlite::Row<'_>) -> rusqlite::Result<RawNoteHit> {
    Ok(RawNoteHit {
        note: NoteHit {
            id: row.get(0)?,
            db_id: row.get(1)?,
            title: normalized_string(row, 2)?,
            folder: normalized_optional_string(row, 4)?,
            folder_path: None,
            snippet: normalized_optional_string(row, 5)?,
            modified: row.get(6)?,
        },
        folder_id: row.get(3)?,
    })
}

fn row_to_indexed_note(row: &rusqlite::Row<'_>) -> rusqlite::Result<RawIndexedNote> {
    let body_data: Option<Vec<u8>> = row.get(7)?;
    let body = body_data
        .as_deref()
        .and_then(|data| decode_note_body(data).ok().flatten());

    Ok(RawIndexedNote {
        note: IndexedNote {
            id: row.get(0)?,
            db_id: row.get(1)?,
            title: normalized_string(row, 2)?,
            folder: normalized_optional_string(row, 4)?,
            folder_path: None,
            snippet: normalized_optional_string(row, 5)?,
            modified: row.get(6)?,
            body,
        },
        folder_id: row.get(3)?,
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

fn indexed_note_in_folder(note: &IndexedNote, folder: &str) -> bool {
    note.folder.as_deref() == Some(folder) || note.folder_path.as_deref() == Some(folder)
}

fn note_hit_in_folder(note: &NoteHit, folder: &str) -> bool {
    note.folder.as_deref() == Some(folder) || note.folder_path.as_deref() == Some(folder)
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

#[derive(Debug, Clone)]
struct RawFolder {
    id: i64,
    name: String,
    parent_id: Option<i64>,
    account_id: Option<i64>,
    account: Option<String>,
}

#[derive(Debug, Clone)]
struct ResolvedTarget {
    parent_id: Option<i64>,
    account_id: Option<i64>,
    name: String,
    path: String,
}

#[derive(Debug, Clone)]
struct ResolvedParent {
    id: Option<i64>,
    account_id: Option<i64>,
    path: Option<String>,
}

#[derive(Debug, Clone)]
struct ResolvedNote {
    id: String,
    db_id: i64,
    title: String,
    folder_id: i64,
}

fn build_folder_entries(raw: &HashMap<i64, RawFolder>) -> Vec<FolderEntry> {
    let mut memo = HashMap::new();
    let mut entries = raw
        .values()
        .map(|folder| {
            let path = folder_path(folder.id, raw, &mut memo, &mut HashSet::new());
            let account_label = account_label(folder);
            let account_path = format!("{account_label}/{path}");
            FolderEntry {
                id: folder.id,
                name: folder.name.clone(),
                parent_id: folder.parent_id,
                account_id: folder.account_id,
                account: folder.account.clone(),
                path,
                account_path,
            }
        })
        .collect::<Vec<_>>();
    entries.sort_by(|left, right| {
        left.account_path
            .to_lowercase()
            .cmp(&right.account_path.to_lowercase())
    });
    entries
}

fn folder_path(
    id: i64,
    raw: &HashMap<i64, RawFolder>,
    memo: &mut HashMap<i64, String>,
    stack: &mut HashSet<i64>,
) -> String {
    if let Some(path) = memo.get(&id) {
        return path.clone();
    }

    let Some(folder) = raw.get(&id) else {
        return id.to_string();
    };
    if !stack.insert(id) {
        return folder.name.clone();
    }

    let path = folder
        .parent_id
        .and_then(|parent_id| raw.get(&parent_id).map(|_| parent_id))
        .map(|parent_id| {
            format!(
                "{}/{}",
                folder_path(parent_id, raw, memo, stack),
                folder.name
            )
        })
        .unwrap_or_else(|| folder.name.clone());
    stack.remove(&id);
    memo.insert(id, path.clone());
    path
}

fn account_label(folder: &RawFolder) -> String {
    folder
        .account
        .clone()
        .filter(|account| !account.trim().is_empty())
        .or_else(|| folder.account_id.map(|id| format!("account:{id}")))
        .unwrap_or_else(|| "unknown-account".to_owned())
}

fn normalize_path_arg(path: &str) -> Result<String, NgError> {
    let path = path.trim().trim_matches('/');
    let parts = path
        .split('/')
        .map(str::trim)
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>();
    if parts.is_empty() {
        return Err(NgError::Command("folder path cannot be empty".to_owned()));
    }
    Ok(parts.join("/"))
}

fn split_path_arg(path: &str) -> Result<Vec<String>, NgError> {
    Ok(normalize_path_arg(path)?
        .split('/')
        .map(ToOwned::to_owned)
        .collect())
}

fn parse_coredata_note_db_id(note_id: &str) -> Option<i64> {
    if !note_id.starts_with("x-coredata://") {
        return None;
    }
    let (_, suffix) = note_id.rsplit_once("/ICNote/p")?;
    if suffix.is_empty() || !suffix.chars().all(|ch| ch.is_ascii_digit()) {
        return None;
    }
    suffix.parse().ok()
}

fn parse_numeric_note_db_id(note_id: &str) -> Result<i64, NgError> {
    if !note_id.chars().all(|ch| ch.is_ascii_digit()) {
        return Err(NgError::Command(
            "note ID must be an x-coredata://.../ICNote/p... ID or an unambiguous numeric database ID"
                .to_owned(),
        ));
    }
    note_id.parse().map_err(|_| {
        NgError::Command(format!(
            "numeric note database ID is out of range: '{note_id}'"
        ))
    })
}

fn resolve_folder(folders: &[FolderEntry], path: &str) -> Result<FolderEntry, NgError> {
    let path = normalize_path_arg(path)?;
    let matches = folders
        .iter()
        .filter(|folder| folder.path == path || folder.account_path == path)
        .cloned()
        .collect::<Vec<_>>();

    match matches.as_slice() {
        [folder] => Ok(folder.clone()),
        [] => Err(NgError::Command(format!("folder path not found: '{path}'"))),
        _ => Err(NgError::Command(format!(
            "folder path is ambiguous: '{path}'. Use an account-prefixed path such as '{}'.",
            matches[0].account_path
        ))),
    }
}

fn resolve_target(
    folders: &[FolderEntry],
    source: &FolderEntry,
    target_path: &str,
) -> Result<ResolvedTarget, NgError> {
    let parts = split_path_arg(target_path)?;
    let Some(name) = parts.last().cloned() else {
        return Err(NgError::Command(
            "target folder path cannot be empty".to_owned(),
        ));
    };
    if name.contains(':') && name.starts_with("account:") {
        return Err(NgError::Command(
            "target folder name cannot be an account placeholder".to_owned(),
        ));
    }

    let parent_parts = &parts[..parts.len() - 1];
    let parent = if parent_parts.is_empty() {
        ResolvedParent {
            id: source.parent_id,
            account_id: source.account_id,
            path: parent_path_for_source(folders, source),
        }
    } else {
        resolve_target_parent(folders, source, parent_parts)?
    };
    let path = match parent.path {
        Some(parent_path) => format!("{parent_path}/{name}"),
        None => name.clone(),
    };

    Ok(ResolvedTarget {
        parent_id: parent.id,
        account_id: parent.account_id,
        name,
        path,
    })
}

fn parent_path_for_source(folders: &[FolderEntry], source: &FolderEntry) -> Option<String> {
    source
        .parent_id
        .and_then(|parent_id| {
            folders
                .iter()
                .find(|folder| folder.id == parent_id)
                .map(|folder| folder.account_path.clone())
        })
        .or_else(|| source.account.clone())
}

fn resolve_target_parent(
    folders: &[FolderEntry],
    source: &FolderEntry,
    parent_parts: &[String],
) -> Result<ResolvedParent, NgError> {
    let parent_path = parent_parts.join("/");

    if let Ok(parent) = resolve_folder(folders, &parent_path) {
        return Ok(ResolvedParent {
            id: Some(parent.id),
            account_id: parent.account_id,
            path: Some(parent.account_path),
        });
    }

    let mut seen_accounts = HashSet::new();
    let accounts = folders
        .iter()
        .filter_map(|folder| {
            let account = folder.account.as_ref()?;
            let key = (account.clone(), folder.account_id);
            if seen_accounts.insert(key.clone()) {
                Some(key)
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    let account_matches = accounts
        .iter()
        .filter(|(account, _)| account == &parent_path)
        .collect::<Vec<_>>();

    match account_matches.as_slice() {
        [(_, account_id)] => Ok(ResolvedParent {
            id: None,
            account_id: *account_id,
            path: Some(parent_path),
        }),
        [] if parent_path == "." => Ok(ResolvedParent {
            id: source.parent_id,
            account_id: source.account_id,
            path: None,
        }),
        [] => Err(NgError::Command(format!(
            "target parent folder path not found: '{parent_path}'"
        ))),
        _ => Err(NgError::Command(format!(
            "target parent account is ambiguous: '{parent_path}'"
        ))),
    }
}

fn parent_chain_contains(
    raw: &HashMap<i64, RawFolder>,
    mut parent_id: Option<i64>,
    needle: i64,
) -> bool {
    let mut seen = HashSet::new();
    while let Some(id) = parent_id {
        if id == needle {
            return true;
        }
        if !seen.insert(id) {
            return false;
        }
        parent_id = raw.get(&id).and_then(|folder| folder.parent_id);
    }
    false
}

fn descendant_folder_ids(raw: &HashMap<i64, RawFolder>, source_id: i64) -> Vec<i64> {
    // Apple Notes shouldn't produce parent cycles, but iCloud merge artifacts
    // and corrupt stores can. `seen` keeps a cycle from spinning this DFS into
    // an unbounded loop and an unbounded `Vec` (which would later blow up
    // `count_notes_in_folders`'s SQL parameter list). The traversal still
    // visits each reachable descendant exactly once.
    let mut out = Vec::new();
    let mut seen = HashSet::new();
    seen.insert(source_id);
    let mut stack = vec![source_id];
    while let Some(parent_id) = stack.pop() {
        for folder in raw
            .values()
            .filter(|folder| folder.parent_id == Some(parent_id))
        {
            if seen.insert(folder.id) {
                out.push(folder.id);
                stack.push(folder.id);
            }
        }
    }
    out
}

fn sibling_exists(
    folders: &[FolderEntry],
    source_id: i64,
    account_id: Option<i64>,
    parent_id: Option<i64>,
    name: &str,
) -> bool {
    folders.iter().any(|folder| {
        folder.id != source_id
            && folder.account_id == account_id
            && folder.parent_id == parent_id
            && folder.name.eq_ignore_ascii_case(name)
    })
}

fn apple_reference_now() -> f64 {
    const APPLE_REFERENCE_UNIX_EPOCH: f64 = 978_307_200.0;
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs_f64() - APPLE_REFERENCE_UNIX_EPOCH)
        .unwrap_or(0.0)
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

    /// Regression: an earlier `LEFT JOIN Z_METADATA AS zmd ON 1=1` produced
    /// one duplicated row per extra Z_METADATA row. Core Data stores usually
    /// have a single row, but legacy/merged stores can have more. The query
    /// must read Z_UUID via a scalar subquery so each note appears once.
    #[test]
    fn search_is_not_duplicated_when_z_metadata_has_multiple_rows() {
        let store = fixture_store();
        store
            .conn
            .execute("INSERT INTO Z_METADATA (Z_UUID) VALUES ('SECOND-UUID')", [])
            .unwrap();

        let hits = store.search("refund", None, 10).unwrap();
        assert_eq!(hits.len(), 1, "search should not duplicate notes");

        let indexed = store.all_indexed_notes().unwrap();
        assert_eq!(indexed.len(), 2, "index should not duplicate notes");
    }

    /// Regression: a parent cycle in the folder table (e.g. corrupt iCloud
    /// merge state where folder A.parent == B and B.parent == A) used to make
    /// `descendant_folder_ids` loop forever and grow an unbounded Vec, which
    /// would then explode `count_notes_in_folders`'s IN-clause. Cycles must
    /// terminate the traversal cleanly.
    #[test]
    fn descendant_folder_ids_terminates_on_cyclic_parent_chain() {
        let mut raw = HashMap::new();
        raw.insert(
            1,
            RawFolder {
                id: 1,
                name: "A".into(),
                parent_id: Some(2),
                account_id: Some(99),
                account: Some("iCloud".into()),
            },
        );
        raw.insert(
            2,
            RawFolder {
                id: 2,
                name: "B".into(),
                parent_id: Some(1),
                account_id: Some(99),
                account: Some("iCloud".into()),
            },
        );

        let descendants = descendant_folder_ids(&raw, 1);
        assert_eq!(descendants, vec![2], "cycle must visit B exactly once");

        let descendants = descendant_folder_ids(&raw, 2);
        assert_eq!(descendants, vec![1], "cycle must visit A exactly once");
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
