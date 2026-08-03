# Checkpoint-Owned Floor Anchor Correction Experiment Notes

Status: complete notes

Implementation complete: true

Created: 2026-07-28

## 1. Dependency

- Master P02; direct predecessor P01 correction `Complete / High`.
- Direct successor P03 stores typed monolith stage difficulty through this corrected checkpoint/save seam.

## 2. Approved Boundary

- Current Floor anchor와 power model version은 checkpoint-owned다.
- Loads와 run peak만 active-run host monotonic metadata다.
- Backup은 자신의 checkpoint payload anchor를 보존하며 다른 generation anchor를 max-merge하지 않는다.
- Breaking schema는 version bump와 old-version rejection을 사용하고 migration/dual reader는 없다.

## 3. Reaudit Conflict To Reconfirm

- 재감사 당시 anchor/model version은 checkpoint payload 밖 `RunPowerAuthority`와 host monotonic envelope에
  있었고 backup publication이 anchor를 max-merge했다.
- 과거 Floor checkpoint load가 newer anchor를 유지해 map/node difficulty generation과 갈라질 수 있었다.
- Current code/version/call order와 existing regression을 production 수정 전에 다시 확인한다.

## 4. Dirty Worktree

- world state/active-run/facility tests, server active-run store/actor와 canonical은 기존 campaign 변경과 겹친다.
- Current scoped source와 diff를 읽고 checkpoint/save target field와 직접 tests만 국소 수정한다.

## 5. Policy Questions

현재 없음. Migration, new DTO 또는 anchor 재계산 규칙이 필요하면 `policy_decision_required`로 전환한다.

## 6. Out-Of-Scope Candidates

없음. P03 monolith typed stage와 reward projection을 이 child에서 선행 구현하지 않는다.

## 7. Final Ownership

- Checkpoint payload: model version, current Floor anchor, map/node difficulty와 모든 rewindable run state.
- Core/host monotonic: successful load count와 run peak only.
- Host rolling backup: previous Core checkpoint bytes를 그대로 유지하고 loads/run peak만 max-merge.
- Runtime `RunPowerAuthorityState`: current generation을 실행 중 보유하지만 checkpoint load와 failed host
  transaction에서 checkpoint/rollback generation으로 exact 복원.

## 8. Strict Compatibility Boundary

- Core payload version 2와 host envelope version 3만 current다.
- 구 Core payload의 missing/version-1 document, stale power model, mismatched checkpoint/node anchor, envelope
  version 2와 removed monotonic anchor field는 거부된다.
- Migration, dual reader, serde default/alias와 anchor recompute fallback은 없다.

## 9. Validation Summary

- 수정 전 Floor 1 map + Floor 2 anchor mismatch를 100 대 125로 재현했다.
- 수정 후 cross-Floor replay, Core export/continue와 server actual-filesystem backup regressions가 통과했다.
- Broad는 Core 790, integration 47, server 54, 실패 0이다.
- Unity public read model은 loads/continue UX만 소비하고 internal save generation field를 소비하지 않는다.

## 10. Completion

- Completion review: `Complete / High`, 2026-07-28.
- P03는 Core payload V2/envelope V3 checkpoint에 typed monolith stage difficulty를 저장해야 한다.
