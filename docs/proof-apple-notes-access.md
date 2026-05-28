# Apple Notes Database Access Proof

Recorded: 2026-05-28

## Full Disk Access

macOS Full Disk Access is granted. `ng doctor` reads the live NoteStore.sqlite directly.

## ng doctor

```
status: ok
db: ~/Library/Group Containers/group.com.apple.notes/NoteStore.sqlite
notes: 6469
folders: 234
next: ng search "query"
```

## ng stats

```
notes: 6469
folders: 234
accounts: 1
```

## ng index

```
index: ok
notes: 6469
body-notes: 6375
cache: ~/Library/Application Support/notes-grep/notes.jsonl
scope: title+snippet+body cache
```

6375 of 6469 notes have extractable body content (98.5%).

## ng search (warmed cache)

```
hits: 20  (default limit, query: "test")
```

Search returns results with coredata URIs, folder paths, titles, and body snippets.

## Permission blocker

None. Full Disk Access is active and all commands succeed without error.
