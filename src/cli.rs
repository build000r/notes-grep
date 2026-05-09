use std::fs::{self, File};
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::PathBuf;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use clap::{Args, Parser, Subcommand};
use serde::Serialize;

use crate::notes::{
    IndexedNote, NgError, NoteHit, StoreStats, default_db_path, open_store, search_indexed_notes,
};

#[derive(Debug, Parser)]
#[command(
    name = "ng",
    version,
    about = "Fast local Apple Notes search",
    long_about = "Fast local Apple Notes search. v0.1 indexes Apple Notes body blobs into a local JSONL cache and searches that warmed cache when available."
)]
struct Cli {
    #[command(subcommand)]
    command: Option<CommandKind>,

    #[arg(long, global = true, env = "NG_NOTES_DB", value_name = "PATH")]
    db: Option<PathBuf>,

    #[arg(long, global = true, value_name = "DIR")]
    cache_dir: Option<PathBuf>,

    #[arg(long, global = true)]
    json: bool,
}

#[derive(Debug, Subcommand)]
enum CommandKind {
    /// Print Notes database status and next useful commands.
    Doctor,
    /// Show counts from the Notes database.
    Stats,
    /// Write the current searchable full-body note cache.
    Index,
    /// Search the warmed full-body cache, falling back to title/snippet SQLite search.
    Search(SearchArgs),
    /// Open a note in Notes.app by AppleScript/coredata ID.
    Open(OpenArgs),
}

#[derive(Debug, Args)]
struct SearchArgs {
    query: String,

    #[arg(long, short = 'f', value_name = "FOLDER")]
    folder: Option<String>,

    #[arg(long, short = 'n', default_value_t = 20)]
    limit: usize,
}

#[derive(Debug, Args)]
struct OpenArgs {
    note_id: String,
}

#[derive(Debug, Serialize)]
struct HomeView {
    tool: &'static str,
    status: &'static str,
    db: String,
    commands: Vec<&'static str>,
}

#[derive(Debug, Serialize)]
struct DoctorView {
    status: &'static str,
    db: String,
    notes: usize,
    folders: usize,
    cache_dir: String,
}

#[derive(Debug, Serialize)]
struct IndexView {
    status: &'static str,
    db: String,
    cache_file: String,
    notes: usize,
    body_notes: usize,
}

pub fn run() -> Result<(), NgError> {
    let cli = Cli::parse();
    let db_path = cli.db.unwrap_or_else(default_db_path);
    let cache_override = cli.cache_dir;

    match cli.command {
        None => print_home(&db_path, cli.json),
        Some(CommandKind::Doctor) => doctor(&db_path, cache_override, cli.json),
        Some(CommandKind::Stats) => stats(&db_path, cli.json),
        Some(CommandKind::Index) => index(&db_path, cache_override, cli.json),
        Some(CommandKind::Search(args)) => search(&db_path, cache_override, args, cli.json),
        Some(CommandKind::Open(args)) => open_note(args),
    }
}

fn print_home(db_path: &PathBuf, json: bool) -> Result<(), NgError> {
    let status = match open_store(db_path).and_then(|store| store.stats()) {
        Ok(_) => "ready",
        Err(NgError::DatabaseMissing(_)) => "missing-notes-db",
        Err(NgError::DatabaseOpen { .. }) => "needs-full-disk-access",
        Err(NgError::Schema(_)) => "unrecognized-notes-schema",
        Err(_) => "needs-attention",
    };
    let view = HomeView {
        tool: "ng",
        status,
        db: db_path.display().to_string(),
        commands: vec!["ng doctor", "ng stats", "ng search \"query\"", "ng index"],
    };

    if json {
        println!("{}", serde_json::to_string_pretty(&view)?);
    } else {
        println!("ng: {}", view.status);
        println!("db: {}", view.db);
        println!("next: {}", view.commands.join(" | "));
    }
    Ok(())
}

fn doctor(db_path: &PathBuf, cache_override: Option<PathBuf>, json: bool) -> Result<(), NgError> {
    let store = open_store(db_path)?;
    let stats = store.stats()?;
    let view = DoctorView {
        status: "ok",
        db: db_path.display().to_string(),
        notes: stats.notes,
        folders: stats.folders,
        cache_dir: cache_dir(cache_override)?.display().to_string(),
    };
    if json {
        println!("{}", serde_json::to_string_pretty(&view)?);
    } else {
        println!("status: ok");
        println!("db: {}", view.db);
        println!("notes: {}", view.notes);
        println!("folders: {}", view.folders);
        println!("next: ng search \"query\"");
    }
    Ok(())
}

fn stats(db_path: &PathBuf, json: bool) -> Result<(), NgError> {
    let store = open_store(db_path)?;
    let stats = store.stats()?;
    if json {
        println!("{}", serde_json::to_string_pretty(&stats)?);
    } else {
        print_stats(&stats);
    }
    Ok(())
}

