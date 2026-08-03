# Combat-Eligible Certified Power Correction Experiment Notes

Status: complete notes

Implementation complete: true

Created: 2026-07-28

## 1. Dependency

- Master P01; direct predecessor P00 correction `Complete / High`.
- Direct successor P02 checkpoint-owned Floor anchor consumes this formula/model version.

## 2. Approved Boundary

- 현재 combat-eligible 전체 roster만 certified power에 포함한다.
- Current HP/Trauma의 정확한 수치는 formula가 아니고 eligibility/readiness만 결정한다.
- Run peak는 historical high-water로 유지한다.
- Formula meaning 변경은 model version bump와 old-version rejection을 동반한다.

## 3. Reaudit Conflict To Reconfirm

- 재감사 당시 current formula는 `is_alive()`만 filter해 Critical/Breakdown을 포함했다.
- current relative threat도 같은 total을 denominator로 사용했다.
- 실제 current code/live version/callers는 production 수정 전에 다시 확인한다.

## 4. Dirty Worktree

- `combat_power.rs`는 untracked이고 employee/world/RunPolicy/RON/canonical은 modified다.
- 기존 campaign의 green baseline을 사용자 변경으로 보존하고 target function/version/test만 국소 수정한다.

## 5. Policy Questions

현재 없음. 새 coefficient, second power authority, external DTO 또는 save compatibility가 필요하면
`policy_decision_required`로 전환한다.

### 5.1 Technical implementation choice

- `Employee::is_combat_eligible()`가 이미 battle entry/deployability와 exact approved boundary를 소유하므로
  별도 damage/trauma 상수를 power module에 복제하지 않는다.
- Formula 의미 변경 generation은 live `power_model_version` 1에서 2로 올린다. Version 1 active-run/node
  authority는 current resolver/continue validation에서 explicit mismatch로 거부되며 migration/default를
  추가하지 않는다.
- Coefficient와 public DTO shape는 바꾸지 않는다.

## 6. Out-Of-Scope Candidates

없음. P02 anchor와 P03 monolith 문제를 이 child에 선행 구현하지 않는다.

## 7. Implemented Authority

- Certified membership은 별도 복제 predicate 없이 `Employee::is_combat_eligible()` 하나를 사용한다.
- Formula 소비자는 current map relative threat, first Floor authority initialization, actual combat-entry peak로
  유지되어 동일한 version-2 total을 공유한다.
- Historical peak update와 active-run/backup max merge는 변경하지 않아 직원 손실·상태 악화·load가 peak를
  낮추지 않는다.
- Zero eligible total은 enemy target이 존재하는 map projection에서 `VeryHigh`이고 readiness는 별도 warning
  scan을 유지한다.
- Version 1 active-run과 manifested node difficulty는 existing strict mismatch branch에서 거부된다. Migration,
  alias, dual reader와 default compatibility는 추가하지 않았다.

## 8. Validation Summary

- 수정 전 matrix regression이 Critical 포함 결함을 재현했다.
- 수정 후 focused formula/threat/version/peak/RON이 모두 통과했다.
- Broad는 Core 789, integration 47, server 53, 실패 0이다.
- Rust format과 scoped whitespace 검사가 통과했다.

## 9. Completion

- Completion review: `Complete / High`, 2026-07-28.
- P02는 이 version-2 certified input을 전제로 checkpoint-owned Floor anchor/save schema를 수정할 수 있다.
