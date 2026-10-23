# U14 코드 검토와 구현 인계

대상: `WR-CEDR-003`의 10요소/31수용 조건. 2026-10-14 현재 dirty workspace를 읽었으며 기존 변경을 U14 변경으로 집계하지 않는다. 이 문서는 구현 제안이며 제품·시험 통과가 아니다. 실제 source bytes/HEAD/관찰은 P2G_20_REVIEW_001과 archive가 식별한다.

## 현재 경계와 변경

| 경계 / 요소 | 현재 실제 코드 | 필요한 변경과 검증 seam |
| --- | --- | --- |
| ENTRY — G13/G14/G15/PERSISTENCE | `world/expedition_lifecycle::prepare_expedition_node_entry`는 U12 receipt와 >0→0 선택을 소유하고 전역 비용1을 차감한다. `node_flow::handle_confirm_enter_node_resuming`은 비용 뒤 활동에 진입한다. `employee::BreakdownCounter`는 remaining_nodes/entered_node이며 complete_breakdown_node가 완료마다 감소한다. node_flow의 완료·SavePoint·Gate와 facility_rooms 하강에서 소비한다. | 작성 비용을 입장 함수 하나가 읽고 receipt와 규정 유예 누적을 같은 transaction에 반영한다. 완료 카운터 호출은 제거하고 trauma 변화 때 reconcile만 유지한다. 새 사망은 recover_assets_for_new_deaths를 재사용하고 활동 전에 런 실패/복구 판정. 비용2/잔량1·0, 유예4, 치료 전 사망, 같은 identity 재입장/후퇴, 깊이2/3 소진을 공개 경로로 확인. |
| COMBAT — PWR01/PWR02/G17 | node_flow가 RunPowerAuthorityState::manifest_floor를 호출하고 해당 anchor는 authored minimum/previous rise/run peak의 max다. facility_generator·map_encounters가 target power를 써 조우를 고른다. CombatPreview 생성은 tuning의 HP/ATK/DEF와 waves를 적용한다. enemy_spawns가 preview scale을 spawn draft에 전달한다. run_policy의 monolith stage tuning은 별도로 작성된 단계 배율을 합성한다. | 편성/peak를 적 난이도 source에서 제거하고 작성된 floor/profile만 유지한다. 진입 전 비용 예상과 실제 입장후 snapshot으로 무억제 HP/ATK만 합성한다. 부활 배율과 곱셈, 봉인 추가 스탯은 무억제 때 제외. scale은 지연 spawn에도 적용. preview cache의 자격변경 stale을 피하고 실제 battle snapshot에 고정. 같은 조우/다른 roster, 150×150=225와 비대상 스탯 비교. |
| REWARD — PWR02/G18/G19 | combat_setup/rewards::apply_combat_reward_multiplier는 기존 authored budget이며 relic catalyst/fragment dust까지 포함한다. 전투 결과에서 battle.rewards와 encounter_victory.reward_effects가 별도로 적용되고 기본 생존XP는 apply_post_battle_resolution_to_state에서 따로 지급한다. 현재 보너스 목표 producer는 없다. 실제 DamageFact에는 source/strike/time/hp_before/hp_after가 있고 damage_runtime의 실제 mutation 후 record_fact에 결속된다. | 무억제 전용 whitelist 수량 변환을 하나로 두어 엔케팔린/장비가루/파츠/XP만 적용한다. 기본XP는 승리 snapshot일 때만2배, 패배10, 후퇴0. authored 성과 목표와 실제 비DOT 단발 피해/시간 accumulator를 연결하고 목표당1회. 일반전 난이도별 엔케팔린/가루 RON 보강. 이벤트 로그나 fixture의 가짜 bonus flag를 성과로 쓰지 않으며 실제 전투 기대값으로 검증. |
| GOLD — G16/G17/PERSISTENCE | FacilityLayout은 rooms/connections/reservations/activities를 작성하고 FacilityExplorationState가 layout/current/visited/activity metadata를 저장한다. 황금가지 plan/봉인 상태/보충 producer는 없다. 고유4종 본체는 현재 지원 제한과 별개인 후속 콘텐츠다. | layout에 유효한 authored plans/후보·가중치·보충/봉인 수치를 두고 new expedition에서 하나 선택. 배정은 전체 고정 room/slot identity로 저장한다. reachability는 연결 방향/깊이/조건으로 평가하며 UI 정보 미공개를 이동불가로 간주하지 않는다. 적합안 부재는 제작 오류. 정상 테스트 boss로 봉인 release/retreat/victory/refill 증명. 원본4종 본체를 가짜 구현으로 바꾸지 않는다. |
| STORE — 전요소/PERSISTENCE | ActiveRun codec v14와 current/SavePoint checkpoint가 run/map/roster/inventory를 함께 저장한다. application operation 전 prepare_continue_before_operation은 입장전 snapshot을 만들지만 node_flow 비용 뒤 capture_expedition_entry_for_continue가 다시 install하여 입장후로 덮는다. 후퇴는 BattleEntrySnapshot, 승패는 staged completion과 confirmed result를 쓴다. actor handlers는 정상 durable active/profile 경로를 소유한다. | 미확정 입장전 Continue snapshot을 덮지 않도록 수리하고 후퇴횟수 overlay는 유지. BattleEntrySnapshot은 비용/유예/입장사망 뒤 봉인해제 전 capture. plan/collected는 checkpoint에 포함, 전투 snapshot은 result owner에 포함. 새 shape는 version bump·명시적 구버전 거절로 원본 보존; 구개발저장 전환 의무 없음(D01). 실제 프로세스 Continue/SavePoint/IO 실패 재시도, 결과 후 중복 지급을 검증. |
| PUBLIC/DATA — 전요소 | FacilityActivityViewDto는 global cost/exhaustion을 공개하고 FacilityView는 미탐사 조우를 숨긴다. PveEncounter/FacilityLayout/RunPolicy는 typed RON/pack loader와 validation, content identity를 거친다. Application commands/snapshots는 Core 결과를 공개하고 server transport는 이를 직렬화한다. | 노드 작성비용·예상유예/사망·강화배율/봉인/가지 공개 DTO를 같은 owner에서 생성한다. pack/live RON/clone shape와 정상 fixture 등록을 함께 전환한다. 가지 표식은 비공개 이름과 분리. unsupported 신분을 supported로 오표기하지 않는다. 정상 loader의 유효/거부 대조와 public 명령/실제 battle 일치를 확인. |

