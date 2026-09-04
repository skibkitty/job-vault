# Agent Handoff

## Current session

Agent:
opencode/big-pickle

Date:
2026-08-31

Tasks:
TASK-051 (DONE, PR #19 open), TASK-052 (DONE, PR #20 open), TASK-053 (DONE, PR #21 open),
TASK-054 (DONE, branch task/TASK-054). TASK-060/061/062 (merged to main).

## What has been done

### TASK-051 (PR #19 open — paragraph/sentence diff engine)

- Implemented paragraph/sentence diff engine in `companion/src/diff/mod.rs`
  (`TextDiffEngine`): `split_paragraphs`, `split_sentences`, `similarity` (Dice
  coefficient), `diff_paragraphs`, `diff_sentences`, `diff_texts`, `diff_segments`
  (LCS alignment over TASK-050-normalized segment text; best-match pairing labels
  Modified vs Added/Removed), configurable similarity threshold.
- Results preserve original un-normalized text; `old_index`/`new_index` locate changes.
- Deterministic; no randomness, no time dependence; no new dependencies, no network,
  no LLM.
- Registered module in `companion/src/main.rs`.
- Fixed all 6 initially failing tests (sentence-level routing, capitalization-aware
  sentence split, Dice similarity with punctuation stripping, best-match pairing).

### TASK-052 (branch task/TASK-052 — bullet diff)

- Extended `companion/src/diff/mod.rs` with `split_bullets` and `diff_bullets`.
- `split_bullets` segments bulleted lists: dash `-`, star `*`, bullet `•`, interpunct
  `·`, and numbered markers (`1.`, `1)`, `1]`). Continuation (wrapped) lines append to
  the owning bullet; blank lines separate blocks; non-bullet prose is ignored.
- `diff_bullets` reuses `diff_segments` (LCS + similarity pairing) so added/removed/
  modified bullet classification, normalization tolerance, and determinism all carry
  over from TASK-051.
- Wrote 16 new tests (splitting variants, add/remove/modify, continuation lines,
  normalization tolerance, empty input, determinism).

### TASK-053 (branch task/TASK-053 — moved/reordered detection)

- Added `ChangeType::Moved` to the diff engine.
- New deterministic move-detection pipeline (`reordered_changes`) with distinct passes:
  1. `anchor_identical`: LCS-aligned in-order identical items are counted as unchanged.
  2. `match_moved`: identical normalized content present in both (but not order-aligned)
     is reported as `Moved` with `old_index`/`new_index`.
  3. `pair_modified`: remaining items paired by Dice similarity (modified threshold).
  4. Leftover items are `Added`/`Removed`.
- Public entry points: `diff_segments_reordered(old, new)` and
  `diff_bullets_reordered(old_text, new_text)`.
- Distinguishes genuine reordering/moves from added/removed items; normalization
  (case/punctuation) is ignored when matching moved content.
- Wrote 10 new tests: rotation, move-to-end, identical, parallel swap, bullet
  rotation, addition+move, pure addition (no false move), normalization tolerance,
  modification+removal, determinism.

### TASK-054 (branch task/TASK-054 — requirement/responsibility changes)

- Added section-level diffing for a job's requirements and responsibilities.
- `segment_section`: segments a section string as bullets, falling back to
  paragraphs for prose blocks.
- `diff_requirements(old, new)` and `diff_responsibilities(old, new)` accept
  `Option<&str>` (missing = empty) and reuse the move/reorder-aware
  `diff_segments_reordered` machinery, so Added/Removed/Modified/Moved carry over.
- `diff_requirement_sections(...)` returns a `SectionChanges` struct with the full
  diff for each section plus `added/removed_requirements` and
  `added/removed_responsibilities` counts.
- Wrote 11 new tests: segmentation (bullets/paragraphs/empty), added/removed/modified
  requirements, responsibilities add+remove, missing->added, combined counts, identical,
  normalization tolerance.

## Test execution workaround (important)

Windows Smart App Control blocks executing freshly compiled unsigned binaries on the
host (`cargo test` fails, os error 4551). Worked around by executing Rust tests in WSL
Ubuntu through the MSVC-targeted Cargo project:

```bash
wsl -d Ubuntu -- bash -lc "rsync -a /mnt/c/.../companion/ /root/job-vault/companion-NNN/ && . ~/.cargo/env && cargo test"
```

- All 228 companion tests pass (202 prior phases + 26 diff-engine tests).
- The default WSL distro is `docker-desktop` (no Rust toolchain); `Ubuntu` distro has
  the Rust toolchain configured.

## What works

- TASK-051 paragraph/sentence diff, TASK-052 bullet diff, TASK-053 moved/reordered
  detection, and TASK-054 requirement/responsibility changes: verified by 239 passing
  Rust tests via WSL.
- Extension UI merged through TASK-062 (job list, job detail, snapshot history/detail),
  verified by 86 extension tests.

## What does not work

- Executing newly built Rust binaries on the Windows host (Smart App Control).
- Companion still returns NOT_IMPLEMENTED for job.list and job.get (IPC handlers not
  yet wired to the diff/storage modules).

## Tests run

- Rust: 239 passed, 0 failed (`cargo test` via WSL Ubuntu).
- Extension: typecheck + vitest (86 passed) + tsc build (as of TASK-062).

## Files changed (task/TASK-054)

- companion/src/diff/mod.rs (SectionChanges, segment_section, diff_requirements,
  diff_responsibilities, diff_requirement_sections, section tests)
- project/TASKS.md (TASK-054 criteria, marked DONE)
- project/CURRENT_STATE.md
- project/HANDOFF.md

## Important discoveries

- Cargo is not on PATH in the default shell: use
  `& "$env:USERPROFILE\.cargo\bin\cargo.exe"`, or run tests via WSL.
- WSL Ubuntu is a reliable path to execute Rust tests despite Smart App Control.
- Branching TASK-052 from task/TASK-051 (stacked) required merging origin/main into it
  to bring the branch current with main's UI work; expect a similar rebase/merge when
  TASK-051's PR merges.

## Decisions

- Hand-rolled LCS alignment and Dice similarity instead of adding a diff crate
  (dependency minimization policy).
- TASK-052 reuses the TASK-051 segment diff machinery with a bullet splitter rather
  than duplicating diff logic.
- Bullet continuation lines are joined onto the owning bullet with a single space to
  keep wrapped list items as one segment.
- TASK-053 move detection is a multi-pass identity-then-similarity pipeline: identical
  normalized content anywhere in the other sequence is a "Moved" (not added/removed);
  only genuinely unmatched content becomes Added/Removed/Modified. LCS anchors keep
  in-order identical items as unchanged, so a full swap is reported as
  unchanged+move(s) rather than removed+added.
- TASK-054 treats requirements and responsibilities as independent "sections", each
  diffed with the move/reorder-aware machinery, and reports added/removed counts per
  section so the UI can surface "3 new requirements" at a glance.

## Known risks

- For an even swap, LCS anchors half the items as "unchanged" and reports the rest as
  "moved" — this is an inherent ambiguity of reorder detection and is deterministic,
  but downstream UI should not assume one "correct" canonical alignment.
- Diff quality for real-world bullet prose is unverified against sanitized fixtures yet.
- TASK-051/052/053/054 branches are stacked; merging order matters. Merge TASK-051
  (#19), TASK-052 (#20), then TASK-053 (#21), then TASK-054.

## Next recommended action

1. Merge PRs for TASK-051 (#19), TASK-052 (#20), TASK-053 (#21), and TASK-054 in order.
2. Continue with TASK-055 — Change ranking (depends on TASK-054). Define acceptance
   criteria and implement in `companion/src/diff/mod.rs`.

## Instructions for next agent

1. Read AGENTS.md, project/CURRENT_STATE.md, project/HANDOFF.md, project/TASKS.md.
2. Run Rust tests via WSL Ubuntu (see workaround above); do not attempt to run
   freshly compiled Rust binaries directly on the Windows host.
3. For the next Rust diff task (TASK-055), branch off `main` after TASK-051/#19,
   TASK-052/#20, TASK-053/#21, and TASK-054 merge, or merge origin/main into a stacked
   branch and resolve doc conflicts.

## Blockers

- Windows Smart App Control blocks host-side Rust test execution (mitigated via WSL).
- Phase 6 UI tasks after TASK-062 (TASK-063+) remain blocked on TASK-055 (change ranking).
