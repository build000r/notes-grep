use assert_cmd::Command;
use flate2::Compression;
use flate2::write::GzEncoder;
use predicates::prelude::*;
use rusqlite::{Connection, params};
use std::fs;
use std::io::Write;
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
        INSERT INTO ZICCLOUDSYNCINGOBJECT (Z_PK, Z_ENT, ZTITLE2, ZMARKEDFORDELETION)
            VALUES (10, 15, 'Finance', 0);
        INSERT INTO ZICCLOUDSYNCINGOBJECT (Z_PK, Z_ENT, ZTITLE2, ZMARKEDFORDELETION)
            VALUES (11, 15, 'Personal', 0);
        INSERT INTO ZICCLOUDSYNCINGOBJECT (Z_PK, Z_ENT, ZNAME, ZUSERRECORDNAME, ZMARKEDFORDELETION)
            VALUES (12, 13, 'iCloud', '_fixture', 0);
        INSERT INTO ZICCLOUDSYNCINGOBJECT (Z_PK, Z_ENT, ZTITLE1, ZSNIPPET, ZFOLDER, ZNOTEDATA, ZMODIFICATIONDATE1, ZMARKEDFORDELETION)
            VALUES (1, 12, 'Stripe refund', 'Refund receipt follow-up', 10, 101, 800000000, 0);
        INSERT INTO ZICCLOUDSYNCINGOBJECT (Z_PK, Z_ENT, ZTITLE1, ZSNIPPET, ZFOLDER, ZNOTEDATA, ZMODIFICATIONDATE1, ZMARKEDFORDELETION)
            VALUES (2, 12, 'Garden list', 'Tomatoes and irrigation', 11, 102, 700000000, 0);
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
        .stdout(predicate::str::contains("folders: 2"));
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
