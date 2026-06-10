use assert_cmd::Command;
use flate2::Compression;
use flate2::write::GzEncoder;
use predicates::prelude::*;
use rusqlite::{Connection, params};
use std::fs;
use std::io::Write;
use std::path::Path;
use tempfile::TempDir;

fn fixture_db() -> (TempDir, std::path::PathBuf) {
    let temp = TempDir::new().expect("temp dir");
    let path = temp.path().join("NoteStore.sqlite");
    let conn = Connection::open(&path).expect("fixture db");
    conn.execute_batch(
        r#"
        CREATE TABLE Z_METADATA (Z_UUID TEXT);
        INSERT INTO Z_METADATA VALUES ('FIXTURE-UUID');
        CREATE TABLE ZICCLOUDSYNCINGOBJECT (
            Z_PK INTEGER PRIMARY KEY,
            Z_ENT INTEGER,
            Z_OPT INTEGER,
            ZTITLE1 TEXT,
            ZTITLE2 TEXT,
            ZSNIPPET TEXT,
            ZFOLDER INTEGER,
            ZPARENT INTEGER,
            ZACCOUNT8 INTEGER,
            ZNOTEDATA INTEGER,
            ZMODIFICATIONDATE1 REAL,
            ZFOLDERMODIFICATIONDATE REAL,
            ZPARENTMODIFICATIONDATE REAL,
            ZMARKEDFORDELETION INTEGER,
            ZNAME TEXT,
            ZUSERRECORDNAME TEXT,
            ZACCOUNTIDENTIFIER TEXT
        );
        CREATE TABLE ZICNOTEDATA (
            Z_PK INTEGER PRIMARY KEY,
            ZDATA BLOB
        );
        INSERT INTO ZICCLOUDSYNCINGOBJECT (Z_PK, Z_ENT, Z_OPT, ZNAME, ZUSERRECORDNAME, ZMARKEDFORDELETION)
            VALUES (12, 13, 1, 'iCloud', '_fixture', 0);
        INSERT INTO ZICCLOUDSYNCINGOBJECT (Z_PK, Z_ENT, Z_OPT, ZTITLE2, ZPARENT, ZACCOUNT8, ZMARKEDFORDELETION)
            VALUES (10, 14, 1, 'Finance', NULL, 12, 0);
        INSERT INTO ZICCLOUDSYNCINGOBJECT (Z_PK, Z_ENT, Z_OPT, ZTITLE2, ZPARENT, ZACCOUNT8, ZMARKEDFORDELETION)
            VALUES (11, 14, 1, 'Personal', NULL, 12, 0);
        INSERT INTO ZICCLOUDSYNCINGOBJECT (Z_PK, Z_ENT, Z_OPT, ZTITLE2, ZPARENT, ZACCOUNT8, ZMARKEDFORDELETION)
            VALUES (20, 14, 1, 'Receipts', 10, 12, 0);
        INSERT INTO ZICCLOUDSYNCINGOBJECT (Z_PK, Z_ENT, Z_OPT, ZTITLE2, ZPARENT, ZACCOUNT8, ZMARKEDFORDELETION)
            VALUES (21, 14, 1, 'Trips', 20, 12, 0);
        INSERT INTO ZICCLOUDSYNCINGOBJECT (Z_PK, Z_ENT, Z_OPT, ZTITLE1, ZSNIPPET, ZFOLDER, ZNOTEDATA, ZMODIFICATIONDATE1, ZMARKEDFORDELETION)
            VALUES (1, 12, 1, 'Stripe refund', 'Refund receipt follow-up', 10, 101, 800000000, 0);
        INSERT INTO ZICCLOUDSYNCINGOBJECT (Z_PK, Z_ENT, Z_OPT, ZTITLE1, ZSNIPPET, ZFOLDER, ZNOTEDATA, ZMODIFICATIONDATE1, ZMARKEDFORDELETION)
            VALUES (2, 12, 1, 'Garden list', 'Tomatoes and irrigation', 11, 102, 700000000, 0);
        "#,
    )
    .expect("fixture schema");
    conn.execute(
        "INSERT INTO ZICNOTEDATA (Z_PK, ZDATA) VALUES (?, ?)",
        params![101, body_blob("cache-only alpha phrase lives in the body")],
    )
    .expect("finance body");
    conn.execute(
        "INSERT INTO ZICNOTEDATA (Z_PK, ZDATA) VALUES (?, ?)",
        params![102, body_blob("cache-only beta phrase lives in the body")],
    )
    .expect("personal body");
    drop(conn);
    (temp, path)
}

