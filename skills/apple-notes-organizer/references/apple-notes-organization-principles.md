# Apple Notes Organization Principles

Use this reference when deciding how folders, tags, Smart Folders, and pins
should work together.

## Source-Grounded Apple Notes Model

- Folders and subfolders are the physical organization layer. Apple supports
  creating folders and subfolders, sorting notes inside folders, and moving
  folders in the sidebar.
  Source: https://support.apple.com/en-asia/guide/notes/apd558a85438/mac
- Apple does not allow subfolders under the built-in "All [account]" or "Notes"
  folders, and notes cannot be moved to "All [account]".
  Source: https://support.apple.com/en-asia/guide/notes/apd558a85438/mac
- Tags are one-word labels that can include hyphens, numbers, and underscores.
  They work across folders and appear in the Tags browser.
  Source: https://support.apple.com/en-us/102288
- Tagged notes and locked notes have constraints: Apple says a note with a tag
  cannot be locked, and a locked note cannot receive a tag.
  Source: https://support.apple.com/en-us/102288
- Smart Folders are views based on criteria such as tags, mentions, checklists,
  creation date, or edit date. They reference notes that remain in their
  original folders.
  Source: https://support.apple.com/guide/notes/use-smart-folders-apd58edc7964/mac
- Smart Folders cannot be locked, turned into subfolders, or shared. Converting
  a normal folder to a Smart Folder moves its notes to Notes and tags them with
  the folder name; Apple says the conversion cannot be undone.
  Source: https://support.apple.com/guide/notes/use-smart-folders-apd58edc7964/mac
- Pins are for very important notes or dashboards. Pinned notes appear at the
  top and sync across devices signed into the same Apple Account.
  Source: https://support.apple.com/en-my/guide/notes/apdb54e469b6/mac

## Working Mental Model

Use four layers:

1. **Folders are homes.** A note gets one durable place: project, area,
   reference library, admin, journal/log, someday, or archive.
2. **Tags are facets.** Tags express facts that cut across homes: status,
   note type, person, source, period, or review state.
3. **Smart Folders are views.** Smart Folders answer repeated questions without
   moving notes: "waiting", "receipts this year", "untagged", "recently edited",
   "checklists", "shared", or "meeting notes".
4. **Pins are dashboards.** A pinned note is a starting point, index, or active
   operating note, not a category.

## Default Folder Skeleton

Do not force this exact tree. Use it as a starting point and adapt to the
person's corpus.

```text
Inbox
Active
  Projects
  Workstreams
Areas
  Personal
  Health
  Home
  Work
Resources
  Reference
  Ideas
People
  Meetings
  Contacts
Admin
  Finance
  Legal
  Receipts
Logs
  Journal
  Daily
Someday
Archive
```

For users who prefer deterministic sorting, use numbered roots sparingly:

```text
00 Inbox
10 Active
20 Areas
30 Resources
40 People
50 Admin
60 Logs
70 Someday
90 Archive
```

Use numbered roots only when the existing account already uses a numbered
system or the user asks for stable ordering.

## Folder Decision Rules

- Create a folder when notes share a durable context and the user would browse
  there intentionally.
- Do not create a folder for a one-off keyword unless it has enough notes,
  ongoing value, or legal/accounting retention meaning.
- Keep archive separate from active work. Archive can mirror active structure
  when needed, but avoid turning Archive into another unsorted inbox.
- Avoid "Misc", "Stuff", and vague catchalls. If a catchall is needed, make it
  temporary and name the triage rule.
- Prefer real nested folders over flat prefix names such as
  `Work - Client - Receipts` when `ng folder mv` can represent the hierarchy.
- Do not mix accounts. Treat each Apple Notes account as an independent corpus.

## Tag Decision Rules

Recommended generic tag families:

```text
#status-active
#status-waiting
#status-reference
#status-review
#type-receipt
#type-meeting
#type-idea
#type-journal
#person-name
#project-name
#year-2026
```

Rules:

- Use kebab-case or underscores consistently; Apple tags must be one continuous
  word.
- Keep the global tag vocabulary small. If a tag will apply to only one note,
  it probably belongs in title/body text instead.
- Do not recommend tags for locked notes unless the plan explicitly says they
  must be unlocked or skipped.
- Because `ng` does not edit note bodies, tag application is a manual Notes UI
  or future tooling step, not an `ng` command.

## Smart Folder Recipes

Good Smart Folders:

- `Review`: `#status-review` or recently edited in the last N days.
- `Waiting`: `#status-waiting`.
- `Receipts`: `#type-receipt` plus date filters.
- `Meeting Notes`: `#type-meeting` or title/body search conventions.
- `Untagged`: Apple's Smart Folder filter for untagged notes.
- `Open Checklists`: checklist criteria when the user uses checklists.

Avoid:

- Converting existing folders to Smart Folders during automated cleanup.
- Smart Folders that duplicate every physical folder.
- Smart Folders that depend on a huge uncontrolled tag vocabulary.

## Quality Rubric

A good Apple Notes organization plan is:

- **Findable:** common lookup questions have a folder, tag, Smart Folder, or
  search path.
- **Low-friction:** capture still works quickly, without requiring 6 decisions
  per note.
- **Stable:** top-level categories do not churn weekly.
- **Auditable:** every move has evidence and a dry-run command.
- **Reversible enough:** before-state inventory exists, changes are small
  batches, and high-risk notes are skipped.
- **Personal but portable:** it reflects the actual corpus without hardcoding
  one person's private taxonomy into the reusable skill.
