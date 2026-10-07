# Game Rulebook Runtime Alignment Audit Goal

Status: complete
Implementation Complete: not_applicable
Goal Type: single
Execution Mode: investigation_only
Production Mutation Authorized: no

## Objective

현재 working tree의 [`docs/game_rulebook.md`](../../game_rulebook.md)가 설명하는 gameplay 규칙을
실제 production runtime code, live RON/data, serialization·server seam과 접근 가능한 소비자에
대조하여 불일치만 식별하고 재현 가능한 근거와 함께 분류한다.

이 Goal은 정정 Goal이 아니다. 코드, live data, schema/DTO, fixture/test, 외부 consumer와 canonical
문서는 수정하지 않는다.

## User/Contract-Visible Problem

룰북이 업데이트되었으나 현재 구현과 live data가 같은 의미를 구현하는지는 아직 검증되지 않았다.
룰북의 문장만 읽거나 symbol 존재만 검색해서는 다음을 구분할 수 없다.

- 현재 live path와 실제로 일치하는 규칙
- 승인됐지만 구현 대기 중인 문서 선행 정책
- 구현이 앞서 있고 룰북이 뒤처진 규칙
- schema에는 있으나 production live data가 사용하지 않는 규칙
- producer와 server/client consumer가 서로 다르게 해석하는 계약
- 제거됐다고 문서화됐지만 live path에 남은 legacy/fallback
- 미래 후보·dormant authoring처럼 현재 불일치가 아닌 문장

완료 결과는 전체 룰북을 빠짐없이 대조했다는 coverage evidence와, 실제 불일치 항목만 담은 catalog다.
어느 쪽을 수정할지 또는 어떤 gameplay 정책을 채택할지는 이 Goal에서 결정하지 않는다.

## Audit Finding ID

없음. 이 Goal은 기존 감사 finding에서 파생되지 않은 사용자 요청 기반 정합성 조사다.

## Current Runtime And Live Data Evidence

Goal 초기화 시점에 확인한 사실:

- 감사 대상 gameplay canonical은 현재 working tree의 `docs/game_rulebook.md`다.
- 룰북은 자신을 `Gameplay canonical`로 선언하면서도 실제 runtime code와 live RON/data가 문서보다
  우선하며, 충돌 시 코드를 먼저 확인하도록 명시한다.
- 룰북은 2026-08-12 정책 승인·구현 대기 주석에서 Standard 튜토리얼 재구성, 연구도 폐기,
  전투 기반 파편 획득, Elite 모노리스·소탕, Endless 재순환, Event/BossOmen 실패 처리와 영구
  개인 기록이 확정 정책이지만 runtime/live data에 이전 규칙이 남을 수 있다고 명시한다.
- 현재 repository는 광범위한 tracked/untracked 사용자 변경을 포함한다. `docs/game_rulebook.md`,
  Core code, game resources와 game server도 dirty 상태이므로 이 감사는 `HEAD`가 아니라 현재
  working tree를 대상으로 해야 한다.
- 실제 command-to-mutation 경로, production loader가 선택하는 live data, DTO producer/consumer와
  관련 test coverage는 아직 `discovery pending`이다. Goal 문서 초기화 전에 repository-wide 코드
  조사를 수행하지 않았다.

## Source Of Truth And Ownership

| 책임 | 조사 기준 source | 감사에서의 취급 |
| --- | --- | --- |
| 플레이어-visible gameplay 의미 | `docs/game_rulebook.md` | 감사 claim의 기준 문장. 구현 사실로 자동 간주하지 않음 |
| Core runtime state·mutation·rollback | `src/`와 `docs/core_runtime_contract.md` | 실제 reachable producer와 authoritative mutation을 함께 추적 |
| 타겟팅·delivery | `src/game/battle/`, `docs/skill_target_contract.md` | 룰북이 위임한 세부 계약은 인접 canonical과 runtime을 함께 확인 |
| live authoring·loader·validation | `../game_resources/data/`, `src/game/data/`, `docs/data_loading_contract.md` | schema 존재와 production live entry 사용을 구분 |
| server mapping·persistence | `../game_server/` | Core 결과가 transport/persistence에서 변형되는지 확인 |
| Unity wire·presentation 소비자 | 룰북이 지목한 외부 Unity canonical과 접근 가능한 source | 접근 가능 범위만 검증하고 미접근 항목은 검증했다고 표현하지 않음 |
| 실행 범위·증거 | 이 Goal의 문서와 감사 산출물 | durable gameplay 정책을 대신하지 않음 |