#[test]
fn search_json_uses_fixture_db() {
    let (_temp, path) = fixture_db();
    let cache_dir = _temp.path().join("empty-cache");
    let mut cmd = Command::cargo_bin("ng").expect("ng binary");
    cmd.args([
        "--db",
        path.to_str().unwrap(),
        "--cache-dir",
        cache_dir.to_str().unwrap(),
        "--json",
        "search",
        "refund",
    ])
    .assert()
    .success()
    .stdout(predicate::str::contains("\"title\": \"Stripe refund\""))
    .stdout(predicate::str::contains("\"folder\": \"Finance\""));
}

#[test]
fn doctor_reports_fixture_counts() {
    let (_temp, path) = fixture_db();
    let mut cmd = Command::cargo_bin("ng").expect("ng binary");
    cmd.args(["--db", path.to_str().unwrap(), "doctor"])
        .assert()
        .success()
        .stdout(predicate::str::contains("status: ok"))
        .stdout(predicate::str::contains("notes: 2"))
        .stdout(predicate::str::contains("folders: 4"));
}

#[test]
fn home_human_output_reports_ready_status_and_next_commands() {
    let (_temp, path) = fixture_db();

    Command::cargo_bin("ng")
        .expect("ng binary")
        .args(["--db", path.to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("ng: ready"))
        .stdout(predicate::str::contains("next: ng doctor"))
        .stdout(predicate::str::contains("ng note mv NOTE_ID FOLDER"));
}

#[test]
fn home_json_output_reports_missing_database_without_error() {
    let missing_db = TempDir::new()
        .expect("temp dir")
        .path()
        .join("missing.sqlite");

    Command::cargo_bin("ng")
        .expect("ng binary")
        .args(["--db", missing_db.to_str().unwrap(), "--json"])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"status\": \"missing-notes-db\""))
        .stdout(predicate::str::contains("\"commands\""))
        .stdout(predicate::str::contains("ng doctor"));
}

#[test]
fn index_writes_full_body_cache_records() {
    let (temp, path) = fixture_db();
    let cache_dir = temp.path().join("cache");
    let cache_file = cache_dir.join("notes.jsonl");

    let mut cmd = Command::cargo_bin("ng").expect("ng binary");
    cmd.args([
        "--db",
        path.to_str().unwrap(),
        "--cache-dir",
        cache_dir.to_str().unwrap(),
        "--json",
        "index",
    ])
    .assert()
    .success()
    .stdout(predicate::str::contains("\"notes\": 2"))
    .stdout(predicate::str::contains("\"body_notes\": 2"));

    let cache = fs::read_to_string(cache_file).expect("cache file");
    assert!(cache.contains("\"body\":\"cache-only alpha phrase lives in the body\""));
}

#[test]
fn search_json_uses_warmed_body_cache_with_folder_and_limit() {
    let (temp, path) = fixture_db();
    let cache_dir = temp.path().join("cache");

    Command::cargo_bin("ng")
        .expect("ng binary")
        .args([
            "--db",
            path.to_str().unwrap(),
            "--cache-dir",
            cache_dir.to_str().unwrap(),
            "index",
        ])
        .assert()
        .success();

    let mut cmd = Command::cargo_bin("ng").expect("ng binary");
    cmd.args([
        "--db",
        path.to_str().unwrap(),
        "--cache-dir",
        cache_dir.to_str().unwrap(),
        "--json",
        "search",
        "cache-only",
        "--folder",
        "Finance",
        "--limit",
        "1",
    ])
    .assert()
    .success()
    .stdout(predicate::str::contains("\"title\": \"Stripe refund\""))
    .stdout(predicate::str::contains("cache-only alpha phrase"))
    .stdout(predicate::str::contains("\"folder\": \"Finance\""))
    .stdout(predicate::str::contains("Garden list").not());
}

