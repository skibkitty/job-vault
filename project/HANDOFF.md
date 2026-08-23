# Agent Handoff

## Current session

Agent:
opencode/big-pickle

Date:
2026-08-23

Tasks:
TASK-051 (BLOCKED, WIP on branch task/TASK-051) then TASK-060 (DONE, branch task/TASK-060)

## What has been done

### TASK-051 (branch task/TASK-051, commit 7f8ff3d)

- Implemented `companion/src/diff/mod.rs`: `TextDiffEngine` with paragraph/sentence
  splitting, LCS alignment over normalized segments, Jaccard-similarity pairing
  (threshold 0.5) producing Added/Removed/Modified changes with original text and indices.
- 30 tests written; compiles clean; test EXECUTION blocked by Smart App Control (see below).
- Marked BLOCKED in TASKS.md; docs updated on that branch.

### TASK-060 (branch task/TASK-060)

- Added `extension/src/ui/joblist.ts`: pure data layer — strict payload validation
  (`parseJobSummary`/`parseJobList`), http/https-only URL normalization (drops
  javascript:/data: URLs), case-insensitive filtering across title/company/location,
  deterministic sorting (updatedAt desc, title tiebreak), state resolution
  (`ListState`: loading/ready/empty/locked/unavailable/not-implemented/error).
- Rewrote `extension/src/ui/popup.ts`: fetches jobs via `chrome.runtime.sendMessage`
  to the service worker; renders via createElement/textContent only (no innerHTML);
  handles all ListStates plus malformed-entry count; search box filters live.
- Updated `popup.html`: filter input, job list container, safe-link styling,
  module script tag.
- Extended `background.ts`: `onMessage` handler forwards `jobList` requests to the
  companion over native messaging (`job.list`) and returns errors as structured
  IPC responses.
- Added `test/joblist.test.ts` (33 tests).
- No new permissions, no network access, no new dependencies.

## Blocker discovered this session (affects all Rust tasks)

Windows Smart App Control is On:

- Freshly compiled unsigned binaries are blocked at execution (os error 4551):
  cargo test harnesses and even proc-macro2 build scripts in a fresh CARGO_TARGET_DIR.
- Diagnostics: `(Get-MpComputerStatus).SmartAppControlState` = On;
  HKLM\SYSTEM\CurrentControlSet\Control\CI\Policy VerifiedAndReputablePolicyState = 1.
- Existing cached artifacts still run; extension tooling (node/vitest) unaffected.
- Only the user can turn SAC off (permanent until Windows reinstall). Human decision.
- Previous sessions ran cargo tests earlier today, so SAC likely flipped from
  Evaluation to On between sessions.

## What works

- TASK-060 end-to-end at the code level; verified by automated tests:
  - `npm run typecheck` — pass
  - `npm test` (vitest) — 46 passed (33 new)
  - `npm run build` (tsc emit) — pass
- Popup renders safely without companion (unavailable state).

## What does not work

- Companion still answers `job.list` with NOT_IMPLEMENTED (expected; popup handles it).
- Rust test execution on this machine while SAC is On.

## Tests run

- Extension: typecheck + vitest (46 passed) + tsc build.
- Rust: none executable this session (SAC); TASK-051's 30 tests remain unexecuted.

## Files changed (task/TASK-060)

- extension/src/ui/joblist.ts (new)
- extension/src/ui/popup.ts (rewritten)
- extension/src/ui/popup.html (updated)
- extension/src/background.ts (job.list forwarding)
- extension/test/joblist.test.ts (new)
- project/TASKS.md, project/CURRENT_STATE.md, project/HANDOFF.md

## Important discoveries

- Cargo is not on PATH in this shell: use `& "$env:USERPROFILE\.cargo\bin\cargo.exe"`.
- node/vitest run fine under Smart App Control; rustc/linker output does not.

## Decisions

- Popup talks to the service worker via chrome.runtime.sendMessage so native messaging
  stays in one place (background.ts), consistent with existing architecture.
- Strict response validation in the UI layer; invalid entries skipped and counted,
  never rendered.
- URLs restricted to http/https before any anchor href is set; rel="noreferrer noopener".

## Known risks

- Companion payload shape for job.list is not finalized (NOT_IMPLEMENTED server-side);
  UI validation may need a small follow-up when the real schema lands.
- TASK-051 diff logic remains runtime-unverified until SAC is resolved.

## Next recommended action

1. Merge PR for task/TASK-060 after review.
2. User disables Smart App Control (or permits local build output), then:
   - checkout task/TASK-051, run `& "$env:USERPROFILE\.cargo\bin\cargo.exe" test`,
     fix failures, mark TASK-051 DONE;
   - continue Rust roadmap (TASK-042 similarity matching or TASK-052 bullet diff).

## Instructions for next agent

1. Read AGENTS.md, project/CURRENT_STATE.md, project/HANDOFF.md, project/TASKS.md.
2. Check `(Get-MpComputerStatus).SmartAppControlState`.
3. If On: work extension-side only (node-based tasks run fine).
4. If Off: finish TASK-051 first (tests already written on its branch), then proceed
   to TASK-042/TASK-052 per the registry.

## Blockers

- TASK-051 / all Rust verification: Windows Smart App Control On (human action needed).