현재 runtime, live data, canonical과 승인된 목표 정책이 다르면 어느 하나를 자동으로 정답으로
선택하지 않는다. 각 finding은 `룰북 문장`, `현재 실제 동작`, `live data`, `인접 canonical/승인 상태`
를 분리해 기록한다.

## Invariants

- production code, live RON/data, schema/DTO, save/checkpoint, fixture/mock/test와 canonical을 수정하지 않는다.
- 현재 working tree의 사용자 변경을 보존하며 reset, restore, stash, cleanup 또는 formatter를 실행하지 않는다.
- symbol·enum·RON field의 존재만으로 규칙이 구현됐다고 판정하지 않는다.
- command/capability 진입점에서 authoritative state mutation 또는 최종 projection까지 실제 reachable
  path를 확인한다.
- schema/default/test fixture와 production live RON/data를 구분한다.
- dead code와 live code, authored candidate와 실제 pool/schedule assignment를 구분한다.
- 룰북이 인접 canonical에 위임한 구현 세부는 룰북 불일치로 오판하지 않는다.
- 명시적 `구현 대기`, `향후 후보`, `dormant`, `현재 계약이 아님`, `후속 authoring` 문장을 현재 live
  규칙 주장과 구분한다.
- 접근할 수 없는 외부 consumer는 일치로 판정하지 않고 evidence gap으로 기록한다.
- finding마다 양쪽 근거 경로와 가능한 경우 line/symbol/data identity를 남긴다.
- 불일치가 없는 claim은 최종 mismatch catalog에 넣지 않되 coverage matrix에는 판정 근거를 남긴다.
- 감사 결과로 correction, migration, compatibility 또는 새 gameplay 정책을 암묵적으로 승인하지 않는다.

## Approved Policy And Evidence

| ID | 승인 내용 | 근거 |
| --- | --- | --- |
| `GRB-AUD-SCOPE-001` | 룰북과 실제 코드·live data의 불일치만 찾는다. | 2026-08-12 사용자 요청 |
| `GRB-AUD-SCOPE-002` | 코드와 production 자료는 수정하지 않는다. | 2026-08-12 사용자 요청 |
| `GRB-AUD-SCOPE-003` | Goal 작성·실행 형식은 `goal_authoring_and_execution_guide.md`를 따른다. | 2026-08-12 사용자 요청 |

Gameplay 정책은 이 Goal이 새로 승인하지 않는다. 룰북의 승인·구현 대기 주석 및 stable decision
record의 실제 coverage는 조사 과정에서 claim별로 연결한다.

## Open Policy Decisions

없음. 현재 범위는 read-only 정합성 조사로 충분히 결정되어 있다.

조사 중 문장 해석에 따라 사용자-visible 의미가 달라지고 기존 canonical/decision record로 해소할 수
없는 경우, 해당 claim의 판단을 중단하고 `policy_decision_required`로 전환한다. 임의로 룰북 또는
구현을 정답으로 선택하지 않는다.

## In Scope

### Canonical claim inventory

- `docs/game_rulebook.md`의 모든 본문 절과 normative bullet
- 문서 책임과 source-of-truth 위임
- 2026-08-12 승인·구현 대기 주석
- Standard/Endless, run loop와 starter roster
- facility map, node, Gate, difficulty, inventory, consumable, Shop, Event와 briefing
- DefenseRoute 전투, tactical resource, CC/Silence, withdraw/redeploy, stance/tile effect,
  relocation/swap, manual skill, field work, Pause, setup/wave/targeting/damage/runtime transition
- reward/XP, 연구도 폐기, skill fragment, compendium/profile record, encounter pool, monolith/sweep,
  equipment와 equipment trigger
- SavePoint, Maintenance, HeadquartersContact
- result commit, attempt, retreat, employee condition, recovery/failure와 Psychology
- FinalBoss pool, BossOmen chain, hidden boss와 gameplay legacy 제거 원칙

### Implementation and data evidence

