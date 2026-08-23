# Agent Handoff

## Current session

Agent:
opencode/big-pickle

Date:
2026-08-23

Task:
TASK-032

## What has been done

- Implemented Indeed job extraction adapter.
- Created IndeedExtractor implementing JobExtractor trait.
- Extracts title, company, location, description from Indeed job pages.
- Parses raw HTML using CSS-class-based selectors.
- Generates unique IDs for Job and JobSnapshot.
- Sets source to "indeed" and extractor_name to "indeed".
- HTML tag stripping for description content.
- 11 comprehensive tests added.
- All 118 tests pass.

## What works

- IndeedExtractor can_extract detects Indeed job URLs
- IndeedExtractor extract parses job data from HTML
- Fallback to page title or "Untitled Position" when title not found
- HTML tag and entity stripping for clean text
- ExtractionResult includes job, snapshot, source_url, extractor_name
- Pipeline integration works

## What does not work

Nothing broken.

## Tests run

- `cargo test` — 118 passed

## Files changed

- companion/src/extraction/indeed.rs (new)
- companion/src/extraction/mod.rs (added indeed module)
- project/TASKS.md
- project/CURRENT_STATE.md
- project/HANDOFF.md

## Important discoveries

- Indeed HTML uses class names like jobsearch-JobInfoHeader-title, jobsearch-JobInfoHeader-location
- Simple string-based HTML parsing works for extraction
- HTML entity decoding needed for clean output
- UUID crate already added in TASK-031

## Decisions

- String-based HTML parsing (no external HTML parser crate)
- Indeed extractor registered in extraction module
- extractor_name field set to "indeed" for tracking

## Known risks

- Indeed class names may change over time, breaking extraction
- String-based parsing is fragile vs DOM parsing

## Next recommended action

TASK-033 — Generic career-page adapter.

## Instructions for next agent

1. Read AGENTS.md, project/CURRENT_STATE.md, project/HANDOFF.md, project/TASKS.md.
2. Implement TASK-033 — Generic career-page adapter.

## Blockers

None.
