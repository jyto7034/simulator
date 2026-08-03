# SavePoint And Gate Trauma Vertical Convergence Experiment Notes

Status: complete notes

Implementation complete: true

Created: 2026-07-27

Current-status correction (2026-07-30): current authoritative status는 `PLAN.md`와
`COMPLETION_REVIEW.md`의 `Complete / High` (2026-07-27)다. 아래 notes는 당시 조사·판단 history로
보존하며 재작성하지 않았다.

이 문서는 P00의 source-of-truth 충돌, dirty overlap, 기술 판단, 정책 질문과 범위 밖 후보를 기록한다.
Durable gameplay/data/wire 계약이 아니다.

## 1. Dependency

- Master: `run_persistence_facility_monolith_policy_master`
- Phase: P00
- 직접 선행 dependency: 없음
- 후속 consumer: P01 failure/checkpoint/active-run

P00은 final recovery schema와 의미를 소유한다. P01은 이를 교체하지 않고 failure와 durable
persistence를 확장해야 한다.

## 2. 초기 승인 경계

- Support/Rest cleanup만 수행하고 flat SavePoint heal을 canonical로 고정하지 않는다.
- SavePoint와 Gate 모두 final tagged `TraumaRecovery`로 전환한다.
- durable active-run과 failure-decision UX는 P01에 남긴다.
- facility map, SavePoint placement, local compendium, difficulty와 monolith는 구현하지 않는다.

## 3. 알려진 현재 충돌

Master 기록 기준으로 다음 충돌이 존재한다. Child 실제 상태 조사에서 현재 유효성을 다시 확인한다.

- Core map enum/payload는 SavePoint로 일부 전환됨
- RunPolicy는 flat SavePoint/percent Gate 혼합 상태
- SavePoint psychology event producer 잔존
- live map generation RON의 삭제된 Support variant
- game_server test의 삭제된 Support API
- Unity runtime/contract/mock/test의 SupportChoice/Rest 계약

## 4. Dirty Worktree

Core support/map/node-flow/run-policy, live map/run RON과 game_server message test가 이미 modified다.
외부 Unity도 dirty하다는 master 기록이 있다. 기존 변경을 초기화하지 않고 파일별 diff를 읽어
P00 target과 안전하게 병합한다.

파일별 diff 확인 결과 P00 overlap은 동일한 중단 migration의 일부다. 이미 전환된 SavePoint
category/session/result를 보존하고, 임시 flat recovery와 Psychology producer, stale RON/server/
Unity consumer만 승인된 final policy로 이어서 교체한다. 관련 없는 기존 dirty 변경은 수정하지 않는다.

## 5. 정책 질문

현재 없음. Final tagged recovery type, 초기 값, recovery 대상, psychology 제거와 P00/P01 경계는
승인 decision과 master에 기록돼 있다. 조사 중 승인 의미를 바꾸는 선택이 필요하면 구현을 멈추고
`policy_decision_required`로 전환한다.

## 5.1 문서 선독 후 source-of-truth 구분

- 현재 실제 상태: 이후 code/live RON/consumer 감사에서 확정한다.
- 현재 durable canonical: Core/Unity 모두 Support/Rest와 flat SavePoint recovery 일부를 보존한다.
- 승인된 목표 정책: Support/Rest를 제거하고 SavePoint/Gate를 tagged `TraumaRecovery`로 수렴한다.
- 판단: canonical의 Support/Rest 서술은 보존할 계약이 아니라 P00 migration 대상이다. 문서만 먼저
  바꾸지 않고 실제 producer, DTO, RON, server와 Unity consumer를 같은 수직 변경에서 갱신한다.
- Unity `unity_client_implementation_goal.md`는 과거 client skeleton 작업 지침이다. P00의 정책
  authority가 아니며 최신 실제 구현과 `unity_core_contract.md`를 확인하는 보조 evidence로만 쓴다.

시설 decision의 Master 표기와 실제 파일명이 달랐다. 실제 읽은 경로는
`docs/decisions/facility_map_and_node_manifestation_policy_2026-07-25_26.md`이며, P00은 이 문서의
facility/template 구현을 가져오지 않고 SavePoint category 및 별도 배치 정책 경계만 확인했다.

## 5.2 Final recovery schema와 preview 기술 결정

