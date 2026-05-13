# Full-Drain Triage

Use this reference when the user asks to empty an Inbox/capture folder, move all
notes somewhere, keep organizing, or persist until a messy note database is
clean.

## Core Rule

Do not stop after the first obvious move batch. A full-drain cleanup is complete
only when the source folder count is zero, or every remaining item has a
specific blocker such as missing destination support, locked/shared state,
cross-account constraints, or user-deferred review.

The agent's job is not to be perfectly clairvoyant. It is to make the best
available categorization decision, preserve evidence, use guarded dry-runs, and
route low-signal captures to an honest fallback bucket instead of leaving the
Inbox as a permanent junk drawer.

## One-Shot Loop

Repeat this loop until completion:

1. Rebuild or refresh the note index.
2. Count the source folder from the fresh index.
3. Classify every source-folder note into one of:
   - an existing durable destination folder,
   - an existing fallback/review/archive folder,
   - a blocked record with reason.
4. Write the classification artifact to a run directory.
5. Dry-run every candidate move individually.
6. Remove or fix dry-run failures; do not apply failed records.
7. Apply only dry-run-passing records.
8. Reindex and recount.
9. If source count is nonzero, repeat from step 3 with broader rules or
   fallback routing.

Never infer completion from effort, a large moved count, or a successful batch.
Only a fresh source-folder count of zero, plus zero apply failures, proves a
full-drain cleanup finished.

## Classification Ladder

Classify in this order. Earlier levels should win ties unless the later evidence
is much stronger.

1. **Exact entity and project names.** Current clients, projects, products,
   accounts, people, or other named domains.
2. **Title and snippet signals.** Higher precision than body text. Use these
   for most direct moves.
3. **Body-text signals.** Useful for unlabeled notes, but noisy; weight lower
   and preserve evidence snippets.
4. **Structural signals.** Age, import source, very short title/body,
   code-like text, URL-heavy notes, checklist-heavy notes, or attachment-like
   saved-photo titles.
5. **Fallback routing.** If the user authorized best-judgment cleanup and no
   durable category is clear, use an existing fallback such as:
   - `Archive/Raw Captures`
   - `Archive/Imports`
   - `Reference/Unsorted`
   - `Writing/Session Notes`
   - `Someday/Review`

Choose a fallback that honestly represents uncertainty. Do not pretend a vague
capture is a real project note just to make the folder tree look smarter.

## Artifacts To Save

Write these files under a run directory for every full-drain cleanup:

- `classification.json`: one row per source note with note id, title, source,
  target, rule kind, score/confidence, and evidence.
- `dryrun-pass.json` and `dryrun-fail.json`.
- `apply-pass.json` and `apply-fail.json`.
- `final-index.json` or equivalent fresh index/stats output.
- `completion-evidence.json`: source count, total note count, dry-run/apply
  counts, destination summary, and blockers.

These artifacts matter because a note database cleanup is a stateful migration,
not a chat answer.

## Completion Audit

Before saying "done", verify:

- The user's requested source folder exists in the fresh pre-run inventory.
- `classification.json` covers every note that was in the source folder for the
  pass.
- Every applied record appears in a dry-run pass record.
- Dry-run failures are either fixed or excluded with blocker reasons.
- Apply failures are zero, or each failure has a blocker reason and the note is
  still accounted for.
- A fresh index was built after the last apply.
- The source folder count from the fresh index is zero, or the remaining count
  equals the blocker list length.
- Total note count is unchanged unless the requested tool explicitly supports
  creation/deletion and the plan says so.
- The final answer reports the source-folder before/after count, moved count,
  failures, run artifact path, and remaining blockers.

## Generalization Guidance

For any note database, adapt the destination names but keep the shape:

- `Inbox` or `Capture`: temporary collection point, not a home.
- `Active` or `Projects`: current work.
- `Areas`: ongoing life/work responsibilities.
- `Resources` or `Reference`: reusable knowledge.
- `Writing` or `Thinking`: drafts, reflections, session notes.
- `Personal` or `Admin`: receipts, finance, health, home, travel.
- `Archive`: closed, imported, stale, or low-signal material.
- `Raw Captures` or `Review`: honest fallback for weak evidence.

The folder names are not the skill. The skill is the persistent migration loop:
classify all, dry-run all, apply only passed records, reindex, and prove the
source folder is empty.
