# `ng` v0.1 CLI Contract

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
- snippet
- modified timestamp
- decoded body text when a body blob is readable

`--cache-dir DIR` redirects both indexing and warmed search for fixture tests and
agent workflows.

## Exit Codes

| Code | Meaning |
|---:|---|
| 0 | Success. |
| 1 | Generic local IO/serialization failure. |
| 2 | Missing/inaccessible Notes database. |
| 3 | Unrecognized Notes database schema. |
| 4 | Failed to open note URL. |

## v0.1 Non-Goals

- No write/edit/delete.
- No MCP server.
- No semantic search.
- No Tantivy or semantic index until the JSONL body cache is trusted.
- No dependency on `xf` or `frankensearch` source.
