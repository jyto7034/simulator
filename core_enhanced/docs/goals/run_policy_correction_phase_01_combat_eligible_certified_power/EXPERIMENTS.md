# Combat-Eligible Certified Power Correction Experiments

Status: complete

Implementation complete: true

Created: 2026-07-28

## 2026-07-28 — Child Goal 초기화

- Master P00 correction completion review: `Complete / High`.
- P01 child 경로는 생성 전에 존재하지 않았다.
- Master/재감사 R-03이 formula 대상과 run peak 의미를 승인해 policy question은 없다.
- 관련 Core/RON/canonical 경로는 기존 implementation campaign 변경과 겹친다.
- Production code/data/canonical 변경과 test 실행은 아직 없다.
- 다음 작업: child 문서 완독, current formula/call/version/diff 조사, focused baseline.

## 2026-07-28 — Current-state와 focused baseline

### Current evidence

- `certified_combat_power_for_roster()`는 현재 `employee.is_alive()`만 filter한다. 따라서 physical damage 5
  Critical과 trauma 100 Breakdown이 persistent profile power를 합산한다.
- `Employee::is_combat_eligible()`는 공통 eligibility authority를 사용하며 alive, physical damage 0..=4,
  trauma 0..=99를 정확히 표현한다.
- Current map relative threat, Floor 최초 authority 초기화와 committed combat entry peak가 모두 같은
  `certified_combat_power_for_roster()`를 호출한다. Formula filter 한 곳을 바꾸면 세 소비자가 수렴한다.
- Relative threat는 certified total 0일 때 `VeryHigh`를 사용하고 별도 readiness scan이
  `NoDeployableEmployees`, Breakdown, Critical warning을 보존한다.
- Live `power_model_version`은 1이고 active-run continue와 manifested difficulty resolver가 current live
  version mismatch를 fail closed한다.

### Dirty gate

- Formula module은 기존 campaign의 untracked 파일이고 world/employee/RON/canonical은 modified다.
- 기존 call graph와 strict version 경계는 승인 목표와 양립하며 formula filter, version, 직접 tests와
  canonical만 국소 변경할 수 있다. 기존 coefficient/DTO/anchor 의미를 덮어쓸 충돌은 확인되지 않았다.

### 수정 전 baseline

| 명령 | Pass | Fail | Filtered | 결과 |
| --- | ---: | ---: | ---: | --- |
| `cargo test -p game_core combat_power --lib -- --nocapture` | 3 | 0 | 785 | 현 all-living 정책 test 포함 성공 |
| run peak exact focused | 1 | 0 | 787 | 성공 |
| `cargo test -p game_core --test ron_loading -- --nocapture` | 23 | 0 | 0 | 성공 |
| active-run checkpoint round-trip exact focused | 1 | 0 | 787 | 성공 |

기존 `certification_is_deterministic_and_ignores_trauma_and_current_hp`는 trauma 100 employee도 동일 power를
기대해 승인 정책과 반대다. 새 eligibility matrix test로 현재 결함을 먼저 재현하고 이 test를 eligible
범위의 magnitude 불민감 계약으로 교체한다.

## 2026-07-28 — Eligibility regression 수정 전 실패

- 새 `certification_membership_matches_combat_eligibility_boundaries`는 physical damage/trauma 경계
  0/4/5/6 및 99/100을 직접 검사한다.
- 수정 전 exact runner: 0 passed, 1 failed, 788 filtered.
- 첫 mismatch는 `physical_damage=5, trauma=0` Critical 직원이 예상 제외와 달리 members에 포함된 것이다.
- 이는 P-01/R-03 current defect를 직접 재현한다. 다음 production 변경은 공통
  `is_combat_eligible()` filter와 model version bump로 제한한다.

## 2026-07-28 — 승인 경계 구현과 focused 검증

### Production 변경

- `certified_combat_power_for_roster()`의 roster filter를 `is_alive()`에서 이미 전투 배치가 공유하는
  `Employee::is_combat_eligible()`로 교체했다.
- 기존 condition 불민감 test는 eligible 범위인 physical damage 4/trauma 99에서 current HP와 condition
  magnitude가 coefficient가 아님을 검증하도록 바꿨다.
- live `run/policy.ron`의 `power_model_version`을 1에서 2로 올렸다. Coefficient, band, DTO shape는 바꾸지
  않았다.
- active-run checkpoint round-trip에 version 1 continue rejection을 추가했다. Current import가 기대하는
  version보다 하나 작은 authority는 `InvalidAction`으로 실패하고 current version은 이어서 성공한다.
- 전원 trauma 100 fixture에서 current certified total 0, relative threat `VeryHigh`,
  `NoDeployableEmployees`와 `TraumaBreakdown`을 함께 검증했다. Manifested node target/band는 바뀌지 않는다.
- live model을 뜻하는 combat-preview/facility test fixture 세 곳만 version 2로 갱신했다. Generic server
  persistence 단위 fixture의 독립적인 version 값은 변경하지 않았다.

### Focused 결과

| 검증 | Pass | Fail | 핵심 증거 |
| --- | ---: | ---: | --- |
| eligibility matrix | 1 | 0 | physical 0/4 포함, 5/6 제외; trauma 99 포함, 100 제외 |
| combat power module | 4 | 0 | 결정론·eligible condition/HP 불민감·장비/fragment 포함 |
| map qualitative threat | 1 | 0 | zero eligible power → `VeryHigh` + no-deployable/breakdown |
| active-run checkpoint/version | 1 | 0 | stale model reject, current model round-trip |
| run peak authority | 1 | 0 | Floor initialization/combat entry와 historical peak 유지 |
| live RON loading | 23 | 0 | version 2 live policy strict load/validation |

## 2026-07-28 — Broad 검증과 종료 판정

- 첫 Core broad는 786 passed, 3 failed였다. 실패는 formula가 아니라 live policy를 의미하면서 version 1을
  고정한 test fixture 세 곳이었다. 해당 fixture만 version 2로 갱신한 뒤 Core lib 789/789가 통과했다.
- `cargo test -p game_server`: 53 passed, 0 failed.
- `cargo test --workspace --all-targets`: Core lib 789, integration 3/4/23/13/4, server 53, 총 실패 0.
- `cargo fmt --all -- --check`: 통과.
- P01 대상 source/data/canonical의 `git diff --check`: 통과.
- Canonical은 exact eligibility, eligible condition magnitude 비계수, Critical/Breakdown/Dead 제외, zero-power
  threat와 live model version 2를 명시한다.
- Unity-visible DTO shape는 변경되지 않았고 exact internal power/version도 계속 wire에 비노출이므로 Unity
  runtime 변경이나 Unity MCP 실행은 필요하지 않았다.
- 독립 completion review 결과는 `Complete / High`다.
