use std::fs::{self, File};
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use clap::{Args, Parser, Subcommand};
use serde::Serialize;

use crate::notes::{
    FolderEntry, FolderMovePlan, IndexedNote, NgError, NoteHit, NoteMovePlan, StoreStats,
    default_db_path, open_store, open_store_for_writing, search_indexed_notes,
};

#[derive(Debug, Parser)]
#[command(
    name = "ng",
    version,
    about = "Fast local Apple Notes search",
    long_about = "Fast local Apple Notes search. Indexes Apple Notes body blobs into a local JSONL cache, searches warmed caches, and provides guarded nested folder moves."
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
    /// List, rename, or move nested Notes folders.
    #[command(alias = "folders")]
    Folder {
        #[command(subcommand)]
        command: FolderCommand,
    },
    /// Move one active note to an existing folder.
    #[command(alias = "notes")]
    Note {
        #[command(subcommand)]
        command: NoteCommand,
    },
    /// Open a note in Notes.app by AppleScript/coredata ID.
    Open(OpenArgs),
}

#[derive(Debug, Subcommand)]
enum FolderCommand {
    /// List folder paths.
    List,
    /// Rename or move a folder to a new nested path.
    #[command(name = "mv", alias = "move", alias = "rename")]
    Move(FolderMoveArgs),
}

#[derive(Debug, Subcommand)]
enum NoteCommand {
    /// Move one active note to an existing same-account folder.
    #[command(name = "mv", alias = "move")]
    Move(NoteMoveArgs),
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

#[derive(Debug, Args)]
struct FolderMoveArgs {
    /// Existing folder path. Account-prefixed paths disambiguate duplicates.
    source: String,
    /// New final folder path. A one-segment target renames in place.
    target: String,

    /// Write the change. Without this flag the command prints a dry-run plan.
    #[arg(long)]
    apply: bool,
}

#[derive(Debug, Args)]
struct NoteMoveArgs {
    /// Stable x-coredata://.../ICNote/p... note ID, or an unambiguous numeric database ID.
    note_id: String,
    /// Existing destination folder path. Account-prefixed paths disambiguate duplicates.
    folder: String,