#[test]
fn search_does_not_use_cache_for_different_db() {
    let (cache_temp, cached_db) = fixture_db();
    let (_other_temp, other_db) = fixture_db();
    let cache_dir = cache_temp.path().join("cache");

    Command::cargo_bin("ng")
        .expect("ng binary")
        .args([
            "--db",
            cached_db.to_str().unwrap(),
            "--cache-dir",
            cache_dir.to_str().unwrap(),
            "index",
        ])
        .assert()
        .success();

    Command::cargo_bin("ng")
        .expect("ng binary")
        .args([
            "--db",
            other_db.to_str().unwrap(),
            "--cache-dir",
            cache_dir.to_str().unwrap(),
            "--json",
            "search",
            "cache-only alpha",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("Stripe refund").not())
        .stdout(predicate::str::contains("[]"));
}

#[test]
fn malformed_cache_reports_line_and_reindex_hint() {
    let (temp, path) = fixture_db();
    let cache_dir = temp.path().join("bad-cache");
    fs::create_dir_all(&cache_dir).expect("cache dir");
    fs::write(cache_dir.join("notes.jsonl"), "{not-json}\n").expect("bad cache");
    fs::write(
        cache_dir.join("manifest.json"),
        format!(
            r#"{{"tool":"ng","db":"{}","cache_file":"{}","notes":1,"body_notes":0}}"#,
            path.display(),
            cache_dir.join("notes.jsonl").display()
        ),
    )
    .expect("manifest");

    Command::cargo_bin("ng")
        .expect("ng binary")
        .args([
            "--db",
            path.to_str().unwrap(),
            "--cache-dir",
            cache_dir.to_str().unwrap(),
            "search",
            "anything",
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains("line 1"))
        .stderr(predicate::str::contains("Rebuild with ng index"));
}

#[test]
fn search_folder_accepts_account_prefixed_paths() {
    let (temp, path) = fixture_db();
    let cache_dir = temp.path().join("cache");

    Command::cargo_bin("ng")
        .expect("ng binary")
        .args([
            "--db",
            path.to_str().unwrap(),
            "--json",
            "search",
            "refund",
            "--folder",
            "iCloud/Finance",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"title\": \"Stripe refund\""))
        .stdout(predicate::str::contains(
            "\"account_path\": \"iCloud/Finance\"",
        ));

    Command::cargo_bin("ng")
        .expect("ng binary")
        .args([
            "--db",
            path.to_str().unwrap(),
            "--cache-dir",
            cache_dir.to_str().unwrap(),
            "index",
        ])
        .assert()
        .success();

    Command::cargo_bin("ng")
        .expect("ng binary")
        .args([
            "--db",
            path.to_str().unwrap(),
            "--cache-dir",
            cache_dir.to_str().unwrap(),
            "--json",
            "search",
            "cache-only alpha",
            "--folder",
            "iCloud/Finance",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"title\": \"Stripe refund\""))
        .stdout(predicate::str::contains(
            "\"account_path\": \"iCloud/Finance\"",
        ));
}

#[test]
fn search_sqlite_fallback_matches_unicode_case_insensitively() {
    let (temp, path) = fixture_db();
    let conn = Connection::open(&path).expect("fixture db");
    conn.execute(
        r#"
        INSERT INTO ZICCLOUDSYNCINGOBJECT
            (Z_PK, Z_ENT, Z_OPT, ZTITLE1, ZSNIPPET, ZFOLDER, ZMODIFICATIONDATE1, ZMARKEDFORDELETION)
        VALUES (3, 12, 1, 'Café résumé', 'naïve notes', 10, 750000000, 0)
        "#,
        [],
    )
    .expect("unicode note");
    drop(conn);

    // No warmed cache, so this exercises the direct SQLite fallback. SQLite
    // `LIKE` only case-folds ASCII; an uppercase non-ASCII query must still
    // match the lower-cased title/snippet, matching the warmed-cache path.
    let cache_dir = temp.path().join("empty-cache");
    Command::cargo_bin("ng")
        .expect("ng binary")
        .args([
            "--db",
            path.to_str().unwrap(),
            "--cache-dir",
            cache_dir.to_str().unwrap(),
            "--json",
            "search",
            "CAFÉ",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"title\": \"Café résumé\""));
}

#[test]
fn search_treats_like_wildcards_as_literals() {
    let (_temp, path) = fixture_db();
    let cache_dir = _temp.path().join("empty-cache");

    Command::cargo_bin("ng")
        .expect("ng binary")
        .args([
            "--db",
            path.to_str().unwrap(),
            "--cache-dir",
            cache_dir.to_str().unwrap(),
            "--json",
            "search",
            "%",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("Stripe refund").not())
        .stdout(predicate::str::contains("Garden list").not())
        .stdout(predicate::str::contains("[]"));
}

#[test]
fn folder_list_outputs_nested_account_paths() {
    let (_temp, path) = fixture_db();
    let mut cmd = Command::cargo_bin("ng").expect("ng binary");
    cmd.args(["--db", path.to_str().unwrap(), "folder", "list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("iCloud/Finance"))
        .stdout(predicate::str::contains("iCloud/Finance/Receipts/Trips"))
        .stdout(predicate::str::contains("iCloud/Personal"));
}

#[test]
fn folder_moves_preserve_fallback_account_labels() {
    let (_temp, path) = fixture_db();
    clear_account_name(&path);

    Command::cargo_bin("ng")
        .expect("ng binary")
        .args([
            "--db",
            path.to_str().unwrap(),
            "folder",
            "mv",
            "Finance",
            "Money",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("source: account:12/Finance"))
        .stdout(predicate::str::contains("target: account:12/Money"));

    Command::cargo_bin("ng")
        .expect("ng binary")
        .args([
            "--db",
            path.to_str().unwrap(),
            "folder",
            "mv",
            "Finance",
            "account:12/Money",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("target: account:12/Money"));
}

#[test]
fn moves_refuse_unknown_account_boundaries() {
    let (_temp, path) = fixture_db();
    clear_folder_account_ids(&path);

    Command::cargo_bin("ng")
        .expect("ng binary")
        .args([
            "--db",
            path.to_str().unwrap(),
            "folder",
            "mv",
            "Finance",
            "Money",
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains("folder account is unknown"));

    Command::cargo_bin("ng")
        .expect("ng binary")
        .args([
            "--db",
            path.to_str().unwrap(),
            "note",
            "mv",
            "x-coredata://FIXTURE-UUID/ICNote/p1",
            "Personal",
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains("folder account is unknown"));
}

#[test]
fn folder_move_dry_run_does_not_write() {
    let (_temp, path) = fixture_db();
    Command::cargo_bin("ng")
        .expect("ng binary")
        .args([
            "--db",
            path.to_str().unwrap(),
            "folder",
            "mv",
            "Finance",
            "Money",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("folder-move: dry-run"))
        .stdout(predicate::str::contains("source: iCloud/Finance"))
        .stdout(predicate::str::contains("target: iCloud/Money"))
        .stdout(predicate::str::contains("next: rerun with --apply"));

    let mut cmd = Command::cargo_bin("ng").expect("ng binary");
    cmd.args(["--db", path.to_str().unwrap(), "folder", "list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("iCloud/Finance"))
        .stdout(predicate::str::contains("iCloud/Money").not());
}

#[test]
fn folder_move_apply_moves_nested_container_and_search_folder_paths() {
    let (temp, path) = fixture_db();
    let cache_dir = temp.path().join("empty-cache");
    Command::cargo_bin("ng")
        .expect("ng binary")
        .args([
            "--db",
            path.to_str().unwrap(),
            "folder",
            "mv",
            "Finance",
            "Personal/Finance",
            "--apply",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("folder-move: ok"))
        .stdout(predicate::str::contains("source: iCloud/Finance"))
        .stdout(predicate::str::contains("target: iCloud/Personal/Finance"))
        .stdout(predicate::str::contains("descendant-folders: 2"))
        .stdout(predicate::str::contains("notes: 1"));

    Command::cargo_bin("ng")
        .expect("ng binary")
        .args(["--db", path.to_str().unwrap(), "folder", "list"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "iCloud/Personal/Finance/Receipts/Trips",
        ));

    Command::cargo_bin("ng")
        .expect("ng binary")
        .args([
            "--db",
            path.to_str().unwrap(),
            "--cache-dir",
            cache_dir.to_str().unwrap(),
            "--json",
            "search",
            "refund",
            "--folder",
            "Personal/Finance",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"title\": \"Stripe refund\""))
        .stdout(predicate::str::contains(
            "\"folder_path\": \"Personal/Finance\"",
        ));
}

#[test]
fn folder_move_refuses_cycles_and_duplicate_siblings() {
    let (_temp, path) = fixture_db();

    Command::cargo_bin("ng")
        .expect("ng binary")
        .args([
            "--db",
            path.to_str().unwrap(),
            "folder",
            "mv",
            "Finance",
            "Finance/Receipts/Finance",
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "cannot move 'iCloud/Finance' into itself",
        ));

    Command::cargo_bin("ng")
        .expect("ng binary")
        .args([
            "--db",
            path.to_str().unwrap(),
            "folder",
            "mv",
            "Finance",
            "Personal",
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains("target sibling already exists"));
}

#[test]
fn note_move_dry_run_json_does_not_write() {
    let (_temp, path) = fixture_db();

    Command::cargo_bin("ng")
        .expect("ng binary")
        .args([
            "--db",
            path.to_str().unwrap(),
            "--json",
            "note",
            "mv",
            "x-coredata://FIXTURE-UUID/ICNote/p1",
            "Personal",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"status\": \"dry-run\""))
        .stdout(predicate::str::contains("\"applied\": false"))
        .stdout(predicate::str::contains("\"changed\": true"))
        .stdout(predicate::str::contains(
            "\"note_id\": \"x-coredata://FIXTURE-UUID/ICNote/p1\"",
        ))
        .stdout(predicate::str::contains(
            "\"note_title\": \"Stripe refund\"",
        ))
        .stdout(predicate::str::contains(
            "\"source_folder_path\": \"iCloud/Finance\"",
        ))
        .stdout(predicate::str::contains(
            "\"target_folder_path\": \"iCloud/Personal\"",
        ));

    assert_note_folder(&path, 1, 10);
}

#[test]
fn note_move_human_output_neutralizes_note_id_escape_sequences() {
    let (_temp, path) = fixture_db();
    let conn = Connection::open(&path).expect("fixture db");
    conn.execute(
        "UPDATE Z_METADATA SET Z_UUID = ?1",
        params!["FIXTURE-\u{1b}[35mUUID"],
    )
    .expect("escape metadata UUID");
    drop(conn);

    let escaped_note_id = "x-coredata://FIXTURE-\u{1b}[35mUUID/ICNote/p1";
    let output = Command::cargo_bin("ng")
        .expect("ng binary")
        .args([
            "--db",
            path.to_str().unwrap(),
            "note",
            "mv",
            escaped_note_id,
            "Personal",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8(output).expect("utf8 stdout");
    assert!(stdout.contains("note-move: dry-run"));
    assert!(
        !stdout.contains('\u{1b}'),
        "raw ESC must not reach the terminal: {stdout:?}"
    );
    assert_note_folder(&path, 1, 10);
}

#[test]
fn note_move_apply_moves_one_note_without_mutating_note_content() {
    let (_temp, path) = fixture_db();

    Command::cargo_bin("ng")
        .expect("ng binary")
        .args([
            "--db",
            path.to_str().unwrap(),
            "--json",
            "note",
            "mv",
            "x-coredata://FIXTURE-UUID/ICNote/p1",
            "Personal",
            "--apply",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"status\": \"ok\""))
        .stdout(predicate::str::contains("\"applied\": true"))
        .stdout(predicate::str::contains("\"changed\": true"));

    let conn = Connection::open(&path).expect("fixture db");
    let row = conn
        .query_row(
            r#"
            SELECT ZFOLDER, ZTITLE1, ZSNIPPET, ZNOTEDATA, Z_OPT, ZFOLDERMODIFICATIONDATE
            FROM ZICCLOUDSYNCINGOBJECT
            WHERE Z_PK = 1
            "#,
            [],
            |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, i64>(3)?,
                    row.get::<_, i64>(4)?,
                    row.get::<_, Option<f64>>(5)?,
                ))
            },
        )
        .expect("moved note");

    assert_eq!(row.0, 11);
    assert_eq!(row.1, "Stripe refund");
    assert_eq!(row.2, "Refund receipt follow-up");
    assert_eq!(row.3, 101);
    assert_eq!(row.4, 2);
    assert!(row.5.is_some());
    assert_note_folder(&path, 2, 11);
}

#[test]
fn note_move_apply_noops_when_note_is_already_in_target_folder() {
    let (_temp, path) = fixture_db();

    Command::cargo_bin("ng")
        .expect("ng binary")
        .args([
            "--db",
            path.to_str().unwrap(),
            "--json",
            "note",
            "mv",
            "x-coredata://FIXTURE-UUID/ICNote/p1",
            "Finance",
            "--apply",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"status\": \"ok\""))
        .stdout(predicate::str::contains("\"applied\": true"))
        .stdout(predicate::str::contains("\"changed\": false"));

    let conn = Connection::open(&path).expect("fixture db");
    let row = conn
        .query_row(
            r#"
            SELECT ZFOLDER, Z_OPT, ZFOLDERMODIFICATIONDATE
            FROM ZICCLOUDSYNCINGOBJECT
            WHERE Z_PK = 1
            "#,
            [],
            |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, i64>(1)?,
                    row.get::<_, Option<f64>>(2)?,
                ))
            },
        )
        .expect("same-folder note");

    assert_eq!(row.0, 10);
    assert_eq!(row.1, 1);
    assert!(row.2.is_none());
}

#[test]
fn note_move_accepts_numeric_database_id_when_unambiguous() {
    let (_temp, path) = fixture_db();

    Command::cargo_bin("ng")
        .expect("ng binary")
        .args([
            "--db",
            path.to_str().unwrap(),
            "--json",
            "note",
            "mv",
            "2",
            "Finance",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"status\": \"dry-run\""))
        .stdout(predicate::str::contains("\"note_db_id\": 2"))
        .stdout(predicate::str::contains(
            "\"note_id\": \"x-coredata://FIXTURE-UUID/ICNote/p2\"",
        ));

    assert_note_folder(&path, 2, 11);
}

#[test]
fn note_move_refuses_missing_note_missing_folder_and_ambiguous_folder() {
    let (_temp, path) = fixture_db();

    Command::cargo_bin("ng")
        .expect("ng binary")
        .args([
            "--db",
            path.to_str().unwrap(),
            "note",
            "mv",
            "x-coredata://FIXTURE-UUID/ICNote/p999",
            "Personal",
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains("note not found"));

    Command::cargo_bin("ng")
        .expect("ng binary")
        .args([
            "--db",
            path.to_str().unwrap(),
            "note",
            "mv",
            "not-a-coredata/ICNote/p1",
            "Personal",
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains("note ID must be an x-coredata://"));

    Command::cargo_bin("ng")
        .expect("ng binary")
        .args([
            "--db",
            path.to_str().unwrap(),
            "note",
            "mv",
            "x-coredata://FIXTURE-UUID/ICNote/p1",
            "Missing",
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains("folder path not found: 'Missing'"));

    add_second_account(&path);
    Command::cargo_bin("ng")
        .expect("ng binary")
        .args([
            "--db",
            path.to_str().unwrap(),
            "note",
            "mv",
            "x-coredata://FIXTURE-UUID/ICNote/p1",
            "Finance",
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains("folder path is ambiguous"));
}

#[test]
fn note_move_refuses_deleted_target_folder() {
    let (_temp, path) = fixture_db();
    mark_folder_deleted(&path, 11);

    Command::cargo_bin("ng")
        .expect("ng binary")
        .args([
            "--db",
            path.to_str().unwrap(),
            "note",
            "mv",
            "x-coredata://FIXTURE-UUID/ICNote/p1",
            "Personal",
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "folder path not found: 'Personal'",
        ));

    assert_note_folder(&path, 1, 10);
}

#[test]
fn note_move_refuses_cross_account_targets() {
    let (_temp, path) = fixture_db();
    add_second_account(&path);

    Command::cargo_bin("ng")
        .expect("ng binary")
        .args([
            "--db",
            path.to_str().unwrap(),
            "note",
            "mv",
            "x-coredata://FIXTURE-UUID/ICNote/p1",
            "On My Mac/Local",
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "cross-account note moves are not supported",
        ));

    assert_note_folder(&path, 1, 10);
}

#[test]
fn note_move_rebuild_index_updates_folder_filtered_body_search() {
    let (temp, path) = fixture_db();
    let cache_dir = temp.path().join("cache-after-note-move");

    Command::cargo_bin("ng")
        .expect("ng binary")
        .args([
            "--db",
            path.to_str().unwrap(),
            "note",
            "mv",
            "x-coredata://FIXTURE-UUID/ICNote/p1",
            "Personal",
            "--apply",
        ])
        .assert()
        .success();

    Command::cargo_bin("ng")
        .expect("ng binary")
        .args([
            "--db",
            path.to_str().unwrap(),
            "--cache-dir",
            cache_dir.to_str().unwrap(),
            "index",
        ])
        .assert()
        .success();

    Command::cargo_bin("ng")
        .expect("ng binary")
        .args([
            "--db",
            path.to_str().unwrap(),
            "--cache-dir",
            cache_dir.to_str().unwrap(),
            "--json",
            "search",
            "cache-only alpha",
            "--folder",
            "Personal",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"title\": \"Stripe refund\""))
        .stdout(predicate::str::contains("\"folder_path\": \"Personal\""));

    Command::cargo_bin("ng")
        .expect("ng binary")
        .args([
            "--db",
            path.to_str().unwrap(),
            "--cache-dir",
            cache_dir.to_str().unwrap(),
            "--json",
            "search",
            "cache-only alpha",
            "--folder",
            "Finance",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("Stripe refund").not());
}

#[test]
fn open_rejects_non_note_urls() {
    let mut cmd = Command::cargo_bin("ng").expect("ng binary");
    cmd.args(["open", "https://example.com"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("note ID must be an x-coredata://"));
}

#[test]
fn open_json_outputs_structured_success() {
    let temp = TempDir::new().expect("temp dir");
    let fake_open = temp.path().join("open");
    fs::write(&fake_open, "#!/bin/sh\nexit 0\n").expect("fake open");
    let mut permissions = fs::metadata(&fake_open)
        .expect("fake open metadata")
        .permissions();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        permissions.set_mode(0o755);
        fs::set_permissions(&fake_open, permissions).expect("fake open permissions");
    }

    let path = format!(
        "{}:{}",
        temp.path().display(),
        std::env::var("PATH").unwrap()
    );
    let mut cmd = Command::cargo_bin("ng").expect("ng binary");
    cmd.env("PATH", path)
        .args(["--json", "open", "x-coredata://FIXTURE-UUID/ICNote/p1"])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"status\": \"ok\""))
        .stdout(predicate::str::contains(
            "\"note_id\": \"x-coredata://FIXTURE-UUID/ICNote/p1\"",
        ));
}

#[test]
fn exit_codes_match_public_contract() {
    let missing_db = TempDir::new()
        .expect("temp dir")
        .path()
        .join("missing.sqlite");
    Command::cargo_bin("ng")
        .expect("ng binary")
        .args(["--db", missing_db.to_str().unwrap(), "doctor"])
        .assert()
        .code(2);

    let schema_temp = TempDir::new().expect("temp dir");
    let schema_db = schema_temp.path().join("schema.sqlite");
    Connection::open(&schema_db).expect("schema db");
    Command::cargo_bin("ng")
        .expect("ng binary")
        .args(["--db", schema_db.to_str().unwrap(), "doctor"])
        .assert()
        .code(3);

    let (_temp, path) = fixture_db();
    Command::cargo_bin("ng")
        .expect("ng binary")
        .args([
            "--db",
            path.to_str().unwrap(),
            "note",
            "mv",
            "bad-id",
            "Finance",
        ])
        .assert()
        .code(1);

    let fake_open_dir = TempDir::new().expect("temp dir");
    let fake_open = fake_open_dir.path().join("open");
    fs::write(&fake_open, "#!/bin/sh\nexit 1\n").expect("fake open");
    let mut permissions = fs::metadata(&fake_open)
        .expect("fake open metadata")
        .permissions();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        permissions.set_mode(0o755);
        fs::set_permissions(&fake_open, permissions).expect("fake open permissions");
    }
    let path_env = format!(
        "{}:{}",
        fake_open_dir.path().display(),
        std::env::var("PATH").unwrap()
    );
    Command::cargo_bin("ng")
        .expect("ng binary")
        .env("PATH", path_env)
        .args(["open", "x-coredata://FIXTURE-UUID/ICNote/p1"])
        .assert()
        .code(4);
}

fn add_second_account(path: &Path) {
    let conn = Connection::open(path).expect("fixture db");
    conn.execute_batch(
        r#"
        INSERT INTO ZICCLOUDSYNCINGOBJECT (Z_PK, Z_ENT, Z_OPT, ZNAME, ZUSERRECORDNAME, ZMARKEDFORDELETION)
            VALUES (13, 13, 1, 'On My Mac', '_local_fixture', 0);
        INSERT INTO ZICCLOUDSYNCINGOBJECT (Z_PK, Z_ENT, Z_OPT, ZTITLE2, ZPARENT, ZACCOUNT8, ZMARKEDFORDELETION)
            VALUES (30, 14, 1, 'Local', NULL, 13, 0);
        INSERT INTO ZICCLOUDSYNCINGOBJECT (Z_PK, Z_ENT, Z_OPT, ZTITLE2, ZPARENT, ZACCOUNT8, ZMARKEDFORDELETION)
            VALUES (31, 14, 1, 'Finance', NULL, 13, 0);
        "#,
    )
    .expect("second account");
}

/// Security regression: untrusted note content (title and decoded body that
/// becomes a snippet) can contain ANSI/terminal escape sequences. The default
/// human-readable `ng search` output must not emit raw control characters
/// (e.g. ESC, U+001B) to the terminal, or a crafted note could hijack the
/// user's terminal. JSON output is unaffected (serde_json escapes controls).
#[test]
fn search_human_output_neutralizes_terminal_escape_sequences() {
    let (temp, path) = fixture_db();
    let conn = Connection::open(&path).expect("fixture db");
    conn.execute(
        "UPDATE Z_METADATA SET Z_UUID = ?1",
        params!["FIXTURE-\u{1b}[35mUUID"],
    )
    .expect("escape metadata UUID");
    // Title and snippet both carry an ESC-based ANSI sequence plus a bare CR.
    conn.execute(
        r#"
        INSERT INTO ZICCLOUDSYNCINGOBJECT
            (Z_PK, Z_ENT, Z_OPT, ZTITLE1, ZSNIPPET, ZFOLDER, ZMODIFICATIONDATE1, ZMARKEDFORDELETION)
        VALUES (3, 12, 1, ?1, ?2, 10, 850000000, 0)
        "#,
        params![
            "Pwned \u{1b}[31mtitle\u{1b}[0m\rline",
            "Refund \u{1b}[2Jcleared\u{1b}[0m snippet",
        ],
    )
    .expect("escape note");
    drop(conn);

    let cache_dir = temp.path().join("empty-cache");
    let mut cmd = Command::cargo_bin("ng").expect("ng binary");
    let output = cmd
        .args([
            "--db",
            path.to_str().unwrap(),
            "--cache-dir",
            cache_dir.to_str().unwrap(),
            "search",
            "refund",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    // The note must still surface (we did not drop the match), but no raw ESC
    // (0x1b) or bare CR (0x0d) byte may reach the terminal.
    let stdout = String::from_utf8(output).expect("utf8 stdout");
    assert!(
        stdout.contains("title"),
        "matching note should still appear: {stdout:?}"
    );
    assert!(
        !stdout.contains('\u{1b}'),
        "raw ESC must not reach the terminal: {stdout:?}"
    );
    assert!(
        !stdout.contains('\r'),
        "raw CR must not reach the terminal: {stdout:?}"
    );
}

fn clear_account_name(path: &Path) {
    let conn = Connection::open(path).expect("fixture db");
    let updated = conn
        .execute(
            "UPDATE ZICCLOUDSYNCINGOBJECT SET ZNAME = NULL WHERE Z_PK = 12",
            [],
        )
        .expect("clear account name");
    assert_eq!(updated, 1);
}

fn clear_folder_account_ids(path: &Path) {
    let conn = Connection::open(path).expect("fixture db");
    let updated = conn
        .execute(
            "UPDATE ZICCLOUDSYNCINGOBJECT SET ZACCOUNT8 = NULL WHERE ZTITLE2 IS NOT NULL",
            [],
        )
        .expect("clear folder accounts");
    assert_eq!(updated, 4);
}

fn assert_note_folder(path: &Path, note_pk: i64, expected_folder: i64) {
    let conn = Connection::open(path).expect("fixture db");
    let actual = conn
        .query_row(
            "SELECT ZFOLDER FROM ZICCLOUDSYNCINGOBJECT WHERE Z_PK = ?",
            [note_pk],
            |row| row.get::<_, i64>(0),
        )
        .expect("note folder");
    assert_eq!(actual, expected_folder);
}

fn mark_folder_deleted(path: &Path, folder_pk: i64) {
    let conn = Connection::open(path).expect("fixture db");
    let updated = conn
        .execute(
            "UPDATE ZICCLOUDSYNCINGOBJECT SET ZMARKEDFORDELETION = 1 WHERE Z_PK = ?",
            [folder_pk],
        )
        .expect("mark deleted");
    assert_eq!(updated, 1);
}

#[test]
fn search_regex_matches_alternation_across_title_and_body() {
    let (temp, path) = fixture_db();
    let cache_dir = temp.path().join("cache");

    // SQLite fallback (no cache): regex alternation on title/snippet
    let empty_cache = temp.path().join("empty-cache");
    Command::cargo_bin("ng")
        .expect("ng binary")
        .args([
            "--db",
            path.to_str().unwrap(),
            "--cache-dir",
            empty_cache.to_str().unwrap(),
            "--json",
            "search",
            "--regex",
            "str(ip|ipe) ref",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"title\": \"Stripe refund\""));

    // Warmed cache: regex alternation matches body text
    Command::cargo_bin("ng")
        .expect("ng binary")
        .args([
            "--db",
            path.to_str().unwrap(),
            "--cache-dir",
            cache_dir.to_str().unwrap(),
            "index",
        ])
        .assert()
        .success();

    Command::cargo_bin("ng")
        .expect("ng binary")
        .args([
            "--db",
            path.to_str().unwrap(),
            "--cache-dir",
            cache_dir.to_str().unwrap(),
            "--json",
            "search",
            "--regex",
            "cache-only (alpha|beta)",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"title\": \"Stripe refund\""))
        .stdout(predicate::str::contains("\"title\": \"Garden list\""));
}

#[test]
fn search_regex_rejects_invalid_pattern() {
    let (_temp, path) = fixture_db();
    Command::cargo_bin("ng")
        .expect("ng binary")
        .args([
            "--db",
            path.to_str().unwrap(),
            "search",
            "--regex",
            "[invalid",
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains("invalid regex"));
}

fn body_blob(text: &str) -> Vec<u8> {
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
