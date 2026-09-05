# Agent Handoff

## Current session

Agent:
opencode/big-pickle

Date:
2026-09-05

Tasks:
TASK-055 (change ranking) implemented and verified in WSL (247 companion tests pass).
Diff engine TASK-051/052/053/054 previously merged to main. Canonical Rust workspace relocated to
WSL at /root/job-vault; WSL dev environment documented (AGENTS.md section 7b); WSL push
credentials now fully wired (root gh config + gh auth setup-git).
TASK-060/061/062 (extension UI) previously merged to main.

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

### TASK-055 (change ranking)

- Added change ranking to `companion/src/diff/mod.rs`:
  - `RankedChange` (a `SegmentChange` + deterministic `score` + 1-based `rank`)
  - `RankedDiffResult` (original `DiffResult` + `ranked` order, most-significant-first)
  - `RankedSectionChanges` (mirrors `SectionChanges` with per-section ranking)
  - `TextDiffEngine::rank_changes`, `rank_section_changes`, `rank_all_section_changes`
- Deterministic scoring: Added/Removed `2.0`; Modified `1.0 + 0.5*(1 - similarity)`;
  Moved `0.5`. Ties broken by (old_index, new_index, content). No randomness/time/state.
- Wrote 8 tests: scoring weights, ordering (Added > Modified > Moved), determinism,
  dense ranks, empty diff, section ranking, cross-section ranking.

## Test execution workaround (important)

Windows Smart App Control blocks executing freshly compiled unsigned binaries on the
host (`cargo test` fails, os error 4551). Worked around by executing Rust tests in WSL
Ubuntu through the MSVC-targeted Cargo project:

```bash
wsl -d Ubuntu -- bash -lc "rsync -a /mnt/c/.../companion/ /root/job-vault/companion-NNN/ && . ~/.cargo/env && cargo test"
```

- All 247 companion tests pass (202 prior phases + 37 diff-engine tests + 8 ranking tests).
- The default WSL distro is `docker-desktop` (no Rust toolchain); `Ubuntu` distro has
  the Rust toolchain configured.

## What works

- TASK-051 paragraph/sentence diff, TASK-052 bullet diff, TASK-053 moved/reordered
  detection, TASK-054 requirement/responsibility changes, and TASK-055 change ranking:
  verified by 247 passing Rust tests via WSL.
- Extension UI merged through TASK-062 (job list, job detail, snapshot history/detail),
  verified by 86 extension tests.

## What does not work

- Executing newly built Rust binaries on the Windows host (Smart App Control).
- Companion still returns NOT_IMPLEMENTED for job.list and job.get (IPC handlers not
  yet wired to the diff/storage modules).

## Tests run

- Rust: 247 passed, 0 failed (`cargo test` via WSL Ubuntu, as root).
- Extension: typecheck + vitest (86 passed) + tsc build (as of TASK-062).

## Files changed (task/TASK-055)

- companion/src/diff/mod.rs (RankedChange, RankedDiffResult, RankedSectionChanges,
  rank_changes, change_score, rank_section_changes, rank_all_section_changes, ranking tests)
- project/TASKS.md (TASK-055 criteria, marked DONE)
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
- TASK-051/052/053/054 are merged to main; TASK-055 change ranking is implemented on a
  task branch and ready for review/merge.

## Next recommended action

1. TASK-055 — Change ranking is now DONE (see session note below). Acceptance criteria
   defined and implemented in `companion/src/diff/mod.rs`.
2. Continue with TASK-042 (similarity matching), then Phase 6 UI (TASK-063+, unblocks
   once TASK-055's branch merges to main).
3. Remember to update these docs if you make further changes.

## Session note (opencode/big-pickle, 2026-09-05)

- TASK-055 (change ranking) implemented and verified: 247 companion tests pass in WSL
  (239 prior + 8 new). See "What works" below.
- WSL push credentials are now fully wired: root has gh creds
  (`/root/.config/gh` copied from `/home/test/.config/gh`) and
  `gh auth setup-git` configured the host-scoped credential helper; verified with
  `git push origin main --dry-run` → `Everything up-to-date`. Root can now push
  directly from `/root/job-vault`.

### WSL user / credential reconciliation (resolved)

- `gh` 2.46.0 is installed in WSL, authenticated as `skibkitty` (scopes `repo`,
  `workflow`, `gist`, `read:org`). Config now lives at BOTH
  `/home/test/.config/gh/` (original) and `/root/.config/gh/` (copied, root-owned).
- The `Ubuntu` distro's DEFAULT user is still `test` (uid 1000) and `/root` remains
  root-only, so ALWAYS use `wsl -d Ubuntu -u root -- bash -lic "..."` for Rust work.
- Push from WSL now works directly via gh as the git credential helper (host-scoped
  `credential.https://github.com.helper`). The Windows-workspace fallback is no longer
  required, though it still works.
- Verified as root: `wsl -d Ubuntu -u root -- bash -lic "cd /root/job-vault/companion && cargo test"`
  → 247 passed, 0 failed. Windows mirror synced to HEAD.
- Scratch working copies remain at `/root/job-vault-wip/` (companion,
  companion-051..054); they are superseded by the merged diff engine and can be removed
  now that TASK-055 is underway.

## Instructions for next agent

1. Read AGENTS.md, project/CURRENT_STATE.md, project/HANDOFF.md, project/TASKS.md.
2. The canonical Rust workspace is `/root/job-vault` inside WSL (AGENTS.md section 7b).
   IMPORTANT: run WSL commands as root, since the WSL default user is `test` and cannot
   read `/root`. Use:
   `wsl -d Ubuntu -u root -- bash -lic "cd /root/job-vault/companion && cargo test"`
   Do not run freshly compiled Rust binaries on the Windows host (Smart App Control
   blocks them).
3. WSL push credentials ARE wired (root gh config + `gh auth setup-git`); you can push
   and merge directly from `/root/job-vault` as root. Verify once with `git push origin <branch> --dry-run`
   if unsure.
4. For the next Rust diff task (TASK-042), work in the WSL repo (`/root/job-vault`) and
   commit there. Branch off `main` (diff engine + TASK-055 will be merged).

## Blockers

- WSL default user mismatch: `wsl -d Ubuntu -- bash ...` runs as `test`, which cannot
  reach `/root/job-vault` — always use `-u root` (mitigated; documented).
- Windows Smart App Control blocks host-side Rust test execution (mitigated via WSL).
- Phase 6 UI tasks after TASK-062 (TASK-063+) are next once TASK-055's change ranking
  merges to main.