    /// Write the change. Without this flag the command prints a dry-run plan.
    #[arg(long)]
    apply: bool,
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

#[derive(Debug, Serialize)]
struct FolderMoveView {
    status: &'static str,
    applied: bool,
    changed: bool,
    source: String,
    target: String,
    descendant_folders: usize,
    notes: usize,
}

#[derive(Debug, Serialize)]
struct NoteMoveView {
    status: &'static str,
    applied: bool,
    changed: bool,
    note_id: String,
    note_db_id: i64,
    note_title: String,
    source_folder_path: String,
    target_folder_path: String,
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
        Some(CommandKind::Folder { command }) => folder_command(&db_path, command, cli.json),
        Some(CommandKind::Note { command }) => note_command(&db_path, command, cli.json),
        Some(CommandKind::Open(args)) => open_note(args),
    }
}

fn print_home(db_path: &Path, json: bool) -> Result<(), NgError> {
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
        commands: vec![
            "ng doctor",
            "ng stats",
            "ng search \"query\"",
            "ng index",
            "ng folder list",
            "ng note mv NOTE_ID FOLDER",
        ],
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

fn doctor(db_path: &Path, cache_override: Option<PathBuf>, json: bool) -> Result<(), NgError> {
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

fn stats(db_path: &Path, json: bool) -> Result<(), NgError> {
    let store = open_store(db_path)?;
    let stats = store.stats()?;
    if json {
        println!("{}", serde_json::to_string_pretty(&stats)?);
    } else {
        print_stats(&stats);
    }
    Ok(())
}

fn index(db_path: &Path, cache_override: Option<PathBuf>, json: bool) -> Result<(), NgError> {
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
    db_path: &Path,
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

fn folder_command(db_path: &Path, command: FolderCommand, json: bool) -> Result<(), NgError> {
    match command {
        FolderCommand::List => folder_list(db_path, json),
        FolderCommand::Move(args) => folder_move(db_path, args, json),
    }
}

fn folder_list(db_path: &Path, json: bool) -> Result<(), NgError> {
    let store = open_store(db_path)?;
    let folders = store.folders()?;
    if json {
        println!("{}", serde_json::to_string_pretty(&folders)?);
    } else {
        print_folders(&folders);
    }
    Ok(())
}

fn folder_move(db_path: &Path, args: FolderMoveArgs, json: bool) -> Result<(), NgError> {
    let mut store = if args.apply {
        open_store_for_writing(db_path)?
    } else {
        open_store(db_path)?
    };
    let plan = store.plan_folder_move(&args.source, &args.target)?;
    if args.apply {
        store.apply_folder_move(&plan)?;
    }
    let view = folder_move_view(&plan, args.apply);

    if json {
        println!("{}", serde_json::to_string_pretty(&view)?);
    } else {
        print_folder_move(&view);
    }
    Ok(())
}

fn note_command(db_path: &Path, command: NoteCommand, json: bool) -> Result<(), NgError> {
    match command {
        NoteCommand::Move(args) => note_move(db_path, args, json),
    }
}

fn note_move(db_path: &Path, args: NoteMoveArgs, json: bool) -> Result<(), NgError> {
    let mut store = if args.apply {
        open_store_for_writing(db_path)?
    } else {
        open_store(db_path)?
    };
    let plan = store.plan_note_move(&args.note_id, &args.folder)?;
    if args.apply {
        store.apply_note_move(&plan)?;
    }
    let view = note_move_view(&plan, args.apply);

    if json {
        println!("{}", serde_json::to_string_pretty(&view)?);
    } else {
        print_note_move(&view);
    }
    Ok(())
}

fn print_stats(stats: &StoreStats) {
    println!("notes: {}", stats.notes);
    println!("folders: {}", stats.folders);
    println!("accounts: {}", stats.accounts);
}

fn print_folders(folders: &[FolderEntry]) {
    println!("folders: {}", folders.len());
    for folder in folders {
        println!("{}", folder.account_path);
    }
}

fn folder_move_view(plan: &FolderMovePlan, applied: bool) -> FolderMoveView {
    FolderMoveView {
        status: if applied { "ok" } else { "dry-run" },
        applied,
        changed: plan.will_change,
        source: plan.source_path.clone(),
        target: plan.target_path.clone(),
        descendant_folders: plan.descendant_folders,
        notes: plan.notes,
    }
}

fn print_folder_move(view: &FolderMoveView) {
    println!("folder-move: {}", view.status);
    println!("source: {}", view.source);
    println!("target: {}", view.target);
    println!("changed: {}", view.changed);
    println!("descendant-folders: {}", view.descendant_folders);
    println!("notes: {}", view.notes);
    if !view.applied {
        println!("next: rerun with --apply to write this folder move");
    }
}

fn note_move_view(plan: &NoteMovePlan, applied: bool) -> NoteMoveView {
    NoteMoveView {
        status: if applied { "ok" } else { "dry-run" },
        applied,
        changed: plan.will_change,
        note_id: plan.note_id.clone(),
        note_db_id: plan.note_db_id,
        note_title: plan.note_title.clone(),
        source_folder_path: plan.source_folder_path.clone(),
        target_folder_path: plan.target_folder_path.clone(),
    }
}

fn print_note_move(view: &NoteMoveView) {
    println!("note-move: {}", view.status);
    println!("note-id: {}", view.note_id);
    println!("title: {}", truncate(&view.note_title, 96));
    println!("source-folder: {}", view.source_folder_path);
    println!("target-folder: {}", view.target_folder_path);
    println!("changed: {}", view.changed);
    println!("applied: {}", view.applied);
    if !view.applied {
        println!(
            "next: rerun with --apply to write this note move, then rebuild caches with ng index"
        );
    }
}

fn print_hits(hits: &[NoteHit]) {
    println!("hits: {}", hits.len());
    for hit in hits {
        let folder = hit
            .folder_path
            .as_deref()
            .or(hit.folder.as_deref())
            .unwrap_or("-");
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
