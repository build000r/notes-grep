# Migration Plan Template

Use this exact shape when proposing an Apple Notes reorganization.

````markdown
# Apple Notes Organization Plan

## Current Shape
- Notes:
- Folders:
- Accounts:
- Largest root folders:
- Largest leaf folders:
- One-note folders:
- Flat-prefix candidates:
- Existing tag signals:
- Important constraints:

## Target Model
[One paragraph explaining the folder/tag/Smart Folder split.]

## Proposed Folder Tree
```text
[tree]
```

## Move Batches

### Batch 1: [name]
Why:
Evidence:
- [counts/searches]

Dry-run:
```bash
ng folder mv "SOURCE" "TARGET"
```

Apply, only after approval:
```bash
ng folder mv "SOURCE" "TARGET" --apply
```

Verification:
```bash
ng index --json
ng search "QUERY" --folder "TARGET" --json
```

## Note-Level Review Queue

These should not be moved in bulk yet:

| Query or folder | Why review is needed | Proposed destination |
|---|---|---|
| | | |

## Full-Drain Mode

Use when the source folder must be emptied in one sustained run.

- Source folder:
- Starting source count:
- Classification artifact:
- Fallback destination for low-signal captures:
- Dry-run pass/fail files:
- Apply pass/fail files:
- Reindex command:
- Completion evidence file:
- Remaining blockers:

## Tag Recommendations

| Tag | Meaning | How to apply | Smart Folder use |
|---|---|---|---|
| | | | |

## Smart Folder Recipes

| Name | Filters | Purpose | Manual setup notes |
|---|---|---|---|
| | | | |

## Exclusions
- Locked notes:
- Shared folders:
- Cross-account material:
- Ambiguous folders:
- Folders that need to be created manually:

## Apply Gate
- Batch:
- Dry-run commands passed:
- Notes/folders affected:
- Known exclusions:
- Verification after apply:

## Final Verification
```bash
ng index --json
python3 "$SKILL_DIR/scripts/audit_notes_cache.py"
ng --json folder list
```

Completion audit:
- Source folder final count:
- Total notes preserved:
- Classified count equals starting source count:
- Dry-run failures:
- Apply failures:
- Blocked notes:
````
