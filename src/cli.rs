use std::fs::{self, File};
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use clap::{Args, Parser, Subcommand};
use serde::{Deserialize, Serialize};

use regex::Regex;

use crate::notes::{
    FolderEntry, FolderMovePlan, IndexedNote, NgError, NoteHit, NoteMovePlan, StoreStats,
    default_db_path, is_coredata_note_id, open_store, open_store_for_writing, search_indexed_notes,
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

    #[arg(long, short = 'e')]
    regex: bool,

    #[arg(long, short = 'c')]
    count: bool,

    #[arg(long, short = 'l')]
    id_only: bool,

    #[arg(long, short = 'q')]
    quiet: bool,

    #[arg(long, value_name = "DATE")]
    after: Option<String>,

    #[arg(long, value_name = "DATE")]
    before: Option<String>,

    #[arg(long, short = 's', value_name = "KEY")]
    sort: Option<SortKey>,

    #[arg(long)]
    no_snippet: bool,
}

#[derive(Debug, Clone, clap::ValueEnum)]
enum SortKey {
    Date,
    Title,
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

#[derive(Debug, Serialize)]
struct OpenView {
    status: &'static str,
    note_id: String,
}

#[derive(Debug, Deserialize)]
struct CacheManifest {
    db: Option<String>,
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
        Some(CommandKind::Open(args)) => open_note(args, cli.json),
    }
}

