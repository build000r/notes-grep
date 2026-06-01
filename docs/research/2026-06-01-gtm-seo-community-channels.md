# GTM Research: SEO and Community Channels for notes-grep

Retrieval date for live web, GitHub, Hacker News, and registry data: 2026-06-01.

Oracle / Deep Research note: the escalation gate fired for this bead because the
decision depends on current market reality. `oracle` was installed and the local
Deep Research route guard reported same-tab routing support, but two Oracle
browser sessions were already running. To avoid interfering with those tab-local
jobs, this report uses direct current-source research instead of launching a new
Deep Research session.

## Executive Summary

- The single highest-leverage first move is a Hacker News `Show HN` plus a
  canonical README/docs page titled "Search Apple Notes from the terminal with
  ng". The closest Apple Notes case, `apple-notes-liberator`, reached 563 HN
  points and 774 GitHub stars in the first 30 days after its 2023 Show HN.
- The core SEO cluster should be "search Apple Notes from terminal / command
  line / script", not generic "Apple Notes export". Export queries have more
  visible demand but more direct incumbents; terminal/search/script queries have
  lower volume and clearer fit for a GitHub/docs page.
- The top community target is Hacker News first, then r/macapps / r/MacOS only
  after the install story is polished, then Obsidian/PKM venues as "use Apple
  Notes as capture, query it locally" rather than as an Obsidian competitor.
- MCP is a real distribution tailwind, but it should follow the first community
  push rather than lead it. Evidence shows multiple Apple Notes MCP projects and
  strong MCP ecosystem attention, but Apple Notes MCP HN launches have weak HN
  response so far.
- GEO / AI-answer strategy should make the README easy to cite: exact query
  headings, safety claims, install snippets, `--json` examples, comparison
  tables, and a short "Apple Notes has no public Notes API" section sourced to
  Apple Developer Forums.

## Section 1 - Search Query Opportunity

Search-volume estimates below are order-of-magnitude judgments from visible
query surfaces, GitHub/HN demand, and current search result quality, not paid
keyword-tool data. Treat them as [INFERRED].

| Query | Intent type | Estimated monthly searches | Current top/result surface observed | AI-answer coverage | Ranking difficulty for a GitHub page |
| --- | --- | ---: | --- | --- | --- |
| `search apple notes command line` | Developer tooling | 10-100 [INFERRED] | Generic web results surfaced Alto Notes and Apple Developer Forum/API discussion before a focused CLI | Likely sparse; current public pages mix apps, APIs, and export | Low-medium because exact CLI intent is under-served |
| `apple notes full text search` | Power-user productivity | 100-1k [INFERRED] | Alto Notes markets full-text search; Alfred workflow also claims body search | Likely answers suggest app search/export tools, not `rg`-style CLI [INFERRED] | Medium because GUI apps can rank |
| `apple notes api developer` | Developer API | 100-1k [INFERRED] | Apple Developer Forums answer says there is no Notes API and AppleScript is macOS-only | High likelihood AI answers cite Apple forum / AppleScript | Medium; a docs page can rank if framed as "no public API, local CLI alternative" |
| `apple notes sqlite` | Developer / recovery | 100-1k [INFERRED] | Ask Different and blog/forum pages about `NoteStore.sqlite`, truncation, and hidden data | Likely incomplete because body blobs are opaque [INFERRED] | Medium-high; technical pages exist |
| `export apple notes terminal` | Export / migration | 100-1k [INFERRED] | Exporter, Apple Notes Exporter, Obsidian migration threads, and GitHub scripts | Stronger existing coverage from exporters | High; not the best first wedge for `ng` |
| `apple notes script` | Automation | 100-1k [INFERRED] | AppleScript/JXA examples and Apple forum guidance | Likely broad and AppleScript-oriented | Medium; only relevant as comparison |
| `chat with apple notes` | Agent / AI | 10-100 [INFERRED] | NoteChat, Apple Notes MCP directories, and RAG demos | Growing but noisy; answers likely mention MCP servers | Medium; content must say exactly what exists |
| `apple notes mcp` | Agent / AI | 10-100 [INFERRED] | MCP directories list RafalWilinski, sirmews, disco-trooper, Conare entries | Likely stronger because MCP directories are indexable | Medium; `ng` needs an MCP surface before owning this |

