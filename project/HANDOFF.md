# Agent Handoff

## Current session

Agent:
opencode/big-pickle

Date:
2026-08-23

Task:
TASK-031

## What has been done

- Implemented LinkedIn job extraction adapter.
- Created LinkedInExtractor implementing JobExtractor trait.
- Extracts title, company, location, description from LinkedIn job pages.
- Parses raw HTML using CSS-class-based selectors.
- Generates unique IDs for Job and JobSnapshot.
- Sets source to "linkedin" and extractor_name to "linkedin".
- HTML tag stripping for description content.
- 10 comprehensive tests added.
- All 107 tests pass.

## What works

- LinkedInExtractor can_extract detects LinkedIn job URLs
- LinkedInExtractor extract parses job data from HTML
- Fallback to page title or "Untitled Position" when title not found
- HTML tag and entity stripping for clean text
- ExtractionResult includes job, snapshot, source_url, extractor_name
- Pipeline integration works

## What does not work

Nothing broken.

## Tests run

- `cargo test` — 107 passed

## Files changed

- companion/src/extraction/linkedin.rs (new)
- companion/src/extraction/mod.rs (added linkedin module)
- companion/Cargo.toml (added uuid dependency)
- project/TASKS.md
- project/CURRENT_STATE.md
- project/HANDOFF.md

## Important discoveries

- LinkedIn HTML uses class names like top-card-layout__title, t-24 t-bold
- Simple string-based HTML parsing works for extraction
- HTML entity decoding needed for clean output
- UUID crate needed for generating unique Job/Snapshot IDs

## Decisions

- String-based HTML parsing (no external HTML parser crate)
- LinkedIn extractor registered in extraction module
- uuid crate added for unique ID generation
- extractor_name field set to "linkedin" for tracking

## Known risks

- LinkedIn class names may change over time, breaking extraction
- String-based parsing is fragile vs DOM parsing

## Next recommended action

TASK-032 — Indeed adapter.

## Instructions for next agent

1. Read AGENTS.md, project/CURRENT_STATE.md, project/HANDOFF.md, project/TASKS.md.
2. Implement TASK-032 — Indeed adapter.

## Blockers

None.
