# R06 WP02 D04 실제 대상 집계 독립 단계검토

작성일: 2026-09-26. 대상: `GS-CAVER-P2G50-001 / GOAL-CAVER-01 / PHASE-CAVER-01`, WP02 부분 구현.

**판정: 수정 필요 — D04-F01의 회귀를 교정한 successor target의 재검토가 필요하다.**
제품 소스와 기존 동결본은 수정하지 않았다. 독립 시험은 동결 archive를 푼 별도 사본에서 수행했다.

## D04-F01 — [P2] 대상이 사라진 target-locked projectile의 빈 impact가 오류로 종료됨

변경 위치: `core_enhanced/src/game/battle/core/sim.rs:2903`–2910,
`execute_ordered_skill_effects`의 새 `first_capture` 조회.

새 코드는 대상별 amount loop에 들어가기 전에 다음 조회를 모든 DamageExpr/HealExpr에 수행한다.

```rust
capture_owner.capture_for(effect_index, targets.first().copied())
```

target-locked projectile은 release 때 `target_at_release=Some(target)`인 capture를 가진다.
발사 후 target이 사라지면 기존 homing runtime은 정상적으로 terminal impact를 `first_hit_unit_id=None`으로 enqueue한다.
`apply_skill_projectile_impact`는 active target을 필터링해 빈 `targets`를 ordered executor에 넘긴다.
이때 새 조회는 `capture_for(effect_index, None)`이 된다. `capture_for`는 같은 target 또는 unbound capture만 찾으므로
고정 target capture와 일치하지 않아 다음 오류를 반환한다.

```text
InvalidStaticData("value-expression effect has no capture for its resolved set")
```

이는 미지원 집계 조합이나 실제 scalar read의 MissingContext가 아니다. 대상별 amount 평가가 하나도 필요 없는
정상 empty impact에서 metadata를 얻기 위한 lookup이 실패한 것이다. aggregate가 없는 `Const(1 Magnitude)`도
Snapshot/Live 모두 영향을 받는다. 오류는 event 소비 caller로 전파되어 해당 candidate의 정상 종료를 막는다.

### 재현 및 선행 대조

[projectile_probes.rs](projectile_probes.rs)는 기존 제품 시험의 실제 `invoke_ability` → projectile step →
queued event 처리 경로를 사용한다. release 후 test fixture에서 target unit을 제거해 기존 target-lost 분기로 들어간다.
capture나 targets를 위조하지 않으며, production 동작 코드를 고치는 hook도 없다.

| 소스 | 조건 | 결과 |
| --- | --- | --- |
| 선행 action/Snapshot 동결본 | 상수식 Snapshot, target 소실 | PASS |
| 선행 action/Snapshot 동결본 | 상수식 Live, target 소실 | PASS |
| D04 동결본 | 상수식 Snapshot, target 소실 | 위 InvalidStaticData로 FAIL |
| D04 동결본 | 상수식 Live, target 소실 | 같은 오류로 FAIL |
| D04 동결본 | ResolvedTargets Sum Snapshot, target 소실 | 같은 오류로 FAIL |
| D04 동결본 | ResolvedTargets Sum Live, target 소실 | 같은 오류로 FAIL |

증거: [D04 4건 실패 로그](projectile_probes_001.log), [receipt](projectile_probes_001.json),
[선행 상수식 2건 통과 로그](predecessor_control/scalar_projectiles_001.log),
[선행 receipt](predecessor_control/scalar_projectiles_001.json).
선행에는 실제 resolved aggregate adapter가 없었으므로 aggregate 두 조합을 선행 PASS로 주장하지 않는다.

### 교정 방향과 재검토 조건

- effect/plan metadata 취득이 `targets.first()`의 존재와 고정 target 일치에 의존하지 않도록 한다.
  또는 amount를 소비할 target이 없는 경우의 준비 경로를 분리하되 기존 empty-delivery 완료 의미를 보존한다.
- 실제 target이 있을 때의 capture identity 검사는 유지한다. 아무 capture나 fallback으로 고르거나 고정 target 검사를 제거하지 않는다.
- skill 전체를 조기에 반환해 다른 effect, 결과/count, prefix settlement, delivery close를 건너뛰는 교정은 피한다.
- 위 상수식·resolved aggregate × Snapshot/Live의 target 소실 4건을 제품 회귀 시험으로 고정한다.
  정상 hit의 release/impact 값 구분, area tick, empty frame의 Count/Sum/Max 및 context 미제공 거부도 유지한다.
