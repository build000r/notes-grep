# Changelog

All notable changes to `notes-grep` are tracked here.

Scope window: initial public package history through the `0.1.0` release
candidate on 2026-05-12.

Evidence sources: local git history, Beads issue history in `.beads/issues.jsonl`,
`Cargo.toml`, `README.md`, `docs/CLI_CONTRACT.md`, and the release validation
commands recorded for `notes-grep-hw0`.

## Version Timeline

| Version | Date | State | Evidence |
|---|---:|---|---|
| 0.2.0 | 2026-06-10 | In development | Search ergonomics: regex, count, id-only, quiet, recursive folder filter |
| 0.1.0 | 2026-05-12 | Release candidate; tag `v0.1.0` will be created after validation | Cargo package metadata and validation ladder |

## 0.2.0 - 2026-06-10

### Delivered capability: rg-style search ergonomics

- Added `--regex/-e` for case-insensitive regex search across title, snippet,
  and body text (warmed cache and SQLite fallback paths).
- Added `--count/-c` to print only the match count.
- Added `--id-only/-l` to print one stable `x-coredata://` note ID per line.
- Added `--quiet/-q` to suppress output and signal matches via exit code (0 on
  match, 1 on no match), matching `grep`/`rg` conventions.
- Changed `--folder` to match notes in the named folder and all its subfolders.
- Added `--after DATE` and `--before DATE` to filter search results by
  modification date.
- Added `--sort date` (newest first) and `--sort title` (alphabetical) to
  control result ordering.
- Added `--no-snippet` to suppress snippet lines in human search output.
- Optimized case-insensitive matching with an ASCII fast path that avoids
  per-note allocation.

## 0.1.0 - 2026-05-12

### Delivered capability: local Apple Notes search CLI

- Added the `ng` Rust CLI for local-first Apple Notes search.
- Added `ng doctor` and `ng stats` for Notes database access checks and store
  counts.
- Added full-body indexing from `ZICNOTEDATA.ZDATA` into a warmed JSONL cache.
- Added `ng search` with warmed-cache body search and direct SQLite fallback.
- Added `ng open` for stable `x-coredata://.../ICNote/p...` note URLs.

### Delivered capability: guarded folder and note operations

- Added `ng folder list` with account-prefixed nested folder paths.
- Added guarded same-account `ng folder mv` dry-run/apply workflows.
- Added guarded `ng note mv` dry-run/apply workflows for moving one active note
  to an existing same-account folder.
- Covered rejection paths for cycles, duplicate siblings, deleted folders,
  missing notes, ambiguous folders, and cross-account targets.

### Delivered capability: public crate release boundary

- Added crates.io package metadata to `Cargo.toml`.
- Added an explicit `include` list so operational repo files such as `.beads/`,
  `.codex/`, `.mcp.json`, local databases, caches, and agent state do not ship
  in the crate archive.
- Added CI coverage for `cargo fmt --check`, clippy, tests, and publish
  dry-runs on the active `master` branch and release tags.
- Documented the CLI contract and current non-goals for deferred regex, Tantivy,
  semantic search, MCP, folder creation/deletion, and note edit/delete surfaces.

### Representative commits

- [`e35a3d0`](https://github.com/build000r/notes-grep/commit/e35a3d0) added the
  first full-body indexing and search spine.
- [`8cb8050`](https://github.com/build000r/notes-grep/commit/8cb8050) added
  folder listing, folder moves, and note moves.
- [`b9deec4`](https://github.com/build000r/notes-grep/commit/b9deec4) documented
  the folder and note move command contracts.
- [`c08c874`](https://github.com/build000r/notes-grep/commit/c08c874) adopted
  Beads as the checked-in issue/work evidence system.

### Completed workstreams

- `notes-grep-sbp-skill-mcp-recalibration-jut`: SBP skill visibility, MCP parity,
  and Beads initialization evidence.
- `notes-grep-post-sbp-reality-smart-goal-5kl`: post-SBP reality check and smart
  goal contract for the release lane.

### Active release gate

- `notes-grep-hw0`: 0.1.0 release evidence, package boundary, CI, changelog, and
  detached-review gate. This Bead remains active until final clean-worktree
  publish dry-run, commit, tag, and push complete.
