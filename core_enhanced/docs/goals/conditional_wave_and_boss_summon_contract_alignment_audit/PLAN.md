# Conditional Wave and Boss Summon Contract Alignment Audit

## Goal Metadata

- Status: `complete`
- Goal mode: `audit_only`
- Implementation authorized: `No`
- Goal name: `conditional_wave_and_boss_summon_contract_alignment_audit`
- Created: 2026-08-17
- Audit finding ID: 없음
- Target repository root: `/mnt/f/work/simulator`
- Primary target paths: `core_enhanced`, 필요한 `game_resources` 및 `game_server` 경계
- Goal document path:
  `core_enhanced/docs/goals/conditional_wave_and_boss_summon_contract_alignment_audit`
- User approval basis: 2026-08-17 대화에서 승인된 단일 read-only 정합성 감사 Goal

`complete`는 이 Goal의 감사 완료를 뜻한다. 구현 완료, 정책 변경 승인 또는 production 변경 승인을 뜻하지
않는다.

## 목표와 사용자-visible 문제

조건부 Wave·보스 소환 정책에 관해 승인된 정책, durable canonical 문서, production live data와 실제 runtime이
같은 사용자-visible 행동을 가리키는지 확인한다.

현재 문제는 다음과 같다.

- 임시 정책 노트의 세부사항이 Final Policy Freeze와 `game_rulebook.md`에 모두 반영됐는지 아직 현재 source
  기준으로 재감사하지 않았다.
- 구현 완료 이후 runtime과 live RON이 정본 문서와 같은 의미를 유지하는지 확인할 fresh evidence가 필요하다.
- runtime에만 존재하는 암묵적 의미, 문서에만 존재하는 미구현 의미 또는 아직 정책으로 닫히지 않은 확장
  주제가 남아 있을 수 있다.

## 감사 결과의 단일 계약 경계

이 Goal의 산출물은 하나의 `조건부 Wave·보스 소환 정책 정합성 감사`다. 여러 runtime 책임을 조사할 수 있지만
production 수정은 수행하지 않는다. 발견된 서로 독립적인 수정 사항은 이 Goal에 합치거나 자동 구현하지 않고
후속 후보로만 분리한다.

## 승인된 정책과 근거

정책 의미를 판정할 때 다음 우선순위를 사용한다.

1. 승인된 정책 결정: 기존 current Final Policy Freeze
   `docs/goals/conditional_wave_and_boss_summon_policy/P2G_30_FINAL_POLICY_FREEZE_002.ko.md`
2. durable gameplay 표현: `docs/game_rulebook.md`
3. runtime·data 책임에 가까운 canonical 계약:
   `docs/core_runtime_contract.md`, `docs/data_loading_contract.md`
4. 현재 실제 동작: reachable runtime code, production loader가 읽는 live RON, 필요한 server projection
5. 조사 후보와 역사적 배경:
   `docs/tmp/conditional_wave_and_boss_summon_policy_notes_2026-08-13.ko.md`

임시 정책 노트와 과거 Goal·실험 기록은 그 자체로 현재 정책 또는 실제 동작의 source of truth가 아니다.
Final Policy Freeze의 current성 및 정확한 member 범위는 감사 시작 시 확인한다. Freeze가 current가 아니거나
서로 모순되는 승인 결정이 발견되면 임의로 우선순위를 정하지 않는다.

## 선택적 읽기 경계

불필요한 문서 완독과 repository-wide 탐색을 피한다.

- Final Policy Freeze에서는 current member와 각 승인 statement·acceptance를 판정하는 데 필요한 payload를
  읽는다.
- 임시 정책 노트는 먼저 heading과 section 범위를 inventory하고, 각 감사 row에 직접 필요한 section만 읽는다.
- `game_rulebook.md`, runtime/data canonical 문서와 live RON은 해당 policy element와 직접 연결된 section 및
  경로만 읽는다.
- 과거 P2G·execution history 전체는 읽지 않는다. Freeze 의미, current성 또는 특정 구현 판단에 직접 필요한
  exact artifact만 읽는다.
- runtime은 symbol 존재가 아니라 relevant entrypoint에서 authoritative state mutation 또는 observable result까지
  추적한다. Goal과 관계없는 Unity 및 다른 gameplay 영역은 탐색하지 않는다.

필요한 근거가 다른 section에 있다는 구체적인 이유가 생긴 경우에만 읽기 범위를 확장하고 그 이유를
`EXPERIMENT_NOTES.md`에 기록한다.

## 현재 runtime 및 live data 근거

- Runtime actual: ASB-002에서 `BattleCore`의 Wave Runtime → Spawn Kernel, Boss Runtime → Summon Runtime,
  terminal outcome, attempt-local compendium과 active battle pause/reconnect 경로를 authoritative mutation까지
  추적했다. 상세 row는 `EXPERIMENT_NOTES.md`의 Alignment Matrix를 따른다.
