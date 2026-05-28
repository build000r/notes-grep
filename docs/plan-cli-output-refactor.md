# CLI Output Contract Refactor Plan

Status: planned 2026-05-28, after profiling confirms no performance concerns

## Problem

Each subcommand in `cli.rs` has a duplicated pattern:
1. Build a view struct
2. Branch on `--json` flag
3. In the JSON branch: `serde_json::to_string_pretty` + `println!`
4. In the human branch: manual `println!` of each field

This works but has two costs:
- Every new subcommand copies the if/else JSON/human pattern
- Human output strings are not tested (only JSON output is machine-parseable)

## Current output contracts

| Command | Human output | JSON output |
|---|---|---|
| `ng` (home) | `ng: {status}` / `db:` / `next:` | HomeView struct |
| `ng doctor` | `status:` / `db:` / `notes:` / `folders:` / `next:` | DoctorView struct |
| `ng stats` | `notes:` / `folders:` / `accounts:` | StoreStats struct |
| `ng index` | `index:` / `notes:` / `body-notes:` / `cache:` / `scope:` | IndexView struct |
| `ng search` | `hits:` header + per-hit lines | Vec\<NoteHit\> |
| `ng folder list` | `folders:` header + per-folder lines | Vec\<FolderEntry\> |
| `ng folder mv` | `folder-move:` / `source:` / `target:` / etc. | FolderMoveView struct |
| `ng note mv` | `note-move:` / `note-id:` / `title:` / etc. | NoteMoveView struct |

## Proposed refactor

Extract a trait or helper that each view struct implements:

```rust
fn emit<T: Serialize + HumanDisplay>(view: &T, json: bool) {
    if json {
        println!("{}", serde_json::to_string_pretty(view).unwrap());
    } else {
        view.print_human();
    }
}
```

Each view struct gets a `print_human()` method (or `Display` impl). The
if/else disappears from every subcommand handler.

## Stability constraints

- Human output format must not change in a breaking way (scripts may parse it)
- JSON output is the stable contract; human output is best-effort
- `--json` flag remains global and works on all subcommands

## Scope

This is a refactor only — no new features, no new subcommands, no behavioral
changes. Tests should assert both human and JSON outputs remain identical
before and after.