Recommended primary cluster:

"Search Apple Notes from the terminal" should be the first cluster. It is close
to `ng`'s real product boundary, avoids competing with mature export apps, and
lets the docs explain the hard technical reason the tool matters: Apple Notes
has no public Notes API, AppleScript/JXA is limited, and full note bodies require
local datastore/protobuf handling.

Recommended content angle:

Title: "Show HN: ng - ripgrep-style search for Apple Notes"

Supporting docs page: "Search Apple Notes from the terminal with ng"

Demo: index a large local Notes database, run exact body searches, filter by
folder, return `--json`, and `ng open` one result. Include the Full Disk Access
setup and a no-network/read-only search guarantee in the first screen.

## Section 2 - GEO / AI-Answer-Engine Gap Analysis

Direct AI-engine answer testing was not available in this run. The claims below
are therefore based on public searchable pages that answer engines can index and
cite, plus current search-result gaps.

What answer engines likely say today:

- For "is there an Apple Notes API", the strongest public answer is Apple's
  Developer Forums: Apple DTS answered "No" to a CRUD API question and pointed
  to AppleScript on macOS plus enhancement requests for a real API.
- For "Apple Notes MCP", public directories already expose multiple answers:
  RafalWilinski's semantic/RAG MCP, sirmews' archived read/search MCP, Conare
  listings for Apple Notes MCP packages, and third-party setup guides.
- For "search Apple Notes command line", public results are fragmented across
  Alto Notes, Alfred workflows, AppleScript tools, exporters, and SQLite
  troubleshooting. [INFERRED] A precise GitHub README with exact-match headings
  can become the clearest answer for the CLI/script intent.
- For "Apple Notes SQLite full body", many pages mention `NoteStore.sqlite`, but
  fewer explain the compressed/protobuf body problem and how a CLI exposes
  stable IDs and JSON output. That is a real GEO gap for `ng`.

README/docs changes to improve citation probability:

- Add exact H2 headings for target questions:
  - "Search Apple Notes from the command line"
  - "Search Apple Notes from scripts with JSON output"
  - "Does Apple Notes have an API?"
  - "How ng reads NoteStore.sqlite"
  - "Full Disk Access and local-only safety"
  - "Apple Notes MCP roadmap"
- Add a compact comparison table:
  `ng` vs AppleScript/JXA vs Alfred workflow vs export apps vs MCP servers.
  Columns should be CLI, full-body search, JSON output, local-only, write
  behavior, install path, and best use.
- Add copy-pasteable examples that mirror likely AI prompts:
  `ng index`, `ng search "phrase" --folder Work --json`, `ng open NOTE_ID`.
- State non-goals plainly: no cloud sync, no note edit/delete, no general note
  app replacement, no Obsidian/Notion comparison.
- Add source-backed caveats: no public Apple Notes API; private datastore may
  drift across macOS versions; Full Disk Access is required.

Documented examples of similar niche tools getting AI-search citation:

- [NOT FOUND] I did not find primary evidence proving a niche macOS CLI gained
  adoption specifically because ChatGPT/Perplexity/Claude cited it.
- [INFERRED] MCP directory listings behave like answer-engine surfaces because
  they aggregate install commands and capability summaries for AI-tool users,
  but they are not proof of AI-answer citation.

## Section 3 - PKM and Developer Community Map

