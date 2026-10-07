# Game Rulebook Full Runtime Alignment Audit

## Goal Metadata

- Status: `complete`
- Goal mode: `audit_only`
- Implementation authorized: `No`
- Goal name: `game_rulebook_full_runtime_alignment_audit`
- Created: 2026-08-20
- Completed: 2026-08-20
- Audit finding ID: `GRFRA-001`~`GRFRA-010`
- Target repository root: `/mnt/f/work/simulator`
- Primary target paths: `core_enhanced`, production `game_resources`, 필요한 `game_server` 경계
- Goal document path:
  `core_enhanced/docs/goals/game_rulebook_full_runtime_alignment_audit`
- User approval basis: 2026-08-20 대화에서 승인된 전체 룰북 read-only 정합성 감사 Goal

`complete`는 승인된 전체 룰북 감사와 기록·검증이 끝났음을 뜻한다. 구현, 정책 변경, canonical 수정 또는
production 변경은 수행하지 않았다.

## 목표와 사용자-visible 문제

`docs/game_rulebook.md` 전체의 현재 적용 gameplay 규칙이 Core runtime, production live RON 및 필요한
`game_server` 경계의 실제 동작과 같은 의미를 가리키는지 확인한다.

감사는 룰북 전체에 대해 current working tree의 실제 producer, state mutation, consumer 및 live data를 연결한
fresh evidence를 만들고, 그 결과 드러난 불일치를 구현과 섞지 않은 별도 보고서로 제공한다. 과거 감사는 조사
단서일 수 있으나 current actual의 증거로 재사용하지 않는다.

## 감사 finding ID

현재 `GRFRA-001`~`GRFRA-010`을 확인했다. 추가 불일치는 발견 즉시 다음 stable ID를 부여하고
`EXPERIMENT_NOTES.md`에 기록한다.

## 현재 runtime 및 live data 근거

- Audit Source Basis `GRFRA-ASB-001`은 repository revision
  `f318414fb0156a67e04c5479000c2a2683fc0c09` 위의 current dirty working tree다.
- 룰북, runtime/data canonical, Core battle/world/data, production `game_resources`와 `game_server`가 모두 기존
  modified/deleted/untracked 변경을 포함한다.
- 50개 coverage group 모두에서 applicable Core producer/mutation/result, live RON/loader 및 필요한 server 경계를
  current source로 추적했다. 10개 finding은 `EXPERIMENT_NOTES.md`에 기대·실제·영향·근거와 함께 기록했다.
- 룰북 heading inventory는 44개 `###`, 4개 `####` subsection을 확인했고, 문서 책임 및 후속 authoring 경계를
  포함한 50개 coverage group `GR-001`~`GR-050`으로 고정했다.
- Focused/live-RON/broad 검증은 성공한 9개 job에서 1,213 tests가 모두 통과했다. 별도의 최초 server incremental
  compile 1건은 test 실행 전 rustc ICE였고, 동일 filter와 server broad suite가 `CARGO_INCREMENTAL=0`에서
  통과했다.

## Source of Truth와 소유 계층

| 의미 | 판정 source 또는 owner | 현재 상태 |
| --- | --- | --- |
| 현재 gameplay 의미 | `docs/game_rulebook.md`의 applicable canonical 규칙 | 50개 coverage group으로 전체 disposition 완료 |
| 세부 targeting/runtime/data 계약 | 룰북이 명시한 인접 durable canonical | applicable claim의 owner 경계 확인 완료 |
| 실제 Core 동작 | reachable Core producer → mutation → result | 전 group trace 및 Core lib 1,080-test broad 검증 완료 |
| production live data | production loader가 실제 선택하는 live RON | loader/reachability trace 및 25-test RON suite 완료 |
| server-visible 경계 | 필요한 `game_server` producer/projection/test seam | source trace, focused 12 tests와 broad 73 tests 완료 |
| Unity 표현·소비 | 외부 Unity canonical 및 repository | 명시적 비범위 |
| 과거 감사·Goal 기록 | 조사 후보와 history | current actual 증거로 사용하지 않음 |

룰북이 미래 후보, 명시적 미구현, dormant 또는 인접 canonical 소유로 선언한 항목은 현재 runtime mismatch와
구분한다. Runtime이 룰북과 다르면 자동으로 어느 한쪽을 수정하지 않고 finding으로만 기록한다.

## 유지할 불변식