## 권장 원자 전환과 대안

입장과 Breakdown, 전투 snapshot과 결과, 저장 reader/writer는 서로 소비하므로 같은 Phase의 원자 작업 묶음으로 구현한다. 배치·공개 경로는 같은 Phase에서 통합하고 마지막에 기능·정상호스트·비용 측정과 입력 동결을 수행한다. 기존 expense ledger나 별도 reward commit을 추가하면 중복/부분복원 위험이 있으므로 U12 receipt와 U04 staged grant·continuation을 확장한다. 파편·촉매제까지 배율을 적용하는 기존 budget helper의 무조건 재사용은 제외한다.

내부 타입/필드/함수 분할·정수 배율의 기존 산술 규칙/정확한 저장 버전·authoring 세부는 구현자 재량이다. authored 비용0을 허용하고 값의 범위/오버플로는 정상 제작 경계에서 검증한다. 수치 초기값은 현행 시작억제력12/비용1/유예4/100XP/기본XP10 및 룰북의 지정 범위에서 시작한다. golden 계획 가중치는 데이터에 명시하며 플레이 튜닝으로 조절하고 hidden fallback은 두지 않는다.

## 검증 인계와 한계

31수용 조건마다 실제 정상 상태→목표 연산→관측/기대값을 PLAN에 적는다. 현재 지원 공식 지도, 황금가지용 정상 제작 테스트 조우, 작은 전투 oracle와 공개 consumer를 구분한다. 실제 보스4종/U16 스트레스 활동/U17 부화 통합 전체는 후속 owner이며 현재 공통 계약을 미래 구현 완료로 간주하지 않는다. 전투 객체의 source/strike와 효과 DOT 분류·다중타격 처리, 모듈별 추가 caller/fixture/clone 소유량은 구현 시 필요한 범위로 확인한다. 최종보스 플레이/Unity/배포/최종 하드웨어 SLO는 미실행이다. Released/Deployed 입력을 제공받지 않았고 이 Goal은 workspace 구현이므로 해당 claim은 N/A다.

실행 시험0. 기존 U13 감사는 선행 완료 근거이고 U14 변경 뒤 기능/저장/성능 결과로 대체하지 않는다. 저장/계층 전환으로 영향받는 U04/U12/U13 회귀는 의미가 같은 것만 재사용한다. 변경된 이전 완료노드 카운터나 adaptive power 기대값 시험은 과거 의미 그대로 통과시키지 않고 최신 요구를 검증하도록 고친다.