fn print_home(db_path: &Path, json: bool) -> Result<(), NgError> {
    let view = HomeView {
        tool: "ng",
        status: home_status(db_path),
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

fn home_status(db_path: &Path) -> &'static str {
    match open_store(db_path).and_then(|store| store.stats()) {
        Ok(_) => "ready",
        Err(NgError::DatabaseMissing(_)) => "missing-notes-db",
        Err(NgError::DatabaseOpen { .. }) => "needs-full-disk-access",
        Err(NgError::Schema(_)) => "unrecognized-notes-schema",
        Err(_) => "needs-attention",
    }
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
    // Write to sibling .tmp files first and rename on success so a partial
    // write (disk full, ctrl-C mid-loop, panic in body decode) cannot leave
    // `ng search` reading a truncated JSONL or stale-but-current manifest.
    let cache_tmp = with_tmp_suffix(&cache_file);
    {
        let mut writer = BufWriter::new(File::create(&cache_tmp)?);
        for note in &notes {
            serde_json::to_writer(&mut writer, note)?;
            writer.write_all(b"\n")?;
        }
        writer.flush()?;
    }

    let manifest = dir.join("manifest.json");
    let manifest_view = IndexView {
        status: "ok",
        db: db_path.display().to_string(),
        cache_file: cache_file.display().to_string(),
        notes: notes.len(),
        body_notes,
    };
    let manifest_tmp = with_tmp_suffix(&manifest);
    fs::write(
        &manifest_tmp,
        serde_json::to_vec_pretty(&serde_json::json!({
            "tool": "ng",
            "indexed_at_unix": unix_now(),
            "db": manifest_view.db,
            "cache_file": manifest_view.cache_file,
            "notes": manifest_view.notes,
            "body_notes": manifest_view.body_notes
        }))?,
    )?;
    fs::rename(&cache_tmp, &cache_file)?;
    fs::rename(&manifest_tmp, &manifest)?;

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
    let matcher = if args.regex {
        Some(compile_regex(&args.query)?)
    } else {
        None
    };
    let cache_file = notes_cache_file(cache_override)?;
    let hits = if cache_file.exists() && cache_matches_db(&cache_file, db_path)? {
        let notes = read_indexed_notes(&cache_file)?;
        search_indexed_notes(
            &notes,
            &args.query,
            matcher.as_ref(),
            args.folder.as_deref(),
            args.limit,
        )
    } else {
        let store = open_store(db_path)?;
        store.search(
            &args.query,
            matcher.as_ref(),
            args.folder.as_deref(),
            args.limit,
        )?
    };

    let mut hits = filter_by_date(hits, args.after.as_deref(), args.before.as_deref())?;

    if let Some(key) = &args.sort {
        match key {
            SortKey::Date => hits.sort_by(|a, b| b.modified.cmp(&a.modified)),
            SortKey::Title => hits.sort_by(|a, b| a.title.cmp(&b.title)),
        }
    }

    if args.quiet {
        return if hits.is_empty() {
            Err(NgError::NoMatch)
        } else {
            Ok(())
        };
    }

    if args.count {
        println!("{}", hits.len());
    } else if args.id_only {
        for hit in &hits {
            println!("{}", hit.id);
        }
    } else if json {
        println!("{}", serde_json::to_string_pretty(&hits)?);
    } else {
        print_hits(&hits, args.no_snippet);
        if hits.is_empty() {
            println!(
                "next: try ng index, ng search \"{}\" --json, or ng doctor",
                args.query
            );
        }
    }
    Ok(())
}

fn open_note(args: OpenArgs, json: bool) -> Result<(), NgError> {
    let note_id = args.note_id.trim();
    if !is_coredata_note_id(note_id) {
        return Err(NgError::Command(
            "note ID must be an x-coredata://.../ICNote/p... ID".to_owned(),
        ));
    }

    let status = Command::new("open").arg(note_id).status()?;
    if status.success() {
        let view = OpenView {
            status: "ok",
            note_id: note_id.to_owned(),
        };
        if json {
            println!("{}", serde_json::to_string_pretty(&view)?);
        } else {
            println!("open: ok");
        }
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
        println!("{}", sanitize_terminal(&folder.account_path));
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
    println!("source: {}", sanitize_terminal(&view.source));
    println!("target: {}", sanitize_terminal(&view.target));
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
    println!("note-id: {}", sanitize_terminal(&view.note_id));
    println!(
        "title: {}",
        sanitize_terminal(&truncate(&view.note_title, 96))
    );
    println!(
        "source-folder: {}",
        sanitize_terminal(&view.source_folder_path)
    );
    println!(
        "target-folder: {}",
        sanitize_terminal(&view.target_folder_path)
    );
    println!("changed: {}", view.changed);
    println!("applied: {}", view.applied);
    if !view.applied {
        println!(
            "next: rerun with --apply to write this note move, then rebuild caches with ng index"
        );
    }
}

fn print_hits(hits: &[NoteHit], no_snippet: bool) {
    println!("hits: {}", hits.len());
    for hit in hits {
        let folder = hit
            .account_path
            .as_deref()
            .or(hit.folder_path.as_deref())
            .or(hit.folder.as_deref())
            .unwrap_or("-");
        let id = sanitize_terminal(&hit.id);
        let title = sanitize_terminal(&truncate(&hit.title, 72));
        println!("{}  {}  {}", id, sanitize_terminal(folder), title);
        if !no_snippet {
            let snippet = sanitize_terminal(&truncate(hit.snippet.as_deref().unwrap_or(""), 96));
            if !snippet.is_empty() {
                println!("  {snippet}");
            }
        }
    }
}

/// Render untrusted note/folder text inert for terminal display. Apple Notes
/// titles, snippets, decoded body lines, and folder/account names are
/// attacker-influenced (a shared or imported note can carry arbitrary bytes).
/// Without this, an embedded ANSI/terminal escape (ESC = U+001B, bare CR, BS,
/// etc.) would reach the user's terminal raw and could recolor or overwrite
/// output, hide note IDs before an `ng note mv`/`ng open`, or trigger
/// terminal-specific control-sequence behavior. C0/C1 control characters are
/// replaced with a visible placeholder; ordinary whitespace runs (including
/// newlines) collapse to a single space so a hit stays on its own line. The
/// `--json` path is unaffected because serde_json already escapes U+0000-U+001F.
fn sanitize_terminal(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    let mut pending_space = false;
    for ch in value.chars() {
        if ch.is_whitespace() {
            pending_space = true;
            continue;
        }
        if pending_space {
            out.push(' ');
            pending_space = false;
        }
        // `char::is_control` covers C0 (U+0000-U+001F) and C1/DEL
        // (U+007F-U+009F) control characters. Any that survived earlier
        // decoding are replaced with a visible marker so the user can tell
        // content was scrubbed rather than silently dropped.
        if ch.is_control() {
            out.push('\u{fffd}');
        } else {
            out.push(ch);
        }
    }
    out
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

fn cache_matches_db(cache_file: &Path, db_path: &Path) -> Result<bool, NgError> {
    let Some(cache_dir) = cache_file.parent() else {
        return Ok(false);
    };
    let manifest = cache_dir.join("manifest.json");
    if !manifest.exists() {
        return Ok(false);
    }

    let manifest: CacheManifest = serde_json::from_slice(&fs::read(&manifest)?)?;
    let Some(manifest_db) = manifest.db else {
        return Ok(false);
    };
    Ok(paths_match(&manifest_db, db_path))
}

fn paths_match(left: &str, right: &Path) -> bool {
    if Path::new(left) == right {
        return true;
    }

    let Ok(left) = fs::canonicalize(left) else {
        return false;
    };
    let Ok(right) = fs::canonicalize(right) else {
        return false;
    };
    left == right
}

fn read_indexed_notes(cache_file: &Path) -> Result<Vec<IndexedNote>, NgError> {
    let reader = BufReader::new(File::open(cache_file)?);
    let mut notes = Vec::new();
    for (line_index, line) in reader.lines().enumerate() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        let line_number = line_index + 1;
        notes.push(serde_json::from_str(&line).map_err(|source| {
            NgError::Command(format!(
                "failed to read cache {} line {line_number}: {source}. Rebuild with ng index.",
                cache_file.display()
            ))
        })?);
    }
    Ok(notes)
}

fn with_tmp_suffix(path: &Path) -> PathBuf {
    let mut name = path
        .file_name()
        .map(|name| name.to_os_string())
        .unwrap_or_default();
    name.push(".tmp");
    path.with_file_name(name)
}

fn filter_by_date(
    hits: Vec<NoteHit>,
    after: Option<&str>,
    before: Option<&str>,
) -> Result<Vec<NoteHit>, NgError> {
    if after.is_none() && before.is_none() {
        return Ok(hits);
    }
    let after = after.map(normalize_date_arg).transpose()?;
    let before = before.map(normalize_date_arg).transpose()?;
    Ok(hits
        .into_iter()
        .filter(|hit| {
            let Some(modified) = hit.modified.as_deref() else {
                return false;
            };
            if let Some(ref after) = after {
                if modified < after.as_str() {
                    return false;
                }
            }
            if let Some(ref before) = before {
                if modified >= before.as_str() {
                    return false;
                }
            }
            true
        })
        .collect())
}

fn normalize_date_arg(date: &str) -> Result<String, NgError> {
    let trimmed = date.trim();
    if trimmed.len() == 10
        && trimmed.as_bytes().get(4) == Some(&b'-')
        && trimmed.as_bytes().get(7) == Some(&b'-')
        && trimmed.bytes().filter(|b| b.is_ascii_digit()).count() == 8
    {
        return Ok(format!("{trimmed} 00:00:00"));
    }
    if trimmed.len() == 19
        && trimmed.as_bytes().get(10) == Some(&b' ')
        && trimmed.as_bytes().get(13) == Some(&b':')
    {
        return Ok(trimmed.to_owned());
    }
    Err(NgError::Command(format!(
        "invalid date: '{trimmed}'. Use YYYY-MM-DD or YYYY-MM-DD HH:MM:SS."
    )))
}

fn compile_regex(pattern: &str) -> Result<Regex, NgError> {
    regex::RegexBuilder::new(pattern)
        .case_insensitive(true)
        .build()
        .map_err(|err| NgError::Command(format!("invalid regex: {err}")))
}

fn unix_now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0)
}
