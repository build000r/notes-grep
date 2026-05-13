---
name: apple-notes-organizer
description: Reorganize Apple Notes with a safe, evidence-first workflow using notes-grep/ng. Use when asked to organize, clean up, restructure, classify, archive, empty an Inbox/capture folder, or migrate Apple Notes; design a folder/tag/Smart Folder taxonomy; turn a messy note corpus into Inbox/Active/Areas/Resources/Archive-style structure; or prepare guarded ng folder/note move commands. Works for this notes-grep repo and for other Apple Notes users who can produce an ng JSONL index.
---

# Apple Notes Organizer

Reorganize Apple Notes by separating durable storage from flexible views:
folders hold stable homes, tags express cross-cutting meaning, Smart Folders are
saved filters, and pinned notes are dashboards.

## First Progress Marker

Start the first progress update with:

`Using apple-notes-organizer to <goal>. First I will <next concrete step>.`

## Safety Contract

- Default to plan-only. Do not run any `--apply` command unless the user
  explicitly asks to apply moves in the current turn.
- If the user explicitly asks to empty an Inbox/capture folder, move all notes,
  keep organizing, or "do it" after a concrete dry-run plan, switch to
  full-drain mode: keep classifying, dry-running, applying, reindexing, and
  auditing until the source folder is empty or every remaining note has a
  concrete blocker.
- Always run `ng` dry-runs before writes. A dry-run failure blocks the
  corresponding apply command.
- Do not create, delete, edit, lock, unlock, tag, or convert notes through
  `ng`; this repo currently supports search, folder moves, note moves, open, and
  indexing only.
- Do not convert folders to Smart Folders as part of an automated cleanup.
  Apple's conversion moves notes and tags them with the folder name, and cannot
  be undone.
- Respect account boundaries. `ng` rejects cross-account moves; preserve that
  boundary in the plan instead of trying to work around it.
- Treat shared folders, locked notes, and ambiguous folder paths as human-review
  items.

## Workflow

1. **Read the local contract.** In this repo, read `AGENTS.md`, `README.md`,
   and `docs/CLI_CONTRACT.md` before proposing moves.
2. **Preflight access.**

   ```bash
   ng doctor --json
   ng stats --json
   ng index --json
   ng --json folder list
   ```

   If Full Disk Access or schema access fails, stop and report the blocker.
3. **Audit the corpus.** Resolve this skill directory, then run the helper on
   the warmed cache:

   ```bash
   SKILL_DIR=""
   for d in "./skills/apple-notes-organizer" "./.claude/skills/apple-notes-organizer" "$HOME/.claude/skills/apple-notes-organizer"; do
     [ -f "$d/SKILL.md" ] && { SKILL_DIR="$d"; break; }
   done
   python3 "$SKILL_DIR/scripts/audit_notes_cache.py"
   ```

   Use a custom cache path when `ng index --cache-dir DIR` was used:

   ```bash
   python3 "$SKILL_DIR/scripts/audit_notes_cache.py" DIR/notes.jsonl
   ```
4. **Load the framework.** Read
   `references/apple-notes-organization-principles.md` before drafting the
   taxonomy, use `references/migration-plan-template.md` for the output, and
   copy `assets/templates/migration-spec.yaml` into a run directory when the
   work needs a machine-readable move spec.
5. **Choose the execution mode.**
   - Use **plan mode** when the user asks for advice, a taxonomy, a report, or
     a review without explicit apply authorization.
   - Use **full-drain mode** when the user asks to empty an Inbox/capture
     folder, move everything somewhere, or persist until the cleanup is done.
     In this mode, read `references/full-drain-triage.md` and do not stop after
     the first easy cleanup batch.
6. **Propose a migration plan.** The plan must include current evidence,
   proposed folder tree, tag and Smart Folder recommendations, dry-run commands,
   apply commands kept in a separate gated section, and verification commands.
7. **Apply only after approval.** Apply in small batches: folder moves first,
   reindex, then note moves. After each batch, rerun `ng index --json`, rerun
   the audit helper, and spot-check representative searches.
8. **Complete with a real audit.** Before claiming success, verify final source
   folder counts from a fresh index, dry-run/apply pass/fail counts, total note
   count preservation, and blocker records. Passing commands are evidence only
   if they cover the requested cleanup.

## Organization Heuristics

- Keep the folder tree thin and stable. Prefer 5-9 top-level homes and no more
  than 3 levels unless the existing corpus strongly justifies deeper nesting.
- Preserve a person's existing naming grammar when it works. If the account
  already uses numbered roots, adapt them; if it does not, do not impose numbers
  just for neat sorting.
- Use folders for durable context: active projects, continuing areas,
  reference libraries, admin/finance, people/meetings, logs, someday, and
  archive.
- Use tags for cross-cutting attributes that should cut across folders:
  status, type, person, time horizon, review state, source, or workflow state.
- Use Smart Folders for views over tags/dates/checklists/mentions, not as a
  replacement for the physical folder tree.
- Prefer an "Inbox" or "Triage" area only if the user actually has unsorted
  capture behavior. Do not create a junk drawer that will hide decisions.
- Archive by decision, not age alone. Old active areas can remain active; stale
  project folders should move to Archive when their purpose is closed.
- For full-drain cleanups, leaving notes in the capture folder is failure unless
  the note is explicitly blocked. If evidence is weak but the user authorized
  best-judgment organization, route unclear notes to a durable fallback such as
  `Archive/Raw Captures`, `Reference/Unsorted`, or another existing review
  bucket instead of abandoning the cleanup.
- Classify with increasing recall: exact known entities first, then
  title/snippet signals, then body-text signals, then fallback destinations for
  short, old, imported, or ambiguous captures.

## Plan Requirements

Every plan must answer:

- What is the current shape? Include note count, folder count, top roots, high
  volume folders, one-note folders, flat-prefix candidates, and existing tags.
- What gets a durable folder home, what gets a tag, and what becomes a Smart
  Folder recipe?
- Which moves are safe folder-level moves, which require note-level review, and
  which are outside current `ng` capabilities?
- What evidence supports each proposed move? Use search queries, counts, folder
  inventory, and representative non-sensitive keywords.
- What is the rollback or recovery story? At minimum, preserve the before-state
  audit bundle and run in small batches.
- In full-drain mode, what is the source folder count, how many notes were
  classified, how many dry-runs passed/failed, how many applies passed/failed,
  and what exact blockers remain?

## Apply Gate

Before any `--apply`, show the user:

```text
Apply gate
- Batch:
- Dry-run commands passed:
- Notes/folders affected:
- Known exclusions:
- Verification after apply:
```

Then wait for explicit approval unless the user already gave explicit apply
authorization in the current message.

## Verification

After changes:

```bash
ng index --json
python3 "$SKILL_DIR/scripts/audit_notes_cache.py"
ng --json folder list
```

Also run targeted `ng search QUERY --json` checks for the largest migrated
categories and any edge cases called out in the plan.

## References

- `references/apple-notes-organization-principles.md` - Apple Notes folder,
  tag, Smart Folder, and pin model with source links.
- `references/migration-plan-template.md` - required plan format and apply gate.
- `references/full-drain-triage.md` - persistent one-shot cleanup loop for
  emptying Inbox/capture folders without giving up after easy wins.
- `assets/templates/migration-spec.yaml` - optional machine-readable planning
  packet for larger migrations.
