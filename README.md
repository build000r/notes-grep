# notes-grep

`notes-grep` is a local-first Apple Notes search CLI. The package installs a
short binary named `ng`.

The target shape is: `xf` for Apple Notes, with `rg`-style search ergonomics.
This repo is intentionally not a fork of `xf`; Apple Notes has a different data
model, and the first product boundary is a clean Notes search tool with guarded
folder-management commands.

## Status

The current build is the full-body search spine plus guarded folder and note
moves:

- `ng doctor` validates access to `NoteStore.sqlite`
- `ng stats` prints basic store counts
- `ng index` writes a JSONL cache with title, snippet, folder, modified time,
  stable note ID, nested folder path, account-prefixed folder path, and decoded body text from
  `ZICNOTEDATA.ZDATA`
- `ng search QUERY` searches the warmed JSONL cache when it exists, falling
  back to direct title/snippet SQLite search before the first matching index
- `ng search QUERY --regex` interprets the query as a case-insensitive regex
- `ng search QUERY --count` prints only the number of matching notes
- `ng search QUERY --id-only` prints one stable note ID per line
- `ng search QUERY --quiet` suppresses output; exit 0 on match, exit 1 on none
- `ng search QUERY --after 2025-01-01` filters to notes modified on or after a date
- `ng search QUERY --before 2026-01-01` filters to notes modified before a date
- `ng search QUERY --sort date` sorts results by date (newest first) or title
- `ng search QUERY --no-snippet` suppresses snippet lines in human output
- `ng folder list` prints account-prefixed nested folder paths
- `ng folder mv SOURCE TARGET` previews a folder rename or nested move
- `ng folder mv SOURCE TARGET --apply` writes the guarded same-account move to
  `NoteStore.sqlite`
- `ng note mv NOTE_ID FOLDER` previews moving one active note to an existing
  same-account folder
- `ng note mv NOTE_ID FOLDER --apply` writes only that note's folder pointer
  plus local sync bookkeeping
- `ng open NOTE_ID` opens an `x-coredata://...` note URL with `open`
- `--json` and `--cache-dir` are available for agent use

The body decoder gzip-decodes Apple Notes `ZICNOTEDATA.ZDATA` blobs and walks
their protobuf wire fields for UTF-8 note text. Search supports both literal
substring and regex modes with `rg`-style output flags (`--count`, `--id-only`,
`--quiet`). Tantivy, semantic search, attachments, and OCR/table text are deferred.

## Install Locally

```bash
cargo install --path .
ng doctor
```

For agent shells on this machine, `~/.local/bin` is already on `PATH` while
`~/.cargo/bin` may not be. Use the repo target to install `ng` where those
shells can resolve it:

```bash
make install-local
ng doctor
```

If macOS blocks access to the Notes database, grant Full Disk Access to the
terminal or agent process that runs `ng`.

Default database path:

```text
~/Library/Group Containers/group.com.apple.notes/NoteStore.sqlite
```

Override it with:

```bash
ng --db /path/to/NoteStore.sqlite doctor
NG_NOTES_DB=/path/to/NoteStore.sqlite ng search refund
```

## Commands

```bash
ng
ng doctor
ng doctor --json
ng stats
ng index
ng search "stripe refund"
ng search "invoice" --folder Finance --json
ng search "invoice" --folder "Finance/Receipts" --json
ng folder list
ng folder mv "Finance/Receipts" "Archive/Receipts"
ng folder mv "Finance/Receipts" "Archive/Receipts" --apply
ng search "refund" --json
ng search "str(ip|ipe) ref" --regex --json
ng search "invoice" --folder Finance --count
ng search "receipt" --id-only
ng search "urgent" --quiet && echo "found"
ng note mv "x-coredata://.../ICNote/p123" "Archive/Receipts"
ng note mv "x-coredata://.../ICNote/p123" "Archive/Receipts" --apply
ng --cache-dir /tmp/ng-cache index
ng --cache-dir /tmp/ng-cache search "body-only phrase" --json
ng open "x-coredata://.../ICNote/p123"
```

Folder paths use `/` as the nested-container separator. `ng search --folder`
matches notes in the named folder and all its subfolders. Commands accept
account-prefixed paths such as `iCloud/Finance/Receipts` when an unprefixed path
is ambiguous. `ng folder mv` treats the target as the folder's final path: a
one-segment target renames in place, while a multi-segment target moves under the
target parent and may rename at the same time.

`ng note mv` accepts the stable `x-coredata://.../ICNote/p...` IDs returned by
`ng search --json` and `ng index`. Numeric database IDs are accepted only when
they resolve to exactly one active note in the selected database. The target
folder must already exist, must resolve using the same account-prefixed nested
path rules as `ng folder mv`, and must be in the same account as the note's
current folder.

Applied note moves update the Notes database directly. Rebuild warmed caches
with `ng index` after `ng note mv ... --apply`; otherwise `ng search` may read a
stale `notes.jsonl` folder path until the cache is refreshed.

Warmed cache search is tied to the database path recorded by `ng index`. If a
cache directory contains records for a different `--db`, `ng search` falls back
to the selected database instead of returning unrelated cached notes.

## Design Rules

- Search is read-only.
- Folder writes require an explicit `--apply`; without it, `ng folder mv` is a
  dry run.
- Note moves require an explicit `--apply`; without it, `ng note mv` is a dry
  run.
- Folder moves are same-account only and reject cycles or duplicate sibling
  names.
- Note moves are same-account only and reject missing, ambiguous, deleted, or
  cross-account target folders.
- Output is compact for humans and structured with `--json` for agents.
- Errors should say whether the failure is missing database, missing Full Disk
  Access, unrecognized schema, or a command problem.
- No network calls during search.
- No note edit/delete.
- Do not copy `xf` or `frankensearch` code; use standard crates and clean-room
  implementation.

## Development

```bash
cargo fmt
cargo test
cargo llvm-cov --lcov --output-path lcov.info
cargo run -- doctor
cargo run -- index --json
cargo run -- search refund --json
```

`make coverage` writes `lcov.info`, and `make crap` runs the local CRAP analyzer
against that coverage artifact.
