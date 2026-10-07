# Baseline

- Goal: `current_runtime_architecture_deep_review`
- Status: `complete`
- Baseline ID: `BASELINE-20260822-064427-KST`
- Capture start: `2026-08-22T06:44:27+09:00`
- Capture end: `2026-08-22T06:46:47+09:00`
- Workspace: `/mnt/f/work/simulator/core_enhanced`
- Repository root: `/mnt/f/work/simulator`

## Git Identity

- HEAD: `f318414fb0156a67e04c5479000c2a2683fc0c09`
- Worktree: substantially dirty before this Goal; existing modified, deleted and untracked files are user-owned.
- Exact `git status --short` capture: `BASELINE_GIT_STATUS.txt` (325 lines)
- The Goal will not reset, restore, clean, stash, format or otherwise mutate those files.

## Audited Runtime/Test Manifest

- Included: `Cargo.toml`, `src/lib.rs`, current files under `src/game/**` and `tests/**`
- File count: 184
- Total bytes: 5,553,490
- Per-file SHA-256 manifest: `BASELINE_MANIFEST.sha256`
- Aggregate identity: SHA-256 of the sorted `sha256sum` manifest:
  `ae1b66e9218ee493bffaa4da3f9b9d7f0bbd8849516d6e2a5f49749be78acb31`
- `Cargo.toml`: `b7fabaf43f0f5aa21b36709f5ff80085ca3d59522edf5748532321477d6f5bab`
- Workspace `../Cargo.lock`: `e1597f3369caa9637fb7e41c6ca5230b0954926610bc661933123a446ba17836`

Reproduction command:

```bash
rg --files src/game tests Cargo.toml src/lib.rs | sort | xargs sha256sum
```

The aggregate is reproduced by piping that manifest to `sha256sum`. Deleted paths are represented by the Git-status capture,
not by the current-file manifest.

## Candidate Source Identity

- Candidate count: 13
- Individual byte sizes and hashes: `DOCUMENT_SOURCE_MAP.md`
- Aggregate identity of candidate `sha256sum` output:
  `d954134db148e6ce98e375a3857228afa4d42ec49eef6e0b78114d2347151013`
- `docs/game_rulebook.md` is always an evidence source. Other candidates enter the evidence baseline only when a routed section
  is used. Changes to skipped/irrelevant candidates alone do not invalidate conclusions.

## Toolchain And Environment

- OS: `Linux Acr0tic 6.18.33.2-microsoft-standard-WSL2 #1 SMP PREEMPT_DYNAMIC Thu Jun 18 21:54:43 UTC 2026 x86_64 GNU/Linux`
- Host: `x86_64-unknown-linux-gnu`
- Rust: `rustc 1.91.1 (ed61e7d7e 2025-11-07)`
- Cargo: `cargo 1.91.1 (ea2d97820 2025-10-10)`
- LLVM: `21.1.2`
- Declared feature: `event_log_checks = []`
- Broad baseline feature selection: `--all-features`, therefore `event_log_checks` is enabled.

## Adjacent Live Data

Not accessed. No `../game_resources/data` manifest has been captured yet. If a criterion requires live authoring evidence, its
exact path and snapshot will be pre-registered before the first content read.

## Baseline Change Rule

Before using new evidence and at completion, compare the audited runtime/test aggregate, HEAD, Rulebook, all actually used
source sections, `Cargo.lock`, and any accessed live-data snapshot. Do not combine evidence from different identities without a
recorded user decision.
