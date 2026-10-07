# DefenseRoute 전술 지휘 Phase 0 — 정책 기준선과 Source Of Truth 분리

Status: complete

Implementation Complete: true

Goal Type: phase

Execution Mode: implement_and_verify

Production Mutation Authorized: yes — 이 Phase의 승인 범위에 한정

Parent Master: [defense_route_tactical_command_master](../defense_route_tactical_command_master/PLAN.md)

Phase: 0

Goal Name: `defense_route_tactical_command_phase_00_policy_baseline`

Target Repository: `/mnt/f/work/simulator`

Start Commit: `f318414fb0156a67e04c5479000c2a2683fc0c09`

Audit Finding ID: 없음

## 목표

DefenseRoute 전술 지휘 정책 `DR-01`~`DR-17`의 승인된 목표 의미를 역사적 조사·초안과 분리한
간결한 decision record로 고정하고, 이후 Phase가 현재 runtime을 미래 정책으로 오해하지 않도록
현재 code/data/server/canonical 기준선을 남긴다. Phase 0이 직접 소유하는 `DR-01` 전투 목표와
`DR-15` Pause 중 명령 의미가 현재 구현과 일치하는지 focused evidence로 확인하고 필요한 범위만
교정한다.

## 해결할 사용자-visible 또는 contract-visible 문제

현재 임시 조사 문서에는 시점 runtime 조사, 초기 권고안과 확정 정책이 함께 있어 이후 구현자가
과거 초안이나 당시 관찰을 승인된 목표 정책으로 잘못 사용할 수 있다. 또한 Phase별 구현 전
`DR-01`, `DR-15`의 현재 일치 여부와 공통 version/canonical ownership을 증명한 기준선이 없다.

## 승인된 정책과 근거

- 정책 ID: `DR-01`, `DR-15`
- 전체 traceability 대상: `DR-01`~`DR-17`
- 승인된 목표 정책:
  [DefenseRoute 전술 지휘 목표 정책 결정 — 2026-08-06](../../decisions/defense_route_tactical_command_policy_2026-08-06.md)
- Historical 조사 근거:
  [DefenseRoute 전술 지휘 확장 임시 조사·설계 노트](../../tmp/defense_route_tactical_command_design_notes_2026-08-04.ko.md)
- 실행 범위와 Phase coverage:
  [Master PLAN](../defense_route_tactical_command_master/PLAN.md)

과거 초안, 초기 권고안과 조사 당시 runtime 설명은 decision record를 대체하지 않는다.

## 현재 runtime 및 live data 근거

현재 tree 대조 결과:

- `DR-01`: `WinCondition`과 `BattleCore::compute_winner`가 required enemy group, protected unit과
  `ProtectUnitUntil`/`SurviveUntil`을 전술 command 사용 여부와 독립적으로 판정한다. Live rulebook도
  같은 `DefenseRoute`/survival 의미를 설명한다.
- `DR-15`: `BattlePlaybackState`와 `simulation_delta_for_tick`이 paused production tick을 차단한다.
  Pause/Resume/SetSpeed 및 배치·철수·skill handler는 같은 battle authority/revision transaction을
  사용하고 game_server actor가 요청을 직렬 처리한다.
- 기존 focused coverage는 clock freeze, replay authority, setup recovery와 stale error→resync를
  보호했지만 Pause 중 deploy/withdraw/manual skill commit과 동일 revision 연속 request를 직접 보호하지
  않았다. 이 Phase에서 회귀 테스트를 추가했다.
- Future Phase state는 아직 구현되지 않았다. 현재 배치 자원은 `LiveBattleDeploymentState`와
  `live_deployment` RON에 결합되어 있고, weapon tactical profile/stance/relocation/swap/selection/
  FieldWork symbol은 production source와 live RON에 없다.

## Source Of Truth와 소유 계층

- 현재 실제 동작: Core/game_server runtime code와 production live data
- 현재 canonical: 실제 구현된 현재 계약만 설명하는 관련 durable 문서
- 승인된 목표 정책: 이 Phase가 생성할 target decision record
- 시점 조사 evidence: 기존 2026-08-04 임시 조사 문서
- 실행 evidence: 이 Phase와 Master Goal 문서

Target decision record는
`core_enhanced/docs/decisions/defense_route_tactical_command_policy_2026-08-06.md`가 소유한다.
현재 canonical별 ownership과 version baseline은 `EXPERIMENT_NOTES.md`에 기록했다. Decision record는
미구현 미래 정책을 현재 runtime 계약처럼 표현하지 않는다.

## 유지할 불변식