| Community | Platform | Size | Receptiveness to CLI tools | Evidence of driving adoption | Notes |
| --- | --- | ---: | --- | --- | --- |
| Hacker News | HN | Large developer audience | High if the demo is concrete and technically novel | `apple-notes-liberator` Show HN: 563 points, 32 top-level child records, 774 stars in first 30 days | Best first launch surface |
| r/macapps | Reddit | ~120k [INFERRED from public subreddit summaries] | Medium-high for polished Mac utilities; strict posting rules visible in removed-post snippets | Public threads discuss Apple Notes exporters and Mac utility discovery | Post after README/install/FDA docs are polished |
| r/MacOS | Reddit | Large [NOT VERIFIED] | Medium; practical Mac utility posts can work but CLI-only pitch may be narrow | Public Apple Notes export/search frustration threads exist | Frame as local Mac power-user tool, not AI hype |
| r/AppleNotes | Reddit | [NOT FOUND] public JSON blocked | Medium if focused on solving search/export pain; lower if terminal-only | Search results surface Apple Notes limitations and export frustration | Use a softer, non-promotional help-post framing |
| r/ObsidianMD | Reddit | 327k per GummySearch page; conflicting 177.7k on another aggregator | Medium for migration/import/local-file topics; low for replacing Obsidian | Obsidian forum has long Apple Notes import/export threads and users with 2k+ notes | Frame as "keep Apple Notes capture, query locally" |
| Obsidian Forum | Forum | Active PKM community | Medium for migration/import utilities; lower for Apple Notes-only CLI | Apple Notes import threads from 2020 onward remain indexed and active | Good for a technical writeup, not first launch |
| Ness Labs / Tools for Thought | Newsletter/blog/community | Large PKM audience [NOT VERIFIED] | Low-medium for CLI tools; higher for story-driven knowledge workflows | Ness Labs covers tools for thought founders, not many CLI utilities | Better after a polished narrative demo |
| MCP directories | Web registries | Fast-growing ecosystem [INFERRED from MCP repo stars] | High if an MCP server exists | mcp.umin.ai, mcpservers.org, Conare, PulseMCP already list Apple Notes MCP servers | Submit after read-only MCP wrapper ships |
| X/Twitter Mac productivity accounts | Social | [NOT VERIFIED] | Medium for short demos | No primary, stable account/post evidence collected | Do not make this the evidence backbone |

Prioritized outreach sequence:

1. Hacker News `Show HN`: "ng - ripgrep-style search for Apple Notes".
   Frame around local/private search, thousands of notes, JSON for agents, and
   the technical body-decoding problem. Success metric: >50 HN points, >100
   GitHub stars in 30 days, and at least five concrete issue/feature requests.
2. README/docs SEO page submitted to GitHub and linked from the HN post.
   Success metric: appears in search results for exact-title queries within
   30-60 days; GitHub traffic shows search referrals. [INFERRED]
3. r/macapps and r/MacOS after packaging is less Rust-only. Frame as a small
   local Mac utility with Full Disk Access caveats. Success metric: comments
   from Apple Notes users about real libraries, not only "cool project".
4. Obsidian Forum / r/ObsidianMD as an import-adjacent workflow: "Keep fast
   capture in Apple Notes, query from terminal, export only when needed."
   Success metric: users comparing against exporter/importer workflows.
5. MCP directories after a thin read-only `ng` MCP exists. Success metric:
   accepted listings in at least three directories and README referrals.
6. PKM newsletters/blogs after there is a narrative case study. Success metric:
   one external mention; this is not first-week leverage.

## Section 4 - MCP Server as Distribution Channel

MCP registries and directories with visible traction:

- Official `modelcontextprotocol/servers`: 86,582 stars and 10,892 forks,
  retrieved 2026-06-01.
- Official `modelcontextprotocol/registry`: 6,885 stars and 840 forks,
  retrieved 2026-06-01.
- Official SDKs show strong developer interest:
  `modelcontextprotocol/python-sdk` had 23,198 stars and
  `modelcontextprotocol/typescript-sdk` had 12,580 stars, retrieved
  2026-06-01.
- Third-party directories already index Apple Notes MCP entries:
  mcp.umin.ai, mcpservers.org, playbooks.com, Conare, mcpmarket, PulseMCP,
  and mcp.so-style category pages.

Apple Notes MCP evidence:

| Project / listing | Capability signal | Adoption signal | Implication for `ng` |
| --- | --- | --- | --- |
| RafalWilinski/mcp-apple-notes | Semantic search/RAG over Apple Notes, LanceDB, local embeddings, JXA | 393 stars, 51 forks; created 2024-12-16; pushed 2024-12-17; Show HN got 1 point | MCP interest exists, but HN did not reward MCP-only launch |
| sirmews/apple-notes-mcp | Read/search local database for Claude Desktop | 128 stars, 20 forks; archived; README lists missing encrypted notes, pinned notes, sync status, attachments, checklist, create/edit | Opportunity for a maintained read/search backend remains |
| kzaremski/apple-notes-exporter v2 | CLI plus MCP exporter tools | 555 stars, active 2026-05-12 | Exporter market is adding MCP; `ng` should not ignore it |
| Conare / mcpservers / mcp.umin listings | Install docs and capability summaries | Directory presence, not usage proof | Useful after a real `ng` MCP wrapper exists |

