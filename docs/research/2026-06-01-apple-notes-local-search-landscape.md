# Apple Notes Local Search Landscape

Retrieval date for live repository, package, and Homebrew data: 2026-06-01.

## Executive Summary

- Apple Notes has a proven unmet tooling gap, but the strongest public demand signal is still export/liberation and reliable local access, not polished semantic search. Multiple projects over 500 GitHub stars exist for export/search-adjacent workflows, while Apple Notes semantic MCP projects are younger and fragmented.
- `ng` already occupies a useful gap: direct `NoteStore.sqlite` reads, decoded full-body text, JSONL cache, folder metadata, stable IDs, and agent-friendly CLI output. Most existing Apple Notes tools are export apps, Alfred workflows, JXA/AppleScript bridges, or MCP demos rather than `rg`-style local search backends.
- Local RAG over personal notes is real in Obsidian and general local-AI tooling, but Apple Notes-specific demand is early. Semantic search should be a roadmap item after regex and an inverted index, not before the text-search spine is fast, trustworthy, and distributable.
- MCP is a meaningful opportunity because MCP adoption is broad and Apple Notes MCP servers already exist, but the opportunity is not greenfield. A thin read/search MCP over `ng` is higher ROI than a first-party embedding stack because it exposes existing strengths to Claude/Cursor/Windsurf without competing head-on with larger RAG apps.
- Distribution should not rely on `cargo install` alone. Homebrew materially improves discovery for CLI tools, and macOS private-datastore tools that succeed publicly tend to provide packaged binaries/apps or Homebrew-style installation, not source-only setup.

## Section 1 - Apple Notes Tooling Landscape

