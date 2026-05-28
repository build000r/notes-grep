# Hot-Path Profiling Results

Recorded: 2026-05-28  
Binary: release build (optimized)  
Dataset: 6469 notes, 234 folders, 1 account  
Cache: 6.2 MB JSONL (6375 body-extracted notes, 98.5% coverage)

## Benchmarks

| Operation | Wall time | Notes |
|---|---|---|
| `ng index` (full re-index) | 426 ms | Extracts + writes 6375 bodies to JSONL |
| `ng search` (warmed, broad "the", 50 hits) | 12 ms | Linear scan of 6.2 MB cache |
| `ng search` (warmed, narrow multi-word, 0 hits) | 23 ms | Full scan, no matches |
| `ng search` (fallback, title/snippet SQLite) | 28 ms | No cache present, direct DB query |
| `ng folder list` (234 folders) | 15 ms | Single SQLite query |
| `ng folder mv` (dry-run, 6 descendants, 189 notes) | 16 ms | Plan-only, no write |
| `ng note mv` (dry-run) | 15 ms | Plan-only, no write |

## Analysis

All operations complete well under 1 second. The index operation (426 ms) is the heaviest,
dominated by zlib decompression of Apple Notes protobuf body blobs. Search, list, and
move operations are all under 30 ms regardless of query selectivity.

The warmed-cache search performs a linear scan of the JSONL file. At the current dataset
size (6.2 MB) this is fast enough that an inverted index would add complexity without
meaningful user-perceived improvement. If the cache grows past ~50 MB (roughly 50k notes),
a Tantivy or similar index would be worth evaluating.

Fallback search (no cache, SQLite title/snippet only) is comparable in latency to
warmed-cache search, confirming the SQLite path is not a bottleneck.

Move operations (folder and note) are guard-heavy (account validation, descendant
enumeration, conflict checks) but all guards resolve in a single SQLite round-trip.