- Production live RON selection과 사용 여부: `live_ron.rs`가 `data/boss_runtime/base.ron`과
  `data/summoned_units/base.ron`을 read하고, `reference_cwbs_final_boss` encounter가 두 catalog에 실제로
  opt-in한다. `successful_live_publication_compiles_boss_runtime_exactly_once` focused test가 통과했다.
- Server projection 관련성: same-process disconnect는 PlayerGameActor가 Core active battle을 자동 Pause하고,
  reconnect는 existing battle setup/update projection을 다시 보낸다. Unity 소비자는 승인된 비범위다.
- 접근 가능한 소비자: Core World와 game_server mapping/test seam. Unity reader/store/command/presentation은
  접근하지 않았으며 검증했다고 표현하지 않는다.
- 기존 focused/integration test가 보호하는 계약: Wave Runtime 10개, Spawn Kernel 10개, Summon Runtime 10개,
  live RON publication 1개를 ASB-002 current source에서 실행했다. game_server reconnect test는 Rust compiler
  ICE로 test binary 실행 전 중단되어 source/test seam만 기록한다.

코드·data·test의 존재만으로 reachable current behavior를 주장하지 않는다.

## Source of Truth와 소유 계층

| 의미 | 판정 source 또는 owner | 초기 상태 |
| --- | --- | --- |
| 승인된 목표 정책 | current Final Policy Freeze | current성 확인 대기 |
| durable gameplay 의미 | `docs/game_rulebook.md` | 관련 section 조사 대기 |
| Core runtime 불변식 | `docs/core_runtime_contract.md`와 실제 authoritative mutation owner | 조사 대기 |
| live RON schema/publication | `docs/data_loading_contract.md`, production loader와 validator | 조사 대기 |
| 현재 실제 행동 | reachable Core runtime 및 live RON | 조사 대기 |
| server-visible projection | 실제 producer, mapping과 접근 가능한 consumer | 관련성 조사 대기 |
| 임시 설계 후보 | 임시 정책 노트 | 비정본 입력 |

## 유지할 불변식

- 임시 정책 노트를 승인된 정책으로 승격하지 않는다.
- `game_rulebook.md`와 runtime 중 어느 하나를 자동으로 정답으로 간주하지 않는다.
- current actual, canonical 계약과 승인된 목표 정책을 구분한다.
- test/type/schema 존재를 runtime reachability의 대체 근거로 사용하지 않는다.
- dirty working tree의 기존 변경을 수정, 정리, restore, stash 또는 덮어쓰지 않는다.
- 감사 finding을 이 Goal 안에서 production 수정으로 전환하지 않는다.
- 사용자-visible 의미나 외부 계약이 미결정이면 작업자가 대신 결정하지 않는다.
- 접근하지 않은 Unity 또는 외부 소비자를 검증했다고 표현하지 않는다.

## 범위

### 포함

- Final Policy Freeze member와 임시 정책 노트 주제의 완전한 감사 disposition
- 조건부 Wave activation, dependency, ordering, required-for-victory 및 cancellation 의미
- 공용 Spawn Kernel의 identity, atomicity, dedupe, placement와 event ordering 의미
- 보스 소환 definition, counter, lifecycle, batch, behavior, placement, lifetime/removal 의미
- 전투 terminal outcome, 같은 시각 정산 및 관련 승리 보류 의미
- 해당 정책에 직접 연결된 compendium/save/restart/reference integration 의미
- 관련 `game_rulebook.md`, Core runtime, production live RON와 필요한 server projection의 current-state 비교
- 기존 test seam의 계약 보호 범위 확인과, 정적 근거가 불충분할 때의 최소 read-only probe
- 정책 확장 후보와 문서·runtime·live-data drift의 구분

### 명시적 비범위

- production code, fixture, mock 또는 test 수정
- `game_rulebook.md`와 그 밖의 canonical 문서 수정
- live RON, schema, DTO, save format 또는 server mapping 수정
- 새로운 gameplay 정책 결정
- 실제 production 콘텐츠 또는 밸런스 설계
- Unity repository 조사·수정
- 후속 Goal, Master Goal 또는 P2G artifact 자동 생성
- 발견된 결함의 구현 또는 migration
- 전체 test suite를 baseline 명목으로 실행하는 작업
- 관련 없는 정리, 리팩토링, formatting 및 dirty change 정돈

## 미결정 정책

초기 승인 범위에서 알려진 미결정 정책은 없다. 조사 결과 다음 중 하나가 필요해지면 해당 row를
`policy_undefined`로 분류하고 Status를 `policy_decision_required`로 변경한다.

