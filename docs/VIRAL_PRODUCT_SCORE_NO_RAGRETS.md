# Viral Product Score No-Ragrets Plan

Date: 2026-07-04

Source bead: `notes-grep-viral-product-score-flywheel-vgi`

`/viral-product-score` and `/dueling-idea-wizards` are not exposed in this
agent lane, so this is the repo-grounded fallback deliverable: a compact score
read, the strongest product bets, and executable Beads to push each score
toward perfect.

## Product Read

`notes-grep` is a local-first Apple Notes search CLI installed as `ng`. Its
current sharp edge is agent-grade retrieval over a private notes corpus:
`ng index` warms full-body JSONL cache records, `ng search` offers `rg`-style
literal/regex filters plus stable JSON, and folder/note moves are guarded by
dry-run defaults plus explicit `--apply`.

The strongest viral wedge is not broad social sharing. Private notes rarely
want public virality. The best loop is proof-sharing among operators and agents:
show a safe redacted transcript where a messy local note is found, opened, and
acted on in seconds, then make the install path short enough that another Mac
user can repeat it immediately.

## Scorecard

| Dimension | Current | Target | Why |
|---|---:|---:|---|
| First-use wow | 6 | 10 | The CLI has strong commands, but needs a polished proof path that demonstrates body search, JSON output, and open-note handoff without relying on the user's real private data. |
| Share loop | 3 | 9 | Notes are private, so raw result sharing is wrong. The shareable object should be a redacted search receipt: query shape, counts, folders, timing, and optional fake fixture excerpts. |
| Activation friction | 5 | 9 | `cargo install`, Full Disk Access, and Notes DB paths are documented, but first-run diagnosis should feel like a five-minute checklist with exact pass/fail states. |
| Trust and safety | 8 | 10 | Read-only search and explicit write gates are strong. The viral path should preserve that trust by proving no note bodies leave the machine. |
| Agent utility | 7 | 10 | Stable JSON, `--id-only`, `--quiet`, and cache isolation are good. A scripted agent scenario would make the value obvious to other automation users. |

## Dueling Ideas Result

1. Build a visual demo page for the CLI.
2. Add semantic search before launch.
3. Create a redacted "search receipt" plus repeatable first-wow transcript.

Winner: redacted search receipt plus first-wow transcript.

The demo page would help later, but it does not prove the local private-data
promise by itself. Semantic search may be valuable, but the README explicitly
defers it until the JSONL body cache is trusted. The receipt/transcript route
uses the current product, respects privacy, and gives every adopter a concrete
artifact they can share without exposing notes.

## Execution Beads

The follow-up beads created from this plan should deliver:

- A first-wow proof transcript using fixtures or redacted sample data that shows
  `ng doctor`, `ng index`, `ng search --json`, `ng search --id-only`, and
  `ng open` handoff.
- A privacy-safe search receipt format that can be posted or pasted without raw
  note body leakage.
- A five-minute install and activation path that makes Full Disk Access,
  database discovery, cache warming, and first search outcomes unambiguous.
- A launch packet that combines those assets into README/docs changes only
  after the proof and receipt exist.

## Retro Check

This plan avoids the biggest likely regret: optimizing for generic social
sharing when the product's durable trust comes from local privacy. It also
avoids promising deferred search technologies before the existing full-body
cache, folder filters, and agent JSON contract are packaged into a repeatable
adoption loop.
