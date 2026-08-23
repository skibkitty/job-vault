# Agent Handoff

## Current session

Agent:
opencode/big-pickle

Date:
2026-08-23

Task:
TASK-033

## What has been done

- Implemented generic career-page extraction adapter.
- Created GenericCareerExtractor implementing JobExtractor trait.
- Detects career page URLs (/careers/, /jobs/, /positions/, etc.)
- Extracts title, company, location, description from HTML
- Uses meta tags as fallback (og:title, og:site_name)
- Generates unique IDs for Job and JobSnapshot
- Sets source to "generic" for tracking
- HTML tag stripping for description content
- 15 comprehensive tests added.
- All 133 tests pass.

## What works

- GenericCareerExtractor can_extract detects career page URLs
- GenericCareerExtractor extract parses job data from HTML
- Multiple selector strategies for each field
- Meta tag fallback for title and company
- Fallback to page title or "Untitled Position" when title not found
- HTML tag and entity stripping for clean text
- ExtractionResult includes job, snapshot, source_url, extractor_name
- Pipeline integration works

## What does not work

Nothing broken.

## Tests run

- `cargo test` — 133 passed

## Files changed

- companion/src/extraction/generic.rs (new)
- companion/src/extraction/mod.rs (added generic module)
- project/TASKS.md
- project/CURRENT_STATE.md
- project/HANDOFF.md

## Important discoveries

- Career pages use varied class names across companies
- Meta tags provide reliable fallback for title/company
- URL pattern matching (/careers/, /jobs/) works well for detection
- Generic extraction is less precise than site-specific but functional

## Decisions

- String-based HTML parsing (no external HTML parser crate)
- Multiple selector strategies for robustness
- Meta tag fallback for better coverage
- extractor_name field set to "generic" for tracking

## Known risks

- Generic extraction may miss fields on some career pages
- Class names vary widely across companies

## Next recommended action

TASK-034 — Manual save fallback.

## Instructions for next agent

1. Read AGENTS.md, project/CURRENT_STATE.md, project/HANDOFF.md, project/TASKS.md.
2. Implement TASK-034 — Manual save fallback.

## Blockers

None.