Evidence for MCP driving adoption:

- MCP as a protocol has major ecosystem attention; the official repos and SDKs
  have high GitHub star counts.
- Apple Notes MCP projects can reach 100-400 GitHub stars quickly, but the
  visible HN launches around Apple Notes MCP and Apple Notes CLI agents were
  weak: 1-2 points in the HN data collected.
- [NOT FOUND] I did not find primary evidence that publishing an MCP server
  reliably drives star growth for the underlying CLI independent of a strong
  demo or existing audience.

Recommendation:

Ship the first community push before the MCP server. The launch should make
`ng` known as the trustworthy local Apple Notes search CLI. Then ship a thin,
read-only MCP wrapper over `ng search`, `ng open`, folders, and stable IDs.
Semantic MCP should wait until the lexical search/index layer is stronger.

## Section 5 - Content Format Benchmarks

| Format | Platform | Evidence of adoption lift | Effort level | Best adjacent example |
| --- | --- | --- | --- | --- |
| `Show HN` technical launch | Hacker News | Apple Notes Liberator: 563 points; 774 stars in first 30 days after launch | Medium | "Show HN: Apple Notes Liberator - Extract Notes.app Data and Save It as JSON" |
| Canonical technical README/docs page | GitHub/docs | GitHub pages are current top answers for several Apple Notes tooling searches [INFERRED from search results] | Low-medium | `osxphotos` README with demo GIF, install, docs, limitations |
| Terminal demo GIF/screencast | README/HN/social | `osxphotos` uses a README screencast and has 3,596 stars; causality not proven | Medium | `osxphotos` README demo GIF |
| Reddit utility post | r/macapps/r/MacOS | Public threads discuss Apple Notes export pain; subreddit posting rules can remove posts | Medium | Apple Notes Exporter / exporter discussion threads |
| Obsidian migration/help post | Obsidian Forum | Long-running Apple Notes import/export threads show demand from thousands-note users | Medium | Obsidian forum Apple Notes import threads |
| MCP directory listing | MCP registries | Many Apple Notes MCP listings exist; usage lift not verifiable | Low after MCP exists | RafalWilinski/sirmews/Conare listings |
| YouTube short demo | YouTube | [NOT FOUND] direct evidence for niche macOS CLI star lift | Medium-high | Not recommended as first asset |
| X/Twitter screen recording | X | [NOT FOUND] stable primary evidence collected | Low-medium | Use only as support |
| Homebrew tap/formula | Homebrew | `nb` formula had 147 installs in 30 days and 3,060 in 365 days; `duf` had 1,114 in 30 days and 15,659 in 365 days | Medium | Homebrew formulae for `nb` and `duf` |

Recommended first content asset:

Post title: "Show HN: ng - ripgrep-style search for Apple Notes"

Hook:

"I had thousands of Apple Notes and no good way to search them from scripts or
agents. `ng` indexes the local `NoteStore.sqlite`, decodes note bodies, and
gives `rg`-style search plus `--json` output. Search is local-only and read-only
by default."

Demo approach:

1. `ng doctor --json` showing database access.
2. `ng index` over a real note count.
3. `ng search "old phrase I could not find" --json`.
4. `ng search "invoice" --folder "iCloud/Finance"`.
5. `ng open x-coredata://...` to jump back to Notes.app.
6. A short "what this is not" paragraph: not a note app, not a cloud sync tool,
   not semantic search yet, not an exporter-first product.

## Section 6 - Comparable macOS CLI Tool Case Studies