- Core가 전투 상태와 명령 수락 여부의 authority다.
- Pause는 전투 simulation clock을 진행시키지 않는다.
- Pause 중에는 승인된 즉시 명령만 허용하고, 명령별 허용 여부를 client/server가 재추론하지 않는다.
- 동일 입력, live data, seed와 명령 순서는 동일한 state/event 결과를 만든다.
- 승인되지 않은 fallback, alias, dual-read/write 또는 compatibility를 추가하지 않는다.
- 이후 Phase의 enum, state, DTO, data field 또는 placeholder를 미리 추가하지 않는다.
- 구현되지 않은 목표 정책을 현재 canonical에 이미 동작하는 것처럼 기록하지 않는다.
- 기존 사용자 dirty 변경과 범위 밖 파일을 보존한다.

## 범위

- `DR-01`~`DR-17` 확정 정책의 간결한 target decision record 생성
- 정책 ID가 정확히 한 번씩 decision record와 Master Phase coverage에 연결되는지 검증
- 기존 임시 조사 문서를 시점 evidence로 동결하고 superseding decision record 링크 추가
- 현재 Core/live data/game_server/canonical의 Phase별 baseline과 version ownership 기록
- `DR-01`, `DR-15`의 현재 runtime·server·canonical 일치 여부 확인
- 필요한 경우 `DR-01`, `DR-15`에만 한정된 correction과 focused/broad 검증
- Phase 0 완료 감사와 Master Phase registry 갱신

## 명시적 비범위

- Phase 1~9 정책의 production 구현 또는 canonical 현재 계약 선행 갱신
- Unity 문서, runtime, contract, consumer와 tests
- 확정 목록 밖 gameplay, 콘텐츠 또는 tuning 변경
- compatibility, schema alias, dual-read/write와 fallback
- 별도 Goal이나 후속 campaign 자동 생성

## 미결정 정책

현재 알려진 미결정 사용자-visible 정책은 없다. Decision record 분리 중 확정 절 사이의 충돌이나
DTO/save/compatibility 선택이 발견되면 영향받는 변경을 중단하고 `policy_decision_required`로
전환한다.

## 구현 범위

1. 임시 문서의 번호별 확정 정책과 승인 tuning을 추출하고 역사적 초안과 구분한다.
2. 간결한 target decision record와 `DR-01`~`DR-17` traceability 표를 작성한다.
3. 임시 문서에 동결 시점, historical evidence 책임과 superseding 링크를 기록한다.
4. Phase별 runtime/data/server/canonical/version baseline을 현재 tree에서 확인해 기록한다.
5. `DR-01`, `DR-15` command/state/time 경로와 관련 tests를 focused 검증한다.
6. 불일치가 있으면 Phase 0 승인 범위 안에서만 교정하고 재검증한다.
7. 완료 검토 가이드에 따라 evidence matrix, legacy/fallback, dirty 보존과 long-term fit을 감사한다.

## 완료 조건

- 확정 정책만 소유하는 target decision record가 생성되고 `DR-01`~`DR-17`이 누락·중복 없이 연결된다.
- 임시 조사 문서가 historical evidence로 동결되고 target decision record를 명시적으로 링크한다.
- 현재 동작, 현재 canonical과 승인된 목표 정책이 문서에서 명확히 구분된다.
- Phase별 canonical 갱신 위치, DTO/data/save/version ownership과 baseline이 기록된다.
- `DR-01` 전투 목표와 `DR-15` Pause 중 명령 의미가 실제 runtime 경로와 focused evidence로 확인된다.
- 필요한 범위의 Core/server focused 및 broad 검증이 통과하고 실제 test 수가 기록된다.
- 미래 Phase placeholder, 승인되지 않은 compatibility와 범위 밖 production 변경이 없다.
- 완료 감사가 `Complete / High`를 판정하고 Master가 Phase 0을 `complete`, Phase 1을 `ready`로 갱신한다.

## DTO migration

없음. Phase 0은 기존 `LiveBattleUpdate`/checkpoint/playback와 server envelope shape를 변경하지 않았고
새 미래 Phase DTO를 추가하지 않았다.

## Data/live RON migration

없음. Production live RON은 baseline과 정책 추적 근거로만 조사하며 수정하지 않았다.

## Save/checkpoint migration

없음. Pause와 목표 관련 checkpoint/event 계약을 확인했으며 setup `1`, event log `27`, Core active-run
payload `2`, server envelope `3`을 유지했다.

## 배포 단위와 producer-consumer rollout 가정

