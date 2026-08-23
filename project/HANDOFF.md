# Agent Handoff

## Current session

Agent:
opencode/big-pickle

Date:
2026-08-23

Task:
TASK-051 (BLOCKED — implementation complete, test execution blocked by OS policy)

## What has been done

- Implemented paragraph/sentence diff engine in `companion/src/diff/mod.rs`.
- `TextDiffEngine` with:
  - `split_paragraphs` (blank-line separated, CRLF-safe, trimmed);
  - `split_sentences` (`.`, `!`, `?`; keeps closing quotes; does not split decimals like `3.5`);
  - `similarity` (Jaccard over normalized tokens, public for reuse);
  - `diff_paragraphs`, `diff_sentences`, `diff_texts`;
  - `diff_segments` (LCS alignment over TASK-050-normalized segment text; runs of
    removals/additions between matches are greedily paired into Modified when
    similarity >= threshold, else emitted as Removed/Added);
  - configurable similarity threshold (`new()` default 0.5, `with_similarity_threshold`).
- Results preserve original un-normalized text; `old_index`/`new_index` locate each change.
- Deterministic: no randomness, no time dependence.
- No new dependencies, no network, no LLM.
- Registered module in `companion/src/main.rs`.
- Wrote 30 unit tests covering splitting, classification, normalization tolerance,
  determinism, and threshold behavior.
- Defined acceptance criteria for TASK-051 in `project/TASKS.md`.

## Blocker (important for next agent)

Windows Smart App Control is On on this machine. It blocks execution of freshly
compiled unsigned binaries:

- `cargo test` compiles but fails to run the test harness: "An Application Control
  policy has blocked this file. (os error 4551)".
- A fresh `CARGO_TARGET_DIR` also fails earlier (proc-macro2 build script blocked).
- Existing cached build artifacts still run (the previously built companion exe runs).
- Diagnostics: `(Get-MpComputerStatus).SmartAppControlState` = On;
  `VerifiedAndReputablePolicyState = 1` under HKLM\SYSTEM\CurrentControlSet\Control\CI\Policy.
- SAC ignores path exclusions and self-signed code signing; only the user can turn it
  off (Settings > Privacy & security > Windows Security > App & browser control), which
  is permanent until Windows reinstall. Human decision required.
- The previous session ran cargo tests successfully earlier today, so SAC state likely
  changed from Evaluation to On between sessions.
- WSL is present but its default distro is docker-desktop (unsuitable for a Rust toolchain).

## What works

- `cargo check` / `cargo build` succeed (cached artifacts reused); zero errors, only
  pre-existing dead-code warnings in `matching`.
- All TASK-051 functionality as described above (compile-verified, not runtime-verified).

## What does not work

- Executing any newly built Rust binary or build script on this machine (SAC).

## Tests run

- `cargo test` — could not execute (os error 4551, blocked by Smart App Control).
  30 new tests written; NOT yet executed. Previous suite count was 172 passing.

## Files changed

- companion/src/diff/mod.rs (new)
- companion/src/main.rs (added `mod diff;`)
- project/TASKS.md (TASK-051 criteria defined, status BLOCKED)
- project/CURRENT_STATE.md
- project/HANDOFF.md

## Important discoveries

- Cargo is not on PATH in this shell: use `& "$env:USERPROFILE\.cargo\bin\cargo.exe"`.
- Smart App Control On makes all Rust test execution impossible for any agent until
  the user changes the setting.

## Decisions

- LCS-based alignment hand-rolled instead of adding a diff crate (dependency
  minimization policy; algorithm is non-cryptographic and fully auditable).
- Greedy sequential pairing of removed/added segments with Jaccard threshold 0.5 to
  label Modified vs Added/Removed. Simple, deterministic; refine in TASK-052/054 if needed.

## Known risks

- Diff quality of greedy pairing unverified at runtime until tests can run.
- Other agents must not assume tests pass on this machine without checking SAC state.

## Next recommended action

User resolves Smart App Control (or permits local build output), then:
run `cargo test` in `companion/` on branch `task/TASK-051`, fix any failures, mark
TASK-051 DONE, update docs, commit, push, create PR.

Alternative work unaffected by the blocker: extension-side tasks (TypeScript/vitest via
already-reputable node.exe), e.g. TASK-060 Job list (READY: TASK-022 DONE, TASK-007 DONE).

## Instructions for next agent

1. Read AGENTS.md, project/CURRENT_STATE.md, project/HANDOFF.md, project/TASKS.md.
2. Check `(Get-MpComputerStatus).SmartAppControlState`.
3. If Off: run `& "$env:USERPROFILE\.cargo\bin\cargo.exe" test` in `companion/`,
   resolve failures, complete TASK-051 per its acceptance criteria.
4. If still On: do not attempt Rust tasks; pick an extension-side READY task instead.

## Blockers

Windows Smart App Control blocking execution of freshly compiled binaries (see above).
