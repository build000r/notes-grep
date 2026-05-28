# Search Engine Layer Decision

Status: decided 2026-05-28, after real DB proof and profiling

## Context

Real Apple Notes DB access is proven (6469 notes, 234 folders, Full Disk Access
granted). Profiling shows warmed-cache linear search at 12–23 ms on a 6.2 MB
JSONL cache (6375 body-extracted notes). Fallback SQLite title/snippet search
runs in 28 ms.

## Decision: stay with linear JSONL scan for now

The current linear scan is fast enough for the observed dataset size. Adding
Tantivy or a similar inverted index would introduce ~3 new crate dependencies,
a persistent index directory, and an incremental-update protocol — all for a
user-perceived improvement of single-digit milliseconds on today's data.

## Trigger to revisit

Re-evaluate when any of these occur:
- Cache exceeds ~50 MB (roughly 50k notes)
- Users report perceptible latency on search
- Regex search is needed (linear scan with `regex` crate is O(n) but still
  fast; Tantivy adds structured field queries and ranking)
- Semantic/embedding search is requested (requires a vector store, which is a
  separate layer regardless of the text index)

## Local-first / no-network contract

Whatever search layer is chosen must:
- Work entirely offline with no network calls
- Read only from the local NoteStore.sqlite and local cache files
- Never phone home, sync externally, or require an API key
- Keep the cache in `~/Library/Application Support/notes-grep/`

## Next steps

- No search engine work needed now
- If regex support is added, use the `regex` crate on the existing linear scan
- If ranking is needed, consider Tantivy as a compiled-in index (no separate
  server process)
