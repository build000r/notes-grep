# `ng` CLI Contract

## Driver

Human plus coding agent. The common path is read-heavy: inspect status, search,
open a note, then decide whether to refine the query.

## Home View

`ng` with no arguments prints a compact live status:

- status
- database path
- next useful commands

`ng --json` emits the same information as JSON.

## Commands

| Command | Contract |
|---|---|
| `ng doctor` | Validate database existence, schema access, and print counts. |
| `ng stats` | Print basic note/folder/account counts. |
| `ng index` | Write a JSONL full-body cache under the local data directory. |
| `ng search QUERY` | Return warmed cache matches, falling back to title/snippet SQLite search when no cache exists. |
| `ng search QUERY --json` | Return stable JSON for agents. |
| `ng folder list` | Return account-prefixed nested folder paths. |
| `ng folder mv SOURCE TARGET` | Dry-run a same-account folder rename or nested move. |
| `ng folder mv SOURCE TARGET --apply` | Apply a guarded same-account folder rename or nested move. |
| `ng note mv NOTE_ID FOLDER` | Dry-run moving one active note to an existing same-account folder. |
| `ng note mv NOTE_ID FOLDER --apply` | Apply the guarded note folder move. |
| `ng open NOTE_ID` | Open an `x-coredata://...` note URL with macOS `open`. |

## Index Contract

`ng index` reads `ZICCLOUDSYNCINGOBJECT.ZNOTEDATA`, joins it to
`ZICNOTEDATA.Z_PK`, gzip-decodes `ZICNOTEDATA.ZDATA`, walks protobuf wire fields
for UTF-8 note text, and writes one JSONL record per note.

Each cache record includes:

- stable `x-coredata://.../ICNote/p...` ID
- database primary key
- title
- folder
- folder_path
- account_path
- snippet
- modified timestamp
- decoded body text when a body blob is readable

`--cache-dir DIR` redirects both indexing and warmed search for fixture tests and
agent workflows. Warmed search reads a cache only when its manifest was written
for the selected database path; otherwise it falls back to direct SQLite search
against the selected database.

## Exit Codes

| Code | Meaning |
|---:|---|
| 0 | Success. |
| 1 | Generic local IO/serialization failure. |
| 2 | Missing/inaccessible Notes database. |
| 3 | Unrecognized Notes database schema. |
| 4 | Failed to open note URL. |

## Folder Move Contract

Folder paths use `/` as the nested container separator. `SOURCE` may be either
an unprefixed path such as `Finance/Receipts` or an account-prefixed path such
as `iCloud/Finance/Receipts` when disambiguation is needed.

`TARGET` is the final path for the source folder:

- `ng folder mv Finance Money` renames `Finance` to `Money` in place.
- `ng folder mv Finance Personal/Finance` moves `Finance` under `Personal`.
- `ng folder mv Finance/Receipts Archive/2026/Receipts` moves a nested folder
  under `Archive/2026`.

The command dry-runs by default and writes only with `--apply`. It rejects
cross-account moves, cycles, and duplicate sibling names before writing.

`ng search --folder` accepts the legacy folder title, nested `folder_path`, or
account-prefixed `account_path` printed by `ng folder list`. Warmed caches
created before these path fields exist must be rebuilt with `ng index` before
nested or account-prefixed filters can match cached results.

## Note Move Contract

`ng note mv NOTE_ID FOLDER` is the only note-level write primitive. It dry-runs
by default and writes only with `--apply`.

`NOTE_ID` should be the stable `x-coredata://.../ICNote/p...` ID returned by
`ng search --json` or persisted in `ng index` JSONL records. Numeric database
IDs are accepted only when the selected database resolves the value to exactly
one active note.

`FOLDER` must already exist and is resolved with the same nested
account-prefixed path rules as `ng folder mv`. The command rejects missing
folders, ambiguous unprefixed folder paths, deleted folders, and target folders
outside the note's current account.

Dry-run JSON and human output report:

- note ID
- note database ID
- note title
- source folder path
- target folder path
- changed
- applied

`--apply` opens the database read-write, updates exactly one active note's
`ZFOLDER`, and increments the minimal local sync bookkeeping used by this CLI
without changing title, body, snippet, or note text.

Applied moves do not update existing warmed cache files in place. Rebuild with
`ng index` after `ng note mv ... --apply` before relying on cached
`ng search --folder` results.

## Current Non-Goals

- No note edit/delete.
- No cross-account folder moves.
- No folder creation or deletion.
- No MCP server.
- No semantic search.
- No Tantivy or semantic index until the JSONL body cache is trusted.
- No dependency on `xf` or `frankensearch` source.