- `/mnt/f/work/simulator/core_enhanced/src`
- `/mnt/f/work/simulator/core_enhanced/tests`
- `/mnt/f/work/simulator/game_resources/data`
- `/mnt/f/work/simulator/game_server/src`, 관련 test/example과 canonical
- repository root manifest와 production loader가 위 경로의 live data를 선택하는 entrypoint
- 룰북이 직접 지목한 인접 Core/data/targeting/transport canonical
- 접근 가능한 경우 `/mnt/f/unity projects/ark`의 지목된 Unity canonical과 실제 consumer

### Audit artifacts

- `RULEBOOK_CLAIM_MATRIX.md`: 모든 auditable claim의 stable claim ID, 문서 위치, 기대 의미,
  runtime/data/consumer evidence와 판정
- `MISMATCH_CATALOG.md`: 불일치 finding만 담은 최종 결과
- `COMPLETION_AUDIT.md`: completion evidence matrix, scoped dirty preservation, canonical alignment,
  long-term fit과 조사 Goal 자체의 명시적 verdict
- `EXPERIMENTS.md`: 실제 검색·probe·test 명령과 결과
- `EXPERIMENT_NOTES.md`: 구조 판단, source-of-truth 충돌, 미검증 외부 경계와 남은 위험

## Out Of Scope

- production code, live data, schema/DTO, save/checkpoint, fixture/mock/test 수정
- `game_rulebook.md` 또는 다른 canonical/decision/설계 문서 수정
- 불일치의 수정 방향 선택, correction plan 또는 구현 Goal 자동 생성
- balance 수치의 적절성 평가. 단, 룰북과 live value가 다른지는 범위 안이다.
- 새 gameplay 정책, UX, compatibility, migration 또는 legacy 제거 승인
- art, animation, VFX와 순수 presentation 품질 감사
- 현재 룰북이 계약으로 선언하지 않은 미래 기능의 구현 준비도 평가
- 전체 repository 일반 코드 품질·리팩토링 후보 조사
- unrelated dirty file 정리, formatter와 dependency 갱신

범위 밖 문제를 발견하면 구현하거나 새 Goal을 만들지 않고 `EXPERIMENT_NOTES.md`의 후속 후보로만
기록한다.

## Implementation Scope

Production implementation은 없다.

감사 실행은 다음 work stage로 진행한다.

1. 현재 working tree와 접근 가능한 repository/external consumer baseline을 고정한다.
2. 룰북을 stable claim ID 단위로 분해하고 각 claim을 현재 규칙, 명시적 구현 대기, 미래 후보,
   인접 canonical 위임 중 하나로 먼저 태깅한다.
3. claim별 예상 authority, command/state/data identity, producer와 consumer 후보를 inventory한다.
4. production entrypoint에서 live path를 추적하고 live RON schedule/pool/definition이 실제로 선택되는지 확인한다.
5. DTO/save/transport가 관련된 claim은 serialization과 접근 가능한 consumer까지 추적한다.
6. 기존 focused test와 필요한 read-only probe를 사용해 static trace만으로 확정할 수 없는 동작을 검증한다.
7. 각 claim을 아래 판정 taxonomy로 분류하고 evidence가 부족하면 일치로 닫지 않는다.
8. 독립적인 completeness pass로 룰북 heading, claim ID, live data identity와 외부 seam 누락을 재검사한다.
9. 전체 matrix는 coverage evidence로 남기고 최종 catalog에는 불일치만 수록한다.

### Alignment taxonomy

| 판정 | 의미 | 최종 mismatch catalog |
| --- | --- | --- |
| `aligned` | 현재 live path와 룰북 의미가 일치 | 제외 |
| `delegated_detail` | 룰북이 세부를 인접 canonical에 위임했고 상위 의미는 충돌하지 않음 | 제외 |
| `future_or_dormant` | 룰북 자체가 현재 계약이 아니라고 명시 | 제외 |
| `document_ahead_expected` | 승인·구현 대기라고 명시된 정책이 아직 live path에 없음 | 포함 |
| `runtime_contradiction` | 실제 reachable 동작이 현재형 룰북 규칙과 반대 | 포함 |
| `document_stale` | live 동작이 존재하지만 룰북이 누락하거나 legacy로 설명 | 포함 |
| `live_data_gap` | code/schema는 지원하지만 production data가 룰북 의미를 작성·활성화하지 않음 | 포함 |
| `producer_consumer_mismatch` | Core/data와 server/접근 가능한 consumer가 서로 다른 shape·의미 사용 | 포함 |
| `legacy_live_path` | 제거 대상으로 명시된 계약이 live producer/data/consumer에서 reachable | 포함 |
| `test_contract_mismatch` | production 동작 또는 current rule과 기존 test expectation이 충돌 | 포함, 단 test만의 고립 문제임을 표시 |
| `evidence_gap` | 필수 source/consumer에 접근할 수 없어 판정 불가 | mismatch와 분리해 catalog의 미검증 절에 포함 |