Phase 0 문서 분리는 runtime 배포 shape를 바꾸지 않는다. Narrow correction이 필요해 producer-consumer
shape 또는 persistence version이 바뀐다면 기존 승인 범위를 넘는지 판정하고, 승인되지 않은
compatibility 없이 원자 배포 가능한지 확인하기 전에는 변경하지 않는다.

## 필수 focused 검증

- `DR-01`의 실제 전투 목표 판정/완료 경로와 대표 테스트
- Pause가 simulation clock과 scheduled event를 진행시키지 않는 경로
- Pause 중 각 현재 command가 Core에서 승인/거부되는 경로와 대표 테스트
- server가 Core command/result를 재추론하지 않는 mapping/transport 근거
- target decision record의 `DR-01`~`DR-17` 누락·중복·Phase coverage 검사
- 임시 문서의 동결·superseding 링크와 future-policy/canonical 분리 검사

Focused 명령은 `EXPERIMENTS.md`에 실제 결과와 함께 기록한다.

## 필수 broad 검증

- 영향받는 Core crate의 Pause/전투 lifecycle 관련 suite
- 영향받는 game_server command/transport suite
- 문서 링크, 정책 ID coverage와 Markdown diff 검사

Phase 0이 문서만 변경하고 runtime baseline이 일치하면 runtime test는 회귀 확인으로 실행한다.
Broad 명령은 Core/server package suite와 live RON loading suite로 확정했으며 완료 감사 전에 실행한다.

## 생략할 검증과 근거

- Unity runtime/contract/tests: 사용자 승인 비범위
- Phase 1~9 production behavior: 선행 구현 금지
- 현재 Phase와 dependency 근거가 없는 repository 전체 test: scoped broad 범위를 조사 후 선택

## 중단 조건

- 확정 정책 절 사이에 사용자-visible 의미 충돌이 발견됨
- `DR-01` 또는 `DR-15` correction이 독립 source-of-truth 경계로 분리되어야 함
- 승인되지 않은 DTO/save/wire migration 또는 compatibility가 필요함
- 필수 Core/server source, live data, canonical 또는 test runner에 접근할 수 없음
- 기존 사용자 변경과 안전하게 병합할 수 없음
- baseline 없이 변경 전후 회귀를 판정할 수 없고 대체 evidence도 불충분함

## Compatibility

승인된 compatibility는 없다. Strict current shape와 문서 분리를 사용하며 alias, fallback,
dual-read/write 또는 shim을 추가하지 않는다.

## 완료 감사 Evidence Matrix

조사와 검증 후 실제 파일·runtime·test evidence로 채운다. 문서 주장만으로 pass 판정하지 않는다.

| ID | 완료 조건 또는 불변식 | 정책 근거 | Runtime/data/contract 근거 | 검증 근거 | 판정 |
| --- | --- | --- | --- | --- | --- |
| P0-01 | 확정 정책 decision record와 전체 coverage | `DR-01`~`DR-17` 확정 절 | Target decision record의 17 heading/coverage row | ID별 count 1, 전체 17 | pass |
| P0-02 | 임시 조사 문서 동결과 superseding 링크 | Master Phase 0 | 임시 문서 상단/문서 지위 | historical/superseded marker 검색 | pass |
| P0-03 | 현재/target/canonical 책임 분리와 Phase baseline | Master source-of-truth 규칙 | Decision 책임 절, notes baseline/version matrix | source/version/canonical 대조 | pass |
| P0-04 | `DR-01` 현재 구현 정합성 | `DR-01` | `WinCondition`, `compute_winner`, rulebook | focused 3 passed, Core broad pass | pass |
| P0-05 | `DR-15` Pause 명령/시간 정합성 | `DR-15` | playback/revision/actor와 3 canonical | baseline 6 + 새 focused 3 passed, broad pass | pass |
| P0-06 | 범위·legacy·compatibility·dirty 보존 | Master 공통 불변식 | Future symbol 0, version 무변경, scoped diff | placeholder/diff/status audit | pass |
| P0-07 | focused/broad 검증과 완료 감사 | Master Phase gate | [Completion Audit](COMPLETION_AUDIT.md) | Core 847, server 54, RON 23 재실행 pass | pass |

## 현재 진행 상태

- Phase Goal 경로와 필수 문서 초기화: 완료
- 정책 추출, decision record 생성과 임시 문서 동결: 완료
- Runtime/data/server/canonical/version baseline: 완료
- `DR-01`, `DR-15` focused 검증과 누락 회귀 테스트 추가: 완료
- 관련 current canonical 동기화: 완료
- Broad 검증: Core 847/server 54/RON 23 재실행 모두 통과
- 완료 감사: `Complete / High`, Phase 1 activation 허용