- 승인된 근거 없이 사용자-visible 의미를 선택해야 함
- 서로 충돌하는 승인 결정 중 하나를 선택해야 함
- runtime의 암묵 동작을 정본화할지 제거할지 결정해야 함
- compatibility, migration, persistence lifetime, identity 또는 reward 의미를 새로 확정해야 함

## 구현 범위

없음. 이 Goal은 조사와 검증 기록만 작성한다. Production 또는 canonical 문서 변경이 필요하면 finding으로
기록하고 별도 사용자 승인과 별도 Goal을 요구한다.

## 사용자-visible 완료 조건

다음을 모두 충족해야 한다.

1. current Final Policy Freeze의 모든 in-scope member가 정확히 하나 이상의 감사 row에 연결된다.
2. 임시 정책 노트의 모든 top-level policy section이 `Freeze에 포함`, `역사적/대체됨`, `확장 후보`,
   `명시적 비범위` 중 하나로 disposition된다.
3. 각 감사 row에 가능한 범위에서 다음 근거가 연결된다.
   - 임시 정책 노트 section 또는 `N/A` 근거
   - Final Policy Freeze member
   - `game_rulebook.md` exact section 또는 부재
   - 실제 runtime entrypoint와 authoritative mutation/observable result
   - production live RON과 loader 경로 또는 `N/A` 근거
   - server/consumer 경계 또는 `N/A` 근거
   - existing test 또는 read-only validation seam
4. 각 row가 다음 중 정확히 하나의 verdict를 갖는다.
   - `aligned`
   - `canonical_document_gap`
   - `runtime_implementation_gap`
   - `live_data_gap`
   - `policy_undefined`
   - `evidence_unavailable`
5. 문서 누락, runtime 결함, data 결함, 진짜 정책 확장 주제가 서로 구분된다.
6. 모든 non-aligned row에 사용자-visible 영향, 직접 영향 경계, 권장 후속 조치와 별도 Goal 분할 기준이
   기록된다.
7. 접근하지 못한 source·consumer와 남은 위험이 명시된다.
8. production 및 canonical 파일을 수정하지 않았고 기존 dirty 변경이 보존됐음을 확인한다.

## Alignment Matrix 형식

감사 중 `EXPERIMENT_NOTES.md`에 다음 schema의 row를 추가한다.

| Row ID | Policy element / observable behavior | Notes ref | Freeze ref | Rulebook ref | Runtime entrypoint → mutation/result | Live RON/loader | Consumer/test seam | Verdict | Impact / follow-up |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| `discovery pending` | `discovery pending` | `discovery pending` | `discovery pending` | `discovery pending` | `discovery pending` | `discovery pending` | `discovery pending` | `discovery pending` | `discovery pending` |

하나의 happy-path test나 broad suite 이름만으로 실패·취소·ordering·lifecycle 같은 별도 observable behavior를
일괄 `aligned`로 판정하지 않는다.

## Migration 여부

- DTO migration: 없음 / 감사 결과에서 필요성이 발견되면 finding으로만 기록
- data 또는 live RON migration: 없음 / 감사 결과에서 필요성이 발견되면 finding으로만 기록
- save migration: 없음 / 감사 결과에서 필요성이 발견되면 finding으로만 기록
- compatibility layer: 추가·변경하지 않음

## 필수 focused 검증

- Freeze member coverage와 임시 정책 노트 section disposition의 누락·중복 점검
- 각 row의 rulebook clause와 runtime entrypoint→mutation/result 연결 확인
- applicable row의 production live RON→loader→runtime reachability 확인
- applicable producer→server mapping→접근 가능한 consumer 경계 확인
- 정적 근거가 모호한 row에 한정한 기존 focused test 결과 확인 또는 최소 read-only probe
- 모든 verdict가 인용된 근거와 일치하는지 최종 재검토
- Goal 문서 외 production/canonical 파일을 수정하지 않았다는 scoped status 확인

검증 명령은 실제 조사 후 확정하고 `EXPERIMENTS.md`에 정확히 기록한다. Test runner가 수치를 보고하지 않으면
추측하지 않는다.

## 필수 broad 검증

전체 test suite는 필수가 아니다. 이 Goal은 production 변경을 하지 않으므로 broad regression suite가
감사 완료를 직접 증명하지 않는다. 여러 row의 runtime actual을 하나의 existing integration test가 직접
증명하며 실행 비용이 합리적인 경우에만 필요한 범위를 선택해 실행할 수 있다.

## 생략할 검증과 근거