- 필요하면 miss/timeout 및 inactive target 경로를 같은 empty-impact 원인에 매핑해 확인한다. 다음 후보만 성공시키는 특례로 처리하지 않는다.
- 교정 후 actual successor source에 빌드·관련 focused/delivery 회귀를 결속하고, 공유 executor 영향에 맞는 전체 회귀와 비용 검증 결과를 인계한다.

## 대상 및 currentness

- [검토 인계서](../../segments/wp02_d04_resolved_target_001/REVIEW_HANDOFF.ko.md)
- [동결 manifest](../../segments/wp02_d04_resolved_target_001/target/d04_resolved_target_001/MANIFEST.json)
- Target SHA-256: `24bd3b7e748b94037b2ec57822a7b7ef2b67406b9d2635a3a23a6b3eac7ba8bf`
- D04 최종 333개 digest: `4c63ca018cd39d50e4a896711debaaf7970dc2636841a77eb74457f83313582f`
- 선행 target SHA-256: `b16697bfe8c518944f0bddcf800b0f1e005ed639edd6fe915e4d96c726a6e1f4`
- 선행 333개 digest: `60fe6643fa136a16773e93042d9db7d3e5b9398088c495771796abd410708717`

변경 9개 파일의 delta와 실제 구현을 [D04 지시서](../../handoffs/WP02_D04_IMPLEMENTATION_REQUEST_20260926.ko.md)에 대조했다.
WholeSet/ByProjectedTile, explicit member Attack/HP, 공유 Snapshot payload, Live의 effect별 준비,
context 미제공/empty 구별, typed refusal, 비용 admission 및 rollback 연결을 검토했다.
resolved ByOwner와 implicit scalar producer의 잔여 처리는 지시서에 명시된 경계이며 이번 finding이 아니다.

`freeze.py verify`의 입구·출구 currentness/양방향 복원 결과를 보존했다.
[verify.py](verify.py)는 현재 333개 파일, after archive의 정확한 membership/내용, records 40개·artifacts 5개,
채택 receipt와 로그 해시, PC copied/built/run identity 및 독립 시험의 hook만 있는 소스 차이를 확인한다.
[verification.json](verification.json)에 결과를 기록했다.

## 직접 실행 및 재사용 증거

| 구분 | 검증 | 결과 |
| --- | --- | --- |
| 직접: 동결 소스 | `cargo test -p game_core --lib value_expr --offline --locked` | 74/74 PASS, [로그](focused_001.log) |
| 직접: test hook 추가 사본 | 기존 target-locked Snapshot/Live 정상 hit 시험 | 1/1 PASS, [로그](target_locked_success_001.log) |
| 직접: test hook 추가 사본 | 기존 area 반복 tick의 Live member 재준비 시험 | 1/1 PASS, [로그](area_success_001.log) |
| 직접: D04 test hook 사본 | 신규 target 소실 시험 | 0/4, 모두 D04-F01 오류 |
| 직접: 선행 test hook 사본 | 같은 상수식 target 소실 대조 | 2/2 PASS |
| 재사용: 최종 digest·로그 검증 | check, 전체 제품 시험, Clippy, 변경 영역 format | exit 0; 전체 1,500/1,500 |
| 재사용: 실제 격리 source 결속 검증 | PC 36조건 | 성공31/typed refusal5, 기존 gameplay oracle·상태·오류 code 차이 0 |
| 재사용: 최종 source 결속 검증 | R05 allocation guard | peak 8,997,016 < 원 cap 9,030,188 bytes |

재사용한 정확한 receipt는 [reused_evidence.json](reused_evidence.json)에 있다. 전체 1,500건과 PC/R05를 이번 검토에서
다시 실행했다고 주장하지 않는다. R05 latency gate는 미적용이다. 기존 sandbox socket 실패 기록도 그대로 유지했다.

검토 도구는 Rust/Cargo 1.98.1, offline/locked 조건이다. 검토자가 이전에 사용한 `/tmp` build cache를 재사용했으며,
각 receipt의 cwd·source 전후 hash는 실제 새 격리 사본을 기록한다. 두 source에서 같은 probe 파일을 사용했고
각 사본의 유일한 제품 소스 차이는 `core/mod.rs`의 `#[cfg(test)]` 모듈 안에 넣은 probe module hook이다.
원 작업공간의 제품 소스 333개는 모든 직접 실행 전후 변하지 않았다. compile supplemental 입력 2개도 config에 기록했다.

## 다음 단계

이 동결본은 보존하고 **D04-F01 교정 구현 → successor target 동결 → 원 대화의 독립 재검토**로 진행한다.
이번 검토에서 제품 교정을 수행하거나 D04 부분 PASS·WP02 전체 완료를 선언하지 않는다.
ByOwner 표현 결정, D07 preview/explain, WP03/04와 production publication 해제는 여전히 후속 범위다.