- 룰북 전체 범위를 임의로 축소하지 않는다.
- 검색 hit, type, test 또는 과거 감사만으로 runtime reachability를 주장하지 않는다.
- 실제 producer에서 authoritative mutation과 observable consumer/result까지 비례적으로 추적한다.
- 현재 적용 규칙, 미래 후보, 명시적 미구현, 인접 canonical 소유 및 Unity-only 항목을 구분한다.
- 의미 있는 조사·실험·판단·불일치는 발생 직후 관련 Goal 문서에 기록한다.
- 실행하지 않은 명령이나 확인하지 않은 사실을 결과처럼 기록하지 않는다.
- production code/data/test, canonical 문서와 기존 dirty work를 수정·정리·복원하지 않는다.
- 이 Goal의 문서는 실행 기록이며 gameplay/data/save/wire 계약의 source of truth가 아니다.

## 범위

### 포함

- `docs/game_rulebook.md` 전체 section과 현재 검증 가능한 normative claim
- Core runtime의 gameplay producer, state transition, persistence와 result commit
- production live RON, 실제 loader·validator·compiled publication과 runtime reachability
- 룰북 claim에 직접 필요한 `game_server` command/projection/persistence 경계
- 관련 current canonical을 이용한 책임 경계 해석
- 과거 감사 finding의 current actual 재검증
- 필요한 기존 focused test 또는 audit-local read-only probe
- 불일치, 검증 제외, evidence gap과 남은 위험의 즉시 기록

### 명시적 비범위

- 외부 Unity repository와 Unity reader/store/presentation 검증
- production code, live RON, fixture, mock 또는 test 수정
- `game_rulebook.md` 및 다른 canonical 문서 수정
- 새로운 gameplay·balance·UX·compatibility 정책 결정
- 배포, release, production 환경 변경
- finding correction 구현이나 후속 Goal 자동 생성
- 관련 없는 cleanup, refactor, formatting 또는 dirty change 정리

## 승인된 정책과 근거

- 감사 대상 gameplay canonical: current `core_enhanced/docs/game_rulebook.md`
- 룰북이 위임한 세부 경계: current `skill_target_contract.md`, `core_runtime_contract.md`,
  `data_loading_contract.md` 및 필요한 server canonical
- 감사 범위·비범위·기록 방식: 2026-08-20 사용자 승인 대화

별도 Final Policy Freeze나 P2G/GEX implementation authorization은 이 audit-only Goal의 source가 아니다.
Material policy 충돌이 발견되면 작업자가 의미를 선택하지 않고 정책 질문으로 기록한다.

## 미결정 정책

후속 correction Goal에서 다음 정책을 결정해야 한다. 이 audit-only Goal은 어느 선택도 승인하지 않는다.

- `GRFRA-001`~`003`: current 결정성 알고리즘을 gameplay canonical에 고정할지, 다른 owner에 위임할지
- `GRFRA-004`: status 차단에도 공식 `immune` feedback을 생산할지, canonical 의미를 damage-only로 줄일지
- `GRFRA-005`: active DTO field를 rename/migrate할지, 제거 선언을 legacy semantics로 한정할지
- `GRFRA-006`: Bonus objective를 current runtime에 구현할지, future authoring으로 재분류할지
- `GRFRA-007`: Breakdown collapse counter와 live 값 `2`를 구현할지, future implementation으로 재분류할지
- `GRFRA-008`: HeadquartersContact의 실제 후보를 NoDeployable recovery로 인정하는 exact eligibility
- `GRFRA-009`: invalid V4 scalar를 strict reject할지와 기존 save compatibility/migration 범위
- `GRFRA-010`: BossOmen Event source만 첫 패배에서 실패시키는 exact retry 경계

## 구현 범위

없음. 이 Goal은 조사, read-only 검증과 세 Goal 문서 기록만 수행한다.

## 사용자-visible 완료 조건

1. 룰북의 모든 top-level/subsection이 coverage inventory에 정확히 disposition된다.
2. 각 current normative rule group이 가능한 범위에서 룰북 ref, Core producer→mutation/result, applicable live
   RON/loader, server/consumer 및 validation evidence에 연결된다.
3. 미래 후보, 명시적 미구현, dormant, 인접 canonical 소유와 Unity-only 항목이 current mismatch와 구분된다.
4. 각 검증 가능 rule group은 `aligned`, `runtime_implementation_gap`, `live_data_gap`,
   `server_integration_gap`, `canonical_ambiguity`, `evidence_unavailable` 중 하나로 판정된다.
5. 모든 non-aligned rule에 stable finding ID, 기대·실제 동작, 직접 근거, 영향, 확신도와 권장 후속 조치가 있다.
6. 실제 실행한 focused validation과 생략한 검증의 근거가 정확히 기록된다.
7. 접근하지 못한 consumer, external boundary, baseline limitation과 남은 위험이 명시된다.
8. 세 Goal 문서 외 production/canonical 파일을 수정하지 않았고 기존 dirty work가 보존됐음을 확인한다.