| Tool | macOS data source | Stars | First distribution channel / visible inflection | Content that drove inflection | Timeline |
| --- | --- | ---: | --- | --- | --- |
| HamburgChimps/apple-notes-liberator | Apple Notes `NoteStore.sqlite` / protobuf-derived export | 1,017; retrieved 2026-06-01 | Hacker News Show HN on 2023-03-26 | 563-point Show HN; GitHub stargazer timestamps show 727 stars by day 7, 774 by day 30, 805 by day 90 | Fast launch spike, then long tail |
| RhetTbull/osxphotos | Apple Photos library database and metadata | 3,596; retrieved 2026-06-01 | Long-running docs, PyPI/Homebrew-style utility surface, README demo GIF | Clear CLI/API docs, supported macOS matrix, active releases through 2026 | Slow durable compounding |
| sballin/alfred-search-notes-app | Apple/iCloud Notes database via Alfred workflow | 586; retrieved 2026-06-01 | Alfred workflow ecosystem and GitHub releases | README demo images, direct download workflow, troubleshooting around Full Disk Access | Durable niche adoption since 2018 |
| RhetTbull/macnotesapp | Apple Notes via AppleScript automation | 265; retrieved 2026-06-01 | Python package/uv plus Homebrew tap | CLI docs, scripting API, Homebrew tap, active releases through 2026 | Moderate technical audience |
| BRO3886/rem | macOS Reminders / EventKit-style local app data | 108; retrieved 2026-06-01 | GitHub plus AI-native CLI positioning | Agent/AI-native README and active releases | Young comparable; ceiling unproven |
| RafalWilinski/mcp-apple-notes | Apple Notes via JXA plus semantic MCP | 393; retrieved 2026-06-01 | MCP wave / GitHub | MCP/RAG README and directory listings; HN launch only 1 point | MCP can drive stars without HN, but maintenance risk |

Pattern analysis:

- The strongest launch evidence is not generic SEO; it is HN rewarding a
  concrete "free your Apple Notes data" technical demo.
- Successful macOS-private-datastore tools explain permissions and private data
  constraints up front. This reduces fear around Full Disk Access.
- Durable tools have install paths beyond source checkout: releases, Homebrew,
  `uvx`, Alfred workflow downloads, or app bundles.
- Demo media helps. `osxphotos` and the Alfred workflow both show visual proof
  immediately.
- The successful angle is a clear wedge, not broad PKM ideology: export Photos,
  liberate Apple Notes, search Notes in Alfred, query Notes from Python.

## Prioritized GTM Sequence

1. Create a canonical GitHub/docs landing page for exact query intent.
   Success metric: page targets five exact H2 queries and includes install,
   Full Disk Access, examples, safety, and comparison sections.
2. Add a terminal demo GIF or short asciinema-style recording to README.
   Success metric: a reader understands the workflow in under 30 seconds.
3. Launch on HN as "Show HN: ng - ripgrep-style search for Apple Notes".
   Success metric: >50 HN points and >100 GitHub stars in 30 days. Stretch
   benchmark from Apple Notes Liberator is ~700 stars in 7-30 days, but that is
   an outlier tied to export/liberation demand.
4. Package for Homebrew or publish a tap before broader Mac-user Reddit posts.
   Success metric: installation no longer requires an existing Rust toolchain;
   Homebrew analytics become measurable.
5. Post to r/macapps / r/MacOS and relevant Apple Notes communities with a
   practical help framing.
   Success metric: comments from real Apple Notes users and at least three
   actionable bug reports or workflow examples.
6. Ship a thin read-only MCP server after the CLI launch.
   Success metric: accepted in at least three MCP directories and one README
   section ranks for `apple notes mcp` exact-title searches. [INFERRED]

## What to Report Back

1. Single most-leveraged first content move:
   `Show HN: ng - ripgrep-style search for Apple Notes`, backed by a canonical
   docs page and terminal demo. Evidence: Apple Notes Liberator's Show HN was
   the strongest comparable public adoption event.
2. Realistic 30-day star-growth benchmark:
   100-250 stars is realistic if HN responds but the project remains CLI/Rust
   only. Apple Notes Liberator's 774 first-30-day stars is a stretch/outlier
   benchmark because its export/liberation pitch was broader than `ng`'s search
   wedge.
3. MCP server verdict:
   Real accelerant after the CLI is known; distraction as the first launch
   story. MCP directories and repos have traction, but Apple Notes MCP Show HN
   launches have not shown strong HN response.
4. Top communities:
   Hacker News first; r/macapps/r/MacOS second after packaging; Obsidian Forum /
   r/ObsidianMD third with an import-adjacent, not competitor, framing.
5. GEO strategy:
   Own exact headings around terminal search, scripts/JSON, no public Notes API,
   `NoteStore.sqlite`, Full Disk Access, and MCP roadmap. Add comparison tables
   and snippets that answer engines can cite directly.

## Uncertainty Flags

