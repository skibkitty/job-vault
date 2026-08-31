# Agent Handoff

## Current session

Agent:
opencode/big-pickle

Date:
2026-08-31

Tasks:
TASK-051 (DONE, PR #19 open), TASK-052 (DONE, branch task/TASK-052).
TASK-060/061/062 (merged to main).

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

## Test execution workaround (important)

Windows Smart App Control blocks executing freshly compiled unsigned binaries on the
host (`cargo test` fails, os error 4551). Worked around by executing Rust tests in WSL
Ubuntu through the MSVC-targeted Cargo project:

```bash
wsl -d Ubuntu -- bash -lc "rsync -a /mnt/c/.../companion/ /root/job-vault/companion-NNN/ && . ~/.cargo/env && cargo test"
```

- All 218 companion tests pass (202 prior + 16 new bullet-diff tests).
- The default WSL distro is `docker-desktop` (no Rust toolchain); `Ubuntu` distro has
  the Rust toolchain configured.

## What works

- TASK-051 paragraph/sentence diff and TASK-052 bullet diff: verified by 218 passing
  Rust tests via WSL.
- Extension UI merged through TASK-062 (job list, job detail, snapshot history/detail),
  verified by 86 extension tests.

## What does not work

- Executing newly built Rust binaries on the Windows host (Smart App Control).
- Companion still returns NOT_IMPLEMENTED for job.list and job.get (IPC handlers not
  yet wired to the diff/storage modules).

## Tests run

- Rust: 218 passed, 0 failed (`cargo test` via WSL Ubuntu).
- Extension: typecheck + vitest (86 passed) + tsc build (as of TASK-062).

## Files changed (task/TASK-052)

- companion/src/diff/mod.rs (split_bullets, diff_bullets, bullet tests)
- project/TASKS.md (TASK-052 criteria; TASK-051 marked DONE)
- project/CURRENT_STATE.md
- project/HANDOFF.md
- Merge of origin/main brought in the extension UI work (TASK-060/061/062) so this
  branch is current with main.

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

## Known risks

- The TASK-052 branch was stacked on TASK-051 then merged with origin/main; doc merge
  conflicts were resolved manually. When TASK-051's PR (#19) merges, revisit whether a
  final rebase is needed before TASK-052 merges.
- Diff quality for real-world bullet prose is unverified against sanitized fixtures yet.

## Next recommended action

1. Merge PR for TASK-051 (#19) after review.
2. Continue with TASK-053 — Moved/reordered detection (depends on TASK-052).
   Define its acceptance criteria and implement in `companion/src/diff/mod.rs`.
3. Then TASK-054 (requirement/responsibility changes) and TASK-055 (change ranking).

## Instructions for next agent

1. Read AGENTS.md, project/CURRENT_STATE.md, project/HANDOFF.md, project/TASKS.md.
2. Run Rust tests via WSL Ubuntu (see workaround above); do not attempt to run
   freshly compiled Rust binaries directly on the Windows host.
3. For the next Rust diff task, branch off `main` after TASK-051/#19 and TASK-052 merge,
   or merge origin/main into a stacked branch and resolve doc conflicts.

## Blockers

- Windows Smart App Control blocks host-side Rust test execution (mitigated via WSL).
- Phase 6 UI tasks after TASK-062 (TASK-063+) remain blocked on TASK-055 (change ranking).