### Finding record minimum fields

- Finding ID와 alignment taxonomy
- 룰북 heading, claim ID와 line anchor
- 룰북이 주장하는 현재/목표/미래 상태
- 실제 runtime 동작과 command-to-authority 경로
- production live data identity와 loader/assignment evidence
- DTO/server/consumer 영향 또는 해당 없음
- existing test evidence 또는 test gap
- 사용자-visible/contract-visible 영향
- 불일치 방향: document-ahead, runtime-ahead, data-only, consumer-only 또는 legacy
- 확신도와 미검증 경계
- correction은 수행하지 않았다는 명시

## DTO/Data/Save Migration

- DTO migration: 없음
- data/live RON migration: 없음
- save/checkpoint migration: 없음

감사에서는 현재 shape와 lifetime을 읽고 비교할 수 있으나 어떤 migration도 작성하거나 적용하지 않는다.

## Deployment And Compatibility

배포와 compatibility 변경은 없다. 현재 working tree의 producer와 consumer가 실제로 어떤 원자 배포
가정을 하는지는 finding의 영향도를 설명하는 데 필요한 범위에서만 조사한다.

승인되지 않은 fallback, dual-read/write 또는 alias를 발견하면 존재와 reachability를 기록할 뿐
제거하거나 정당화하지 않는다.

## Focused Validation

각 claim group은 최소한 다음 evidence 조합으로 판정한다.

- canonical heading/claim과 관련 stable decision record 확인
- production command/capability 또는 loader entrypoint에서 authoritative state까지 `rg` 기반 static trace
- production live RON identity, schedule/pool과 loader selection 확인
- 관련 serialization/server mapping 및 접근 가능한 consumer 확인
- existing focused test 이름과 assertion 확인
- static evidence로 의미가 확정되지 않을 때만 기존 test runner 또는 임시 read-only probe 실행

실행한 모든 명령은 실제 결과와 test 수를 `EXPERIMENTS.md`에 기록한다. runner가 수를 제공하지 않으면
추측하지 않는다. 임시 probe가 필요하면 `/tmp`만 사용하고 tracked/untracked production 경로에 파일을
만들지 않는다.

## Broad Validation

- `game_rulebook.md`의 모든 heading과 normative claim이 `RULEBOOK_CLAIM_MATRIX.md`의 claim ID 또는
  명시적인 non-auditable/delegated row에 매핑됐는지 확인한다.
- Core, game resources, game server와 접근 가능한 Unity seam의 조사 대상/생략 대상을 대조한다.
- live RON catalog/schedule에서 rulebook이 이름을 명시한 identity와 값이 누락 없이 확인됐는지 검사한다.
- `research_progress`, 폐기 mode/command, fallback/default, BossOmen, monolith, sweep, fragment,
  checkpoint/profile persistence 등 high-risk legacy·문서 선행 keyword를 독립 검색한다.
- 기존 test가 문서와 같은 의미를 보호하는지, 과거 계약을 보호하는지 구분한다.
- 최종 scoped diff에서 Goal 문서와 감사 artifact 외 production 변경이 이 Goal로 인해 생기지 않았는지 확인한다.
- `goal_completion_review_guide.md`를 read-only 감사 Goal에 맞게 적용해 policy restatement, runtime/live
  data/external-contract evidence, legacy/fallback audit, long-term fit과 명시적 verdict를 남긴다.

## Omitted Validation

- 전체 test suite는 기본 필수 조건이 아니다. 이 Goal은 production을 변경하지 않으며 모든 규칙을
  실행하는 비용이 크므로, claim 판정에 직접 필요한 focused tests/probes를 우선한다.