- [INFERRED] Search-volume estimates are order-of-magnitude estimates because no
  paid keyword planner export was used.
- [NOT FOUND] Reddit subscriber counts and activity levels could not be verified
  from Reddit JSON because Reddit returned blocked/non-JSON responses to this
  shell. Third-party pages were used only as weak size signals.
- [NOT FOUND] Direct ChatGPT/Perplexity/Claude answer snapshots were not
  captured. GEO analysis is based on indexable public pages and likely citation
  surfaces.
- [NOT FOUND] No primary evidence proved that AI-answer citation alone caused
  adoption for a comparable niche macOS CLI.
- [INFERRED] Homebrew will improve adoption more than `cargo install` alone for
  Mac power users, based on Homebrew formula analytics and the audience's likely
  install habits.
- [INFERRED] MCP directory listings may provide discovery, but their usage data
  is not transparent enough to forecast star lift.

## Source Notes

- Apple Developer Forums, "Apple Notes API":
  https://developer.apple.com/forums/thread/813810
- Hacker News item 35316679, "Show HN: Apple Notes Liberator - Extract
  Notes.app Data and Save It as JSON":
  https://news.ycombinator.com/item?id=35316679
- HN Algolia API result for item 35316679: 563 points, retrieved 2026-06-01.
- HN Algolia API result for item 42433866, "Show HN: Claude and Apple Notes
  integration using MCP": 1 point, retrieved 2026-06-01.
- HN Algolia API result for item 47090580, "Show HN: Apple Notes CLI for
  Agents": 2 points, retrieved 2026-06-01.
- GitHub API, retrieved 2026-06-01:
  `HamburgChimps/apple-notes-liberator` 1,017 stars;
  `RhetTbull/osxphotos` 3,596 stars;
  `sballin/alfred-search-notes-app` 586 stars;
  `RhetTbull/macnotesapp` 265 stars;
  `BRO3886/rem` 108 stars;
  `RafalWilinski/mcp-apple-notes` 393 stars;
  `sirmews/apple-notes-mcp` 128 stars;
  `kzaremski/apple-notes-exporter` 555 stars;
  `modelcontextprotocol/servers` 86,582 stars;
  `modelcontextprotocol/registry` 6,885 stars;
  `modelcontextprotocol/python-sdk` 23,198 stars;
  `modelcontextprotocol/typescript-sdk` 12,580 stars.
- GitHub stargazer timestamp API, retrieved 2026-06-01:
  `HamburgChimps/apple-notes-liberator` had 727 stars by day 7 after the
  2023-03-26 HN launch, 774 by day 30, and 805 by day 90.
- RafalWilinski/mcp-apple-notes README:
  https://github.com/RafalWilinski/mcp-apple-notes
- sirmews/apple-notes-mcp README:
  https://github.com/sirmews/apple-notes-mcp
- kzaremski/apple-notes-exporter README:
  https://github.com/kzaremski/apple-notes-exporter
- RhetTbull/osxphotos README:
  https://github.com/RhetTbull/osxphotos
- RhetTbull/macnotesapp README:
  https://github.com/RhetTbull/macnotesapp
- sballin/alfred-search-notes-app README:
  https://github.com/sballin/alfred-search-notes-app
- Anthropic MCP announcement:
  https://www.anthropic.com/news/model-context-protocol
- Official MCP repositories:
  https://github.com/modelcontextprotocol/servers
  https://github.com/modelcontextprotocol/registry
- MCP directory/search surfaces observed:
  https://mcp.umin.ai/server/apple_notes
  https://playbooks.com/mcp/sirmews/apple-notes-mcp
  https://mcpservers.org/
  https://conare.ai/marketplace/mcp/apple-notes/setup
  https://www.pulsemcp.com/servers/rafal-wilinski-apple-notes
- Obsidian forum Apple Notes import thread:
  https://forum.obsidian.md/t/import-from-apple-notes-to-obsidian/732
- Obsidian forum Apple Notes migration thread:
  https://forum.obsidian.md/t/migrate-from-apple-notes-to-obsidian-retaining-folder-structure-and-images/64000
- Homebrew formula analytics API, retrieved 2026-06-01:
  `nb` 147 installs in 30 days and 3,060 in 365 days;
  `duf` 1,114 installs in 30 days and 15,659 in 365 days.