fn index(db_path: &PathBuf, cache_override: Option<PathBuf>, json: bool) -> Result<(), NgError> {
    let store = open_store(db_path)?;
    let notes = store.all_indexed_notes()?;
    let body_notes = notes
        .iter()
        .filter(|note| note.body.as_deref().is_some_and(|body| !body.is_empty()))
        .count();
    let dir = cache_dir(cache_override)?;
    fs::create_dir_all(&dir)?;
    let cache_file = notes_cache_file(Some(dir.clone()))?;
    let mut writer = BufWriter::new(File::create(&cache_file)?);
    for note in &notes {
        serde_json::to_writer(&mut writer, note)?;
        writer.write_all(b"\n")?;
    }
    writer.flush()?;

    let manifest = dir.join("manifest.json");
    let manifest_view = IndexView {
        status: "ok",
        db: db_path.display().to_string(),
        cache_file: cache_file.display().to_string(),
        notes: notes.len(),
        body_notes,
    };
    fs::write(
        manifest,
        serde_json::to_vec_pretty(&serde_json::json!({
            "tool": "ng",
            "indexed_at_unix": unix_now(),
            "db": manifest_view.db,
            "cache_file": manifest_view.cache_file,
            "notes": manifest_view.notes,
            "body_notes": manifest_view.body_notes
        }))?,
    )?;

    if json {
        println!("{}", serde_json::to_string_pretty(&manifest_view)?);
    } else {
        println!("index: ok");
        println!("notes: {}", manifest_view.notes);
        println!("body-notes: {}", manifest_view.body_notes);
        println!("cache: {}", manifest_view.cache_file);
        println!("scope: title+snippet+body cache");
    }
    Ok(())
}

fn search(
    db_path: &PathBuf,
    cache_override: Option<PathBuf>,
    args: SearchArgs,
    json: bool,
) -> Result<(), NgError> {
    let cache_file = notes_cache_file(cache_override)?;
    let hits = if cache_file.exists() {
        let notes = read_indexed_notes(&cache_file)?;
        search_indexed_notes(&notes, &args.query, args.folder.as_deref(), args.limit)
    } else {
        let store = open_store(db_path)?;
        store.search(&args.query, args.folder.as_deref(), args.limit)?
    };

    if json {
        println!("{}", serde_json::to_string_pretty(&hits)?);
    } else {
        print_hits(&hits);
        if hits.is_empty() {
            println!(
                "next: try ng index, ng search \"{}\" --json, or ng doctor",
                args.query
            );
        }
    }
    Ok(())
}

fn open_note(args: OpenArgs) -> Result<(), NgError> {
    let status = Command::new("open").arg(args.note_id).status()?;
    if status.success() {
        println!("open: ok");
        Ok(())
    } else {
        Err(NgError::OpenFailed)
    }
}

fn print_stats(stats: &StoreStats) {
    println!("notes: {}", stats.notes);
    println!("folders: {}", stats.folders);
    println!("accounts: {}", stats.accounts);
}

fn print_hits(hits: &[NoteHit]) {
    println!("hits: {}", hits.len());
    for hit in hits {
        let folder = hit.folder.as_deref().unwrap_or("-");
        let title = truncate(&hit.title, 72);
        let snippet = truncate(hit.snippet.as_deref().unwrap_or(""), 96);
        println!("{}  {}  {}", hit.id, folder, title);
        if !snippet.is_empty() {
            println!("  {}", snippet.replace('\n', " "));
        }
    }
}

fn truncate(value: &str, max_chars: usize) -> String {
    let mut chars = value.chars();
    let mut out = String::new();
    for _ in 0..max_chars {
        match chars.next() {
            Some(ch) => out.push(ch),
            None => return out,
        }
    }
    if chars.next().is_some() {
        out.push_str("...");
    }
    out
}

fn cache_dir(override_dir: Option<PathBuf>) -> Result<PathBuf, NgError> {
    if let Some(dir) = override_dir {
        return Ok(dir);
    }
    let base = dirs::data_local_dir().ok_or(NgError::NoDataDir)?;
    Ok(base.join("notes-grep"))
}

fn notes_cache_file(override_dir: Option<PathBuf>) -> Result<PathBuf, NgError> {
    Ok(cache_dir(override_dir)?.join("notes.jsonl"))
}

fn read_indexed_notes(cache_file: &PathBuf) -> Result<Vec<IndexedNote>, NgError> {
    let reader = BufReader::new(File::open(cache_file)?);
    let mut notes = Vec::new();
    for line in reader.lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        notes.push(serde_json::from_str(&line)?);
    }
    Ok(notes)
}

fn unix_now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0)
}