- 순수 Unity presentation, art와 animation은 gameplay meaning을 소비하지 않는 한 생략한다.
- 접근 불가능한 외부 repository는 검증하지 않고 `evidence_gap`으로 기록한다.
- 미래 후보로 명시된 기능은 구현 존재 여부를 일반적으로 검사하지 않는다. 단 현재 live path에
  우연히 활성화돼 룰북의 현재 규칙에 영향을 주는 정황이 있으면 조사한다.

생략 근거가 claim의 완료 판정을 약화하면 해당 claim을 `aligned`로 닫지 않는다.

## Completion Conditions

- 룰북의 모든 gameplay section과 normative claim이 stable claim ID로 inventory됐다.
- 모든 claim에 runtime/data/consumer 적용 여부와 alignment taxonomy가 기록됐다.
- 현재형 claim은 symbol 존재가 아니라 실제 production path와 live data 근거로 판정됐다.
- 승인·구현 대기, future/dormant와 현재 계약이 서로 구분됐다.
- mismatch catalog에는 불일치와 필수 evidence gap만 있고 aligned 항목은 포함되지 않았다.
- 각 mismatch는 룰북과 실제 구현 양쪽의 재현 가능한 경로·symbol/data identity를 가진다.
- 외부 consumer와 생략 검증은 확인 범위를 과장하지 않고 기록됐다.
- correction, migration, compatibility와 production/canonical 수정이 수행되지 않았다.
- 관련 없는 dirty worktree와 사용자 변경이 보존됐다.
- 독립 completeness pass와 completion review가 수행되고 명시적 verdict가 기록됐다.

## Completion Audit Evidence Matrix

| 완료 조건 또는 불변식 | 구현/조사 근거 | 검증 명령/결과 | 판정 |
| --- | --- | --- | --- |
| 전체 룰북 claim coverage | `RULEBOOK_CLAIM_MATRIX.md` | 58 contiguous ranges cover L1-L1210; 58 headings and 365 list items | pass |
| current/target/future 의미 분리 | matrix lifecycle/taxonomy와 catalog exclusions | hidden boss/exact authoring/approved-pending independent review | pass |
| production runtime·live data 추적 | 15 finding의 command-to-authority와 live identity | Core 1,063 tests passed; static entrypoint/data trace recorded | pass |
| producer/consumer 경계 확인 | Core DTO, server mapping/persistence, current Unity source | server 63 passed; Unity source/assertions inspected, runner limitation disclosed | pass |
| 불일치만 최종 catalog에 포함 | `MISMATCH_CATALOG.md` | 15 summary + 15 detailed; matrix↔catalog ID difference empty | pass |
| legacy/fallback 독립 감사 | `EXPERIMENTS.md`, `EXPERIMENT_NOTES.md` | research/pool/transport/ChooseSupport/sweep/persistence search completed | pass |
| 미검증 외부 경계 명시 | catalog evidence-gap 절과 `COMPLETION_AUDIT.md` | Unity runner unavailable; no fresh Unity pass claimed | pass |
| production/canonical 비수정 | final scoped status and rulebook hash | only Goal directory changed; rulebook hash unchanged | pass |
| 사용자 dirty 변경 보존 | initial/final commit, status and operation record | no reset/restore/stash/format/destructive command | pass |
| completion review verdict | `COMPLETION_AUDIT.md` | `Complete / High confidence` investigation verdict | pass |

## Stop Conditions

- 불일치 판정에 새 gameplay/UX/balance/persistence/compatibility 정책 결정이 필요한 경우
- 현재 working tree가 조사 중 변경되어 evidence generation을 특정할 수 없는 경우
- 필수 repository, live data, loader 또는 consumer에 접근할 수 없어 전체 판정이 불가능한 경우
- read-only 조사로 재현할 수 없고 production/fixture/test 수정이 필요한 경우
- 사용자 변경과 감사 artifact가 충돌해 안전하게 문서를 갱신할 수 없는 경우
- 조사 결과가 하나의 audit deliverable로 정리할 수 없는 독립 범위들로 분리되어 Master 재구조화가 필요한 경우

중단 시 안전한 read-only evidence는 보존하되 불일치나 일치를 추측하지 않고 실제 상황에 맞춰
`policy_decision_required`, `externally_blocked` 또는 `scope_decision_required`로 전환한다.

> 사용자와 의논하여 정해야 할 정책이 있을 경우 goal을 종료한다.