## Migration 여부

- DTO migration: 수행하지 않음. `GRFRA-005`의 field rename을 선택하면 별도 compatibility/migration 판단이 필요함
- data 또는 live RON migration: 수행하지 않음. `GRFRA-006` 구현 시 encounter authoring,
  `GRFRA-007` 구현 시 run policy authoring이 필요할 수 있음
- save migration: 수행하지 않음. `GRFRA-007`의 새 persistent counter와 `GRFRA-009` strict rejection 정책을
  구현할 때 기존 V4 save 처리 방식을 별도 결정해야 함

## 필수 focused 검증

- 룰북 heading 및 normative rule coverage 누락·중복 점검
- rule group별 Core entrypoint→authoritative mutation/result 추적
- applicable production live RON→loader/validator→runtime reachability 확인
- applicable Core producer→game_server projection/command/persistence 경계 확인
- 정적 근거가 불충분한 high-impact rule의 기존 focused test 또는 최소 read-only probe
- finding evidence와 현재 source identity의 최종 재검토
- Goal 문서 외 production/canonical 파일을 수정하지 않았다는 scoped status 확인

## 필수 broad 검증

Production mutation이 없지만 룰북 전 범위의 current actual을 폭넓게 확인하기 위해 `game_core --lib` 1,080 tests,
`game_server --lib` 73 tests 및 production RON integration 25 tests를 실행했다. 룰북 전체 coverage 자체는 broad
test 이름이 아니라 50개 claim inventory와 source trace로 검증한다.

## 생략할 검증과 근거

- Unity 검증: 사용자 승인 비범위
- 배포 환경·외부 consumer 검증: production rollout 비범위
- migration test: schema/save/DTO를 변경하지 않음
- 전체 workspace/repository test suite: production mutation이 없고 Core/server library와 production RON integration을
  포함한 applicable suite를 실행했으며 unrelated package까지 확장하지 않음
- 최종 밸런스·콘텐츠 품질 판정: bootstrap과 future authoring을 룰북이 별도로 구분함

정적 근거가 부족한 applicable claim의 focused validation까지 자동으로 생략하지 않는다. 실행 불가면
`evidence_unavailable`로 기록한다.

## 중단 조건

- current canonical끼리 충돌해 expected behavior를 선택해야 함
- 사용자-visible 의미나 compatibility/migration 정책을 새로 결정해야 함
- 필수 source/live data/server evidence에 접근할 수 없어 핵심 범위 전체를 판정할 수 없음
- 감사 대상 dirty source가 조사 중 바뀌어 기록한 basis를 신뢰할 수 없음
- 세 Goal 문서 외 파일을 수정해야만 감사를 계속할 수 있음

국소 evidence gap은 전체 감사를 즉시 중단하지 않고 해당 row와 남은 위험에 기록한다.

## Compatibility

Compatibility 추가·변경은 승인되지 않았다. 현재 승인된 compatibility가 claim에 존재하면 그 exact canonical
근거, 지원 범위와 제거 조건만 감사한다. 새로운 shim, fallback 또는 migration을 제안 이상으로 구현하지 않는다.

## 초기 Dirty Worktree

2026-08-20 신규 Goal 경로 생성 직후 read-only 기준선을 고정했다.

- Repository root: `/mnt/f/work/simulator`
- Revision: `f318414fb0156a67e04c5479000c2a2683fc0c09`
- Source disposition: clean HEAD가 아닌 current dirty working-tree actual
- `game_rulebook.md`, 세 인접 canonical, Core의 광범위한 battle/world/data 경로, production live RON과
  `game_server`에 기존 modified/deleted/untracked 변경이 있다.
- 새 Goal 경로만 이번 감사가 추가한 untracked 경로다.
- 기존 diff의 작성자나 완결성을 추정하지 않고 각 claim은 조사 시점 current content로 판정한다.
- 대상 source가 조사 중 변하면 영향받은 observation과 verdict를 재검증한다.

## 실행 순서

1. repository revision, scoped dirty baseline과 source identity를 고정한다.
2. 룰북 전체 heading과 rule group inventory를 작성한다.
3. section별 actual producer/data/server 경계를 추적하고 조사 직후 문서를 갱신한다.
4. 필요한 focused validation을 실행하고 즉시 결과를 기록한다.
5. coverage, finding evidence, source currentness와 dirty preservation을 최종 확인한다.
6. 모든 완료 조건에 direct evidence가 연결된 경우에만 Status를 `complete`로 바꾼다.