- Core/game_server 전수 test: production mutation이 없고 계약 row별 actual 판정이 목적이므로 기본 생략
- Unity 검증: 사용자 승인 비범위이며 접근하지 않은 consumer로 기록
- save migration test: migration을 수행하지 않음
- schema migration/compatibility test: schema나 compatibility를 변경하지 않음
- production 콘텐츠 양적 검사: 실제 콘텐츠 설계는 비범위

정적 근거만으로 필수 row의 actual을 판정할 수 없다면 해당 focused test/probe를 생략하지 않는다. 실행할 수
없으면 `evidence_unavailable`과 남은 위험을 기록한다.

## Baseline 정책

Production 파일을 수정하지 않으므로 production test baseline은 실행하지 않는다. 대신 감사 시작 시 exact
dirty working tree 상태와 감사 대상 파일의 기존 diff를 read-only로 고정한다.

이 Goal의 범위가 추후 production 수정으로 바뀌지 않는다. 수정이 필요하면 별도 Goal이 자신의 변경 전
baseline을 새로 확보해야 한다.

## 중단 조건

- 여러 승인 정책 근거가 모순되어 expected behavior를 선택해야 함
- 사용자-visible 의미 또는 외부 계약을 새로 결정해야 함
- 감사 자체가 여러 독립된 deliverable로 분리돼야 하며 현재 matrix로 안전하게 완결할 수 없음
- 필수 source, live data 또는 consumer에 접근할 수 없어 핵심 verdict를 판정할 수 없음
- 감사 대상 path의 dirty 변경이 조사 중 바뀌어 recorded source basis를 신뢰할 수 없음
- Goal 문서 외 파일을 수정해야만 다음 단계로 진행할 수 있음

정책 결정이 필요하면 `policy_decision_required`, 범위 선택이 필요하면 `scope_decision_required`, 필수 외부
근거가 없으면 `externally_blocked`로 변경한다. 단순히 조사가 어렵거나 오래 걸린다는 이유로 중단하지 않는다.

## 초기 Dirty Worktree

2026-08-17 신규 Goal 경로 생성 전에 다음을 실행했다.

```bash
git -C core_enhanced rev-parse --show-toplevel
git -C core_enhanced status --short
```

- 확인된 repository root: `/mnt/f/work/simulator`
- Goal 경로 충돌: 없음
- 상태: 다수의 modified/deleted/untracked 사용자 변경이 존재하는 대규모 dirty working tree
- 직접 겹치는 것으로 보이는 경로: `core_enhanced/docs/game_rulebook.md`, Core battle/runtime/data 파일,
  `game_resources/data`, `game_server` 및 기존 정책 Goal artifacts
- 초기 확인은 status만 수행했으며 기존 diff의 의미는 아직 조사하지 않았다.
- 이 Goal은 read-only audit이므로 겹치는 기존 변경을 수정하지 않는다. 각 claim은 조사 당시 exact dirty
  source basis를 함께 기록한다.

## 실행 순서

1. 필수 Goal 문서 세 개와 위 승인 basis의 current성·존재를 확인한다.
2. 감사 대상 경로의 current status/diff를 read-only로 고정한다.
3. Freeze member와 임시 정책 노트 section inventory를 만든다.
4. 각 observable behavior에서 rulebook, runtime mutation, live data와 consumer/test seam을 필요한 범위만
   추적한다.
5. Alignment Matrix를 채우고 finding을 분류한다.
6. coverage, verdict evidence와 dirty preservation을 검증한다.
7. 모든 완료 조건에 증거를 연결한 경우에만 Status를 `complete`로 변경한다.

## 감사 완료 판정 — 2026-08-20

- `FPF-CWBS-P2G30-002`의 25개 current member 모두가 Alignment Matrix의 정확히 한 row에 연결됐다.
- 임시 정책 노트의 25개 top-level policy section은 모두 same-ID Freeze member로 disposition된 기존 기록을
  ASB-002에서도 유지했다.
- current runtime, live RON/loader, server seam과 focused test evidence를 row별로 연결했다.
- runtime implementation gap 또는 live-data gap은 발견하지 않았다. `CWBS-AUD-001`~`003`은 current runtime과
  Freeze에 맞춰 canonical wording을 보강해야 하는 문서 gap이다.
- Unity와 배포 외부 consumer는 비범위/미확인으로 기록했다. game_server reconnect test는 Rust compiler ICE로
  pass 증거가 없으나, 이 Goal의 row verdict는 static source trace와 Core focused test 범위로 제한했다.
- production/canonical 파일은 수정하지 않았으며, source hash와 scoped dirty worktree 보존은 E-007에서
  재확인했다.

이 `complete`는 read-only alignment audit 완료만 의미한다. `CWBS-AUD-001`~`003`을 실제 canonical 문서에
반영하는 작업은 별도 correction Goal과 사용자 승인 범위가 필요하다.