승인 decision의 exact semantic name을 따라 `RunPolicyData`는 Support group을 제거하고 다음 두
category-wide field를 직접 소유한다.

- `save_point_trauma_recovery: TraumaRecovery`
- `gate_trauma_recovery: TraumaRecovery`

`TraumaRecovery`는 `Flat { amount } | CurrentPercent { percent }`의 단일 tagged enum이다. Live RON은
각 recovery에 한 variant만 authoring하고 두 값은 모두 `1..=100`을 validation한다. CurrentPercent는
`current / 100` 분해를 사용한 올림 계산으로 overflow와 내림 오차를 피한다.

Core preview는 node category별 recovery와 roster에서 한 번 계산해 NodePreview command result와
NodeConfirm snapshot에 함께 싣는다. 각 직원 row는 UUID, eligibility, Physical Damage, 현재/예상
Trauma와 현재/예상 Mental 상태를 포함한다. 공통 preview는 `physical_damage_unchanged: true`와
SavePoint의 `replaces_run_checkpoint`를 명시한다. Unity는 이 DTO만 표시하고 percent나 대상 여부를
재계산하지 않는다.

SavePoint selected-event/state도 같은 preview를 사용한다. Gate는 진입과 동시에 Floor transition을
commit하므로 NodeConfirm preview가 유일한 사전 표시 source다.

SavePoint와 Gate commit은 전체 `GameCoreState` rollback guard 안에서 수행한다. 단계 중 오류가 나면
roster recovery, pending delivery, map/Floor state, checkpoint를 진입 전 상태로 되돌리고 성공했을
때만 최종 상태를 유지한다. 이는 P01의 durable file publish를 구현하지 않으면서 P00의 process-local
atomicity를 보장한다.

구현 후 Core/live RON/server 검색에서는 제거 대상 production symbol이 남지 않았다. 검색에 남은
구형 `Support`/`choose_support` 문자열은 legacy shape가 strict하게 거부됨을 증명하는 음성 fixture다.
따라서 이를 제거 대상 compatibility로 보지 않고 P00 검증 증거로 유지한다.

외부 Unity consumer도 final wire로 수렴했다. `SelectedEventModel`과
`SavePointResolutionModel`은 `type: save_point`와 strict recovery preview를 요구하고,
NodeConfirm/SavePoint presentation은 Core 전후 값을 그대로 표시한다. SupportChoice/Rest route와
command는 제거됐다.

## 5.3 Strict tagged boundary와 legacy 명칭

Externally tagged enum에 variant별 `deny_unknown_fields`가 없으면 RON/JSON이 선택한 variant에
다른 variant field를 섞어도 조용히 무시할 수 있음을 새 음성 test가 드러냈다.
`TraumaRecovery` 자체에 `deny_unknown_fields`를 적용해 `Flat`과 `CurrentPercent`의 mutual
exclusion을 deserialize 경계에서 강제했다.

Completion audit 중 `ensure_support_before_boss`가 SavePoint만 강제하는 live authoring field로
남아 있는 것을 찾았다. 이는 단순 역사 표현이 아니라 active schema라
`ensure_save_point_before_boss`로 교체했다. `world::support`는 SavePoint와 Maintenance를 함께
소유해 domain 이름도 맞지 않았으므로 `world::facility`로 이동했다. 관련 test-only Rest 변수와
fixture id도 SavePoint로 정렬했다.

HeadquartersContact 내부의 `HeadquartersSupportChoice`, `HeadquartersSupport` shop context와
player-facing support 표현은 SavePoint/Rest node 선택 mode와 다른 독립 domain이다. 이 명칭을 P00
legacy로 제거하면 HQ contract를 무관하게 바꾸므로 보존하고 legacy audit에서 구분했다.

Canonical은 실제 구현 뒤에 동기화했다. Core gameplay/runtime/data 문서와 외부 Unity wire/goal/
screen spec은 category-wide tagged recovery, SavePoint 15%, Gate 10%, Core preview,
Critical/Breakdown 포함·Dead 제외, Physical Damage 불변, Psychology 미생산 및 process-local
transaction을 current contract로 기술한다.

## 6. 범위 밖 후보

현재 없음. 독립 문제는 이 절에만 기록하고 새 Goal을 자동 생성하거나 P00에 섞지 않는다.