| Tool/project | Access method | Full-body search | Open source | Maintenance status | Adoption signal |
| --- | --- | --- | --- | --- | --- |
| [kzaremski/apple-notes-exporter](https://github.com/kzaremski/apple-notes-exporter) | Direct local Notes database; Swift app plus CLI/MCP in v2 docs | Export/filter oriented, not `rg`-style search | Yes, GPL-3.0 | Active; pushed 2026-05-12 | 555 stars, 37 forks; retrieved 2026-06-01 |
| [threeplanetssoftware/apple_cloud_notes_parser](https://github.com/threeplanetssoftware/apple_cloud_notes_parser) | Direct database/parser for Apple Notes records and protobuf formats | Parser/export foundation, not user search CLI | Yes, MIT | Active-ish; pushed 2026-02-14 | 528 stars, 39 forks; retrieved 2026-06-01 |
| [HamburgChimps/apple-notes-liberator](https://github.com/HamburgChimps/apple-notes-liberator) | Apple Notes data liberation/export | Export oriented | Yes, MIT | Not recently pushed; last push 2023-09-24 | 1,017 stars, 26 forks; retrieved 2026-06-01 |
| [sballin/alfred-search-notes-app](https://github.com/sballin/alfred-search-notes-app) | Alfred workflow for opening/searching iCloud/Apple Notes | Search/open workflow, not standalone body-index CLI | Yes, MIT | Maintained; pushed 2025-01-15 | 586 stars, 26 forks; retrieved 2026-06-01 |
| [dunhamsteve/notesutils](https://github.com/dunhamsteve/notesutils) | Python utilities for extracting Notes.app data | Extraction utilities, lightly maintained | Yes, Unlicense | README says lightly maintained; pushed 2022-12-06 | 250 stars, 15 forks; retrieved 2026-06-01 |
| [KrauseFx/notes-exporter](https://github.com/KrauseFx/notes-exporter) | Deprecated direct SQLite exporter for old Apple Notes format | Export only; broken by Apple format changes | Yes, MIT | Stale; pushed 2018-08-14 | 203 stars, 10 forks; retrieved 2026-06-01 |
| [yirogue/apple_notes_export](https://github.com/yirogue/apple_notes_export) | Direct database export script | Export only | Yes | Stale; pushed 2023-08-01 | 2 stars; retrieved 2026-06-01 |

Key developer pain points:

- Bulk export remains a recurring pain point. Zapier's 2024 guide says Apple supports single-note PDF export but no built-in bulk export, and points users to third-party Exporter-style tools. Source: [Zapier](https://zapier.com/blog/export-apple-notes/).
- The data format is opaque. Reddit support threads show users finding `NoteStore.sqlite` and SQLite tables but not the actual note bodies, because body text lives in compressed protobuf-ish blobs. Source: [r/mac recovery thread](https://www.reddit.com/r/mac/comments/1lg6kuv/need_help_recovering_lost_apple_notes_after/), [r/mac DB Browser thread](https://www.reddit.com/r/mac/comments/1j4xyg9).
- Apple Notes Exporter's public README and related community discussion show sustained demand for local export, folder preservation, and database-backed speed; it also shows the market's copycat risk once a local Notes parser works. Sources: [kzaremski/apple-notes-exporter](https://github.com/kzaremski/apple-notes-exporter), [r/macapps discussion](https://www.reddit.com/r/macapps/comments/1pzjnsh/apple_notes_exporter_pro_export_keep_backup_of/).
- AppleScript/JXA approaches are accessible but fragile and slower for large libraries; direct database approaches are faster but depend on private schema interpretation and Full Disk Access.

Gap analysis:

- Existing tools prove demand for local access, export, and opening notes, but few are composable search tools. `ng` should not become another exporter.
- The durable gap is "fast local note body search with stable IDs and JSON output", especially for agents and shell workflows.
- Direct ZICNOTEDATA body extraction is valuable because many scripts either defer to AppleScript-rendered HTML or only partially decode body text.
- Attachment OCR, checklist semantics, encrypted notes, shared-note state, and schema-version drift remain real gaps across the ecosystem.

## Section 2 - Local-First Note Search Ecosystem

Obsidian is the clearest adjacent market because it uses local Markdown files and has a mature plugin ecosystem. The pattern is: keyword search and fuzzy search are table stakes; semantic search is increasingly expected, but users still complain about reliability, relevance, licensing, indexing time, and local/cloud boundaries.

| Tool | Note format | Search type | Local/cloud | Active maintenance | Adoption signal |
| --- | --- | --- | --- | --- | --- |
| [brianpetro/obsidian-smart-connections](https://github.com/brianpetro/obsidian-smart-connections) | Obsidian Markdown vault | Semantic related notes, chat, embeddings | Local models or API providers | Active; pushed 2026-05-30 | 5,076 stars, 317 forks; retrieved 2026-06-01 |
| [scambier/obsidian-omnisearch](https://github.com/scambier/obsidian-omnisearch) | Obsidian Markdown plus PDFs/OCR | Full-text search, OCR/PDF indexing | Local plugin | Active; pushed 2026-05-25 | 2,019 stars, 103 forks; retrieved 2026-06-01 |
| [GoBeromsu/Open-smart-connections](https://github.com/GoBeromsu/Open-smart-connections) | Obsidian Markdown vault | Semantic related notes/search | Local embeddings by default; optional APIs | Active by search snippet; API repo moved | [INFERRED] Fork/rebuild signal indicates licensing/stability demand |
| [achekulaev/obsidian-qmd](https://github.com/achekulaev/obsidian-qmd) | Obsidian Markdown vault | Local semantic search plus BM25 fallback | Local | Young; pushed 2026-02-01 | 62 stars, 8 forks; retrieved 2026-06-01 |
| [bbawj/obsidian-semantic-search](https://github.com/bbawj/obsidian-semantic-search) | Obsidian Markdown vault | Semantic search | Local/API depending setup | Some maintenance; pushed 2025-10-13 | 150 stars, 13 forks; retrieved 2026-06-01 |
| [mmargenot/tezcat](https://github.com/mmargenot/tezcat) | Obsidian Markdown vault | Local Ollama semantic indexing/search | Local | Active; pushed 2026-03-30 | 21 stars; retrieved 2026-06-01 |
| [xwmx/nb](https://github.com/xwmx/nb) | Plain text/Markdown notes | CLI text search, tags, links, bookmarks | Local; optional git sync | Active; pushed 2026-04-28 | 8,193 stars, 251 forks; retrieved 2026-06-01 |

Unmet needs reported by users:

- Better relevance than literal search without losing privacy. Obsidian users repeatedly compare stock search, Omnisearch, and semantic plugins. Sources: [r/ObsidianMD Omnisearch discussion](https://www.reddit.com/r/ObsidianMD/comments/1laddm2/differences_between_the_omnisearch_plugin_and/), [r/ObsidianMD semantic search threads](https://www.reddit.com/r/ObsidianMD/comments/1nbk5a2).
- Fully local RAG/chat that does not require external APIs. This is visible in Obsidian plugin announcements and LocalLLM-style posts. Sources: [Obsidian Sonar announcement](https://forum.obsidian.md/t/ann-sonar-offline-semantic-search-and-agentic-ai-chat-for-obsidian-powered-by-llama-cpp/110765), [r/ObsidianMD local LLM hub](https://www.reddit.com/r/ObsidianMD/comments/1ruboff/i_built_a_fully_local_ai_plugin_for_obsidian_rag/).
- Search tools that expose results outside the note app. Omnisearch includes an optional local HTTP server; `nb` exposes a CLI-first workflow. That is directly relevant to `ng` as an agent backend.

## Section 3 - Local RAG and Semantic Search Momentum

Evidence for demand:

- General local RAG tools survived the first hype wave. [Khoj](https://github.com/khoj-ai/khoj) has 34,797 stars, [PrivateGPT](https://github.com/zylon-ai/private-gpt) has 57,218, [AnythingLLM](https://github.com/Mintplex-Labs/anything-llm) has 60,905, and [Quivr](https://github.com/QuivrHQ/quivr) has 39,173; retrieved 2026-06-01. These are broader than personal notes, but they prove local/private document QA has durable attention.
- Ollama remains a major local model substrate: [ollama/ollama](https://github.com/ollama/ollama) had 172,852 stars and active pushes on 2026-06-01. Its official docs include embedding generation for semantic search/RAG. Source: [Ollama embeddings docs](https://docs.ollama.com/capabilities/embeddings).
- Apple Notes-specific semantic MCP projects exist, including [RafalWilinski/mcp-apple-notes](https://github.com/RafalWilinski/mcp-apple-notes) and [Dan8Oren/mcp-apple-notes](https://github.com/Dan8Oren/mcp-apple-notes), but their maintenance/activity profiles are uneven. That supports "real but early", not "obviously dominant".

Evidence against semantic search as the immediate next `ng` feature:

- Apple Notes users publicly ask for export, recovery, bulk access, and working search/open workflows more often than vector search.
- The strongest semantic note ecosystem is Obsidian, where clean Markdown vaults remove the hardest part of Apple Notes: reliable extraction from a private Core Data/protobuf schema.
- For 5k-50k notes, embeddings are practical, but result quality depends heavily on chunking, body extraction fidelity, folder metadata, and incremental invalidation. Those are downstream of a correct text/index layer.

| Embedding model | Apple Silicon support | Practical notes-scale latency | License |
| --- | --- | --- | --- |
| `all-MiniLM-L6-v2` | Common in Transformers.js/SentenceTransformers; used by Apple Notes MCP projects | Practical for 5k-50k notes; small model, lower quality ceiling | Apache-2.0 via SentenceTransformers model card ecosystem [INFERRED from common distribution] |
| `nomic-embed-text-v1.5` | Available through Hugging Face and Ollama-style local stacks | Practical; long-context model reduces aggressive chunking pressure | Apache-2.0. Sources: [Hugging Face](https://huggingface.co/nomic-ai/nomic-embed-text-v1.5), [technical report](https://arxiv.org/abs/2402.01613) |
| `mxbai-embed-large` | Available in Ollama; larger model | Practical but heavier; better suited after a fast incremental index exists | License should be verified from the model card before bundling. Sources: [Ollama model page](https://ollama.com/library/mxbai-embed-large), [mixedbread model card](https://huggingface.co/mixedbread-ai/mxbai-embed-large-v1) |
| BGE small/base variants | Widely available through SentenceTransformers/Ollama-compatible stacks | Practical; small variants fit note-scale corpora well | BAAI model licenses vary by variant; verify before bundling. Source: [BGE docs](https://bge-model.com/bge/bge_v1_v1.5.html) |
| Apple Natural Language `NLEmbedding` / `NLContextualEmbedding` | Native Apple frameworks | Useful for native experiments, but less portable for a Rust CLI and less proven for retrieval ranking than standard embedding models | Apple platform APIs, no model redistribution by `ng`. Sources: [NLEmbedding](https://developer.apple.com/documentation/naturallanguage/nlembedding), [NLContextualEmbedding](https://developer.apple.com/documentation/naturallanguage/nlcontextualembedding) |

Apple platform APIs relevant to on-device semantic search:

- Natural Language `NLEmbedding` supports word and sentence embeddings and nearest-neighbor lookup. It can help native macOS apps, but bridging this into a Rust CLI would add platform complexity.
- `NLContextualEmbedding` computes contextual token vectors and can download assets. Apple docs explicitly point semantic similarity work toward `NLEmbedding`, while contextual embeddings require pooling for whole-text representations.
- Foundation Models provides on-device language generation/tool-calling access for Apple Intelligence tasks, but it is not a turnkey vector-search API over Notes. Source: [Apple Foundation Models docs](https://developer.apple.com/documentation/FoundationModels).

## Section 4 - MCP Server Landscape for Notes/Local Data

| Server/project | Capabilities | Limitations | Adoption signal |
| --- | --- | --- | --- |
| [RafalWilinski/mcp-apple-notes](https://github.com/RafalWilinski/mcp-apple-notes) | RAG over Apple Notes, semantic search with `all-MiniLM-L6-v2`, LanceDB, JXA integration | Last pushed 2024-12-17 despite high stars; no license returned by GitHub API | 393 stars, 51 forks; retrieved 2026-06-01 |
| [sirmews/apple-notes-mcp](https://github.com/sirmews/apple-notes-mcp) | Read local Notes database, get all notes, read note, search notes | Archived; README lists missing encrypted notes, pinned note filtering, sync status, attachments, checklist status, create/edit | 128 stars, 20 forks; archived; retrieved 2026-06-01 |
| [Dan8Oren/mcp-apple-notes](https://github.com/Dan8Oren/mcp-apple-notes) | Semantic search, folder browsing, RAG, local/no API keys | Very young/small project | 8 stars, pushed 2026-05-27; retrieved 2026-06-01 |
| [Siddhant-K-code/mcp-apple-notes](https://github.com/Siddhant-K-code/mcp-apple-notes) | Create/search/retrieve notes via MCP listing pages | Capability details need code audit before trust | 21 stars, pushed 2026-05-08; retrieved 2026-06-01 |
| [0xatrilla/Apple-MCP](https://github.com/0xatrilla/Apple-MCP) | Broad local macOS MCP server for Apple apps including Notes | Very new, low adoption | 2 stars, created 2026-05-28; retrieved 2026-06-01 |
| [rusudinu/orbit-mcp](https://github.com/rusudinu/orbit-mcp) | Announced Notes/Reminders MCP app | Public repo has minimal metadata | 0 stars, created 2026-05-25; retrieved 2026-06-01 |

General MCP adoption signals:

- Anthropic introduced MCP as an open standard for connecting AI tools to data sources in November 2024. Source: [Anthropic announcement](https://www.anthropic.com/news/model-context-protocol).
- The official [modelcontextprotocol/servers](https://github.com/modelcontextprotocol/servers) repo had 86,582 stars and 10,892 forks on 2026-06-01.
- The official [modelcontextprotocol/registry](https://github.com/modelcontextprotocol/registry) had 6,885 stars on 2026-06-01.
- Official SDKs have large adoption signals: [python-sdk](https://github.com/modelcontextprotocol/python-sdk) had 23,198 stars and [typescript-sdk](https://github.com/modelcontextprotocol/typescript-sdk) had 12,580 on 2026-06-01.

Assessment:

- Apple Notes MCP is meaningful but not well-served. There are multiple servers, but no obvious durable, well-maintained, extraction-correct standard.
- `ng` should expose MCP after stabilizing the CLI/search contract. The MCP should be a thin wrapper over `ng search`, `ng open`, folder filters, and stable IDs, with explicit read-only defaults.
- A semantic MCP mode can be added later as an index backend. Shipping MCP first without local embeddings still helps agents retrieve exact notes and avoids duplicating fragile parser code in TypeScript/Python servers.

## Section 5 - Distribution and Adoption Patterns

Cargo-only distribution:

- `cargo install` is acceptable for Rust developers but a poor primary channel for macOS users who do not already have Rust. The Rust/Cargo install path requires rustup/toolchain setup. Source: [Cargo Book installation](https://doc.rust-lang.org/stable/cargo/getting-started/installation.html).
- Crates.io can show real CLI demand for famous cross-platform tools: `ripgrep` had 1,427,740 total crate downloads and 73,415 recent downloads; `fd-find` had 713,572 total and 34,201 recent; `bottom` had 244,815 total and 8,256 recent. Retrieved from crates.io API on 2026-06-01.
- Those projects are not macOS-private-datastore tools; they are broad developer utilities with large brands. [INFERRED] A macOS-only Notes CLI should expect a lower ceiling from `cargo install` alone.

Homebrew:

- Homebrew analytics show substantial CLI discovery/install volume. On 2026-06-01, Homebrew formula data reported: `ripgrep` 71,732 installs in 30 days, `fd` 17,702 installs in 30 days, and `bottom` 1,128 installs in 30 days.
- Homebrew matters because it avoids the "install Rust first" problem and puts `ng` where macOS developers already look for CLI tools.
- A custom tap is enough at first; core formula can wait until versioned releases, tests, and licensing metadata are clean.

Comparable case studies:

- [RhetTbull/osxphotos](https://github.com/RhetTbull/osxphotos) is the strongest macOS-private-datastore analogue even though it is Python, not Rust: it reads Apple Photos libraries, has CLI/library surfaces, and had 3,596 stars on 2026-06-01. This suggests private Apple data-store CLIs can exceed hobby scale when they solve export/search/metadata tasks well.
- [BRO3886/rem](https://github.com/BRO3886/rem) is a young macOS Reminders CLI written in Go, with direct EventKit access, JSON/CSV export, and agent-skill positioning. It had 108 stars on 2026-06-01. This is closer to `ng`'s "native Apple app from terminal" positioning, but not yet a proven adoption ceiling.
- [HCYT/cueward](https://github.com/HCYT/cueward) is a Rust CLI for scattered macOS knowledge fragments including Apple Notes/Reminders/Calendar, but had only 2 stars on 2026-06-01. It is evidence of emerging agent-local-data interest, not adoption.
- [NOT FOUND] I did not find a Rust CLI targeting a macOS-specific private data store with more than 500 GitHub stars. The adjacent >500-star successes are Swift/Java/Go/Python or cross-platform Rust CLIs.

Show HN / launch implications:

- [INFERRED] A Show HN can work if the pitch is concrete: "ripgrep for Apple Notes" plus a demo over thousands of local notes, JSON output for agents, no network calls, and Full Disk Access caveats.
- Do not lead with "semantic chat with notes" until semantic search exists and beats exact/BM25 search on real queries. The current market already has semantic MCP demos; `ng`'s differentiator is correctness, speed, and composability.

## Strategic Implications

- Priority 1: regex and search ergonomics. `rg`-style flags, literal/regex modes, folder/account filters, stable JSON output, snippets, exit codes, and cache invalidation are the most defensible next improvements.
- Priority 2: Tantivy/BM25 inverted index. The ecosystem shows users value relevance and speed, but exact local search remains the base layer. Tantivy gives fast ranked lexical search without entering model-serving complexity.
- Priority 3: thin read-only MCP wrapper. MCP demand is broad, Apple Notes MCP is not yet standardized, and `ng` can expose a high-trust backend with fewer moving parts than a TypeScript parser.
- Priority 4: semantic search as an optional backend. Use the Tantivy/index abstraction to add embeddings later, likely via optional local providers such as Ollama or a small bundled model. Do not make embeddings required for the default CLI.
- Priority 5: distribution before public launch. Add versioned releases, a Homebrew tap, install docs for Full Disk Access, and a read-only safety story before Show HN.
- Avoid building an export-first GUI/app. That market has stronger incumbents and different user expectations. `ng` should remain the composable search/access layer.

## Uncertainty Flags

- [NOT FOUND] No official Apple Notes public API for full-body local search or export was found. Existing tools rely on AppleScript/JXA, Shortcuts, direct SQLite/private schema reads, or app automation.
- [NOT FOUND] No Rust macOS-private-datastore CLI over 500 GitHub stars was found during this pass.
- [INFERRED] Star counts for Apple Notes MCP projects indicate interest, but not necessarily active use; installation/download telemetry is sparse.
- [INFERRED] Homebrew will likely improve `ng` adoption more than crates.io alone, based on Homebrew analytics for comparable CLI tools and the macOS audience, but exact lift cannot be measured before release.
- [INFERRED] For 5k-50k notes, local embedding latency is practical on Apple Silicon, but actual `ng` performance depends on chunking strategy, vector store choice, incremental indexing, and whether body extraction preserves enough semantic context.
