#!/usr/bin/env python3
"""Summarize a notes-grep JSONL cache for organization planning."""

from __future__ import annotations

import argparse
import collections
import datetime as dt
import json
import re
import sys
from pathlib import Path
from typing import Any


DEFAULT_CACHE = Path.home() / "Library/Application Support/notes-grep/notes.jsonl"
TOKEN_RE = re.compile(r"[A-Za-z][A-Za-z0-9_-]{2,}")
HASHTAG_RE = re.compile(r"(?<!\w)#([A-Za-z][A-Za-z0-9_-]{1,60})")
FLAT_PREFIX_RE = re.compile(r"\s(?:-|:|>)\s")
STOPWORDS = {
    "about",
    "after",
    "again",
    "also",
    "and",
    "any",
    "are",
    "but",
    "can",
    "for",
    "from",
    "have",
    "how",
    "into",
    "not",
    "notes",
    "the",
    "that",
    "this",
    "todo",
    "with",
    "you",
    "your",
}


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Summarize notes-grep notes.jsonl for Apple Notes organization planning."
    )
    parser.add_argument(
        "cache",
        nargs="?",
        default=str(DEFAULT_CACHE),
        help=f"Path to notes.jsonl (default: {DEFAULT_CACHE})",
    )
    parser.add_argument("--json", action="store_true", help="Emit JSON instead of Markdown.")
    parser.add_argument("--top", type=int, default=20, help="Rows per ranked section.")
    return parser.parse_args()


def load_notes(path: Path) -> list[dict[str, Any]]:
    try:
        lines = path.read_text(encoding="utf-8").splitlines()
    except FileNotFoundError:
        raise SystemExit(f"cache not found: {path}\\nRun `ng index` first.")

    notes: list[dict[str, Any]] = []
    for line_no, line in enumerate(lines, 1):
        if not line.strip():
            continue
        try:
            note = json.loads(line)
        except json.JSONDecodeError as exc:
            raise SystemExit(f"invalid JSON on line {line_no}: {exc}") from exc
        notes.append(note)
    return notes


def folder_path(note: dict[str, Any]) -> str:
    value = note.get("folder_path") or note.get("folder") or "(unfiled)"
    value = str(value).strip()
    return value or "(unfiled)"


def root_folder(path: str) -> str:
    if path.startswith("("):
        return path
    return path.split("/", 1)[0]


def parse_modified(value: Any) -> dt.datetime | None:
    if not value:
        return None
    text = str(value)
    for fmt in ("%Y-%m-%d %H:%M:%S", "%Y-%m-%dT%H:%M:%S"):
        try:
            return dt.datetime.strptime(text[:19], fmt)
        except ValueError:
            continue
    return None


def note_tokens(note: dict[str, Any]) -> set[str]:
    text = " ".join(
        str(note.get(key) or "")
        for key in ("title", "folder", "folder_path", "snippet", "body")
    )
    tokens = {
        token.lower().strip("_-")
        for token in TOKEN_RE.findall(text)
        if token.lower() not in STOPWORDS and not token.isdigit()
    }
    return {token for token in tokens if len(token) >= 3}


def note_hashtags(note: dict[str, Any]) -> set[str]:
    text = " ".join(str(note.get(key) or "") for key in ("title", "snippet", "body"))
    return {tag.lower() for tag in HASHTAG_RE.findall(text)}


def top_rows(counter: collections.Counter[str], limit: int) -> list[dict[str, Any]]:
    return [{"name": name, "count": count} for name, count in counter.most_common(limit)]


def build_summary(notes: list[dict[str, Any]], cache: Path, top: int) -> dict[str, Any]:
    folders = collections.Counter(folder_path(note) for note in notes)
    roots = collections.Counter(root_folder(path) for path in folders.elements())
    leaf_folders = collections.Counter(folders)
    tags: collections.Counter[str] = collections.Counter()
    tokens: collections.Counter[str] = collections.Counter()
    modified_values: list[dt.datetime] = []

    for note in notes:
        tags.update(note_hashtags(note))
        tokens.update(note_tokens(note))
        modified = parse_modified(note.get("modified"))
        if modified:
            modified_values.append(modified)

    one_note = collections.Counter(
        {folder: count for folder, count in folders.items() if count == 1}
    )
    flat_candidates = collections.Counter(
        {folder: count for folder, count in folders.items() if FLAT_PREFIX_RE.search(folder)}
    )
    body_notes = sum(1 for note in notes if note.get("body"))
    missing_folder = sum(1 for note in notes if not (note.get("folder_path") or note.get("folder")))

    modified_range: dict[str, str | None] = {"oldest": None, "newest": None}
    if modified_values:
        modified_range = {
            "oldest": min(modified_values).strftime("%Y-%m-%d %H:%M:%S"),
            "newest": max(modified_values).strftime("%Y-%m-%d %H:%M:%S"),
        }

    return {
        "cache": str(cache),
        "notes": len(notes),
        "body_notes": body_notes,
        "folders": len(folders),
        "missing_folder": missing_folder,
        "modified_range": modified_range,
        "top_roots": top_rows(roots, top),
        "top_folders": top_rows(leaf_folders, top),
        "one_note_folders": top_rows(one_note, top),
        "flat_prefix_candidates": top_rows(flat_candidates, top),
        "hashtags": top_rows(tags, top),
        "keywords": top_rows(tokens, top),
    }


def print_rows(rows: list[dict[str, Any]]) -> None:
    if not rows:
        print("- None found")
        return
    for row in rows:
        print(f"- {row['name']}: {row['count']}")


def print_markdown(summary: dict[str, Any]) -> None:
    print("# Apple Notes Corpus Audit")
    print()
    print(f"- Cache: `{summary['cache']}`")
    print(f"- Notes: {summary['notes']}")
    print(f"- Notes with decoded body: {summary['body_notes']}")
    print(f"- Folders: {summary['folders']}")
    print(f"- Notes without folder metadata: {summary['missing_folder']}")
    print(f"- Modified range: {summary['modified_range']['oldest']} -> {summary['modified_range']['newest']}")
    print()
    print("## Top Root Folders")
    print_rows(summary["top_roots"])
    print()
    print("## Top Leaf Folders")
    print_rows(summary["top_folders"])
    print()
    print("## One-Note Folders")
    print_rows(summary["one_note_folders"])
    print()
    print("## Flat-Prefix Folder Candidates")
    print_rows(summary["flat_prefix_candidates"])
    print()
    print("## Existing Hashtags")
    print_rows(summary["hashtags"])
    print()
    print("## High-Signal Keywords")
    print_rows(summary["keywords"])
    print()
    print("## Planning Notes")
    print("- Use these facts as evidence, not as an automatic migration plan.")
    print("- Review one-note folders before collapsing them; some may be valid projects.")
    print("- Treat flat-prefix candidates as possible nested-folder migrations.")
    print("- Existing hashtags suggest the user's natural tag vocabulary.")


def main() -> int:
    args = parse_args()
    cache = Path(args.cache).expanduser()
    notes = load_notes(cache)
    summary = build_summary(notes, cache, args.top)
    if args.json:
        print(json.dumps(summary, indent=2, sort_keys=True))
    else:
        print_markdown(summary)
    return 0


if __name__ == "__main__":
    sys.exit(main())
