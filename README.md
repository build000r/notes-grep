# notes-grep

`notes-grep` is a local-first Apple Notes search CLI. The package installs a
short binary named `ng`.

The target shape is: `xf` for Apple Notes, with `rg`-style search ergonomics.
This repo is intentionally not a fork of `xf`; Apple Notes has a different data
model, and the first useful product boundary is a clean read-only Notes search
tool.

## Status

`v0.1` is the read-only full-body search spine:

- `ng doctor` validates access to `NoteStore.sqlite`
- `ng stats` prints basic store counts
- `ng index` writes a JSONL cache with title, snippet, folder, modified time,
  stable note ID, and decoded body text from `ZICNOTEDATA.ZDATA`
- `ng search QUERY` searches the warmed JSONL cache when it exists, falling
  back to direct title/snippet SQLite search before the first index
- `ng open NOTE_ID` opens an `x-coredata://...` note URL with `open`
- `--json` and `--cache-dir` are available for agent use

The body decoder gzip-decodes Apple Notes `ZICNOTEDATA.ZDATA` blobs and walks
their protobuf wire fields for UTF-8 note text. Regex search, Tantivy, semantic
search, attachments, and OCR/table text are deferred until this extraction/cache
path is proven.

## Install Locally

```bash
cargo install --path .
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
ng --cache-dir /tmp/ng-cache index
ng --cache-dir /tmp/ng-cache search "body-only phrase" --json
ng open "x-coredata://.../ICNote/p123"
```

## Design Rules

- Search is read-only by default.
- Output is compact for humans and structured with `--json` for agents.
- Errors should say whether the failure is missing database, missing Full Disk
  Access, unrecognized schema, or a command problem.
- No network calls during search.
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
