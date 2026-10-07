# R07 사본별 파편 실행 계획

현재: R06 경계 동결 인수 완료. R07 GEX-10 schema/소유 명령/조회/전투 source/resource/V8 및 명시 V7 변환 구현과 집중14개 수용 시험 PASS. 마지막 Run owner 정리 시험을 포함하여 전체 통합 검증 중. GOAL-CAFCP-01 / GS-CAFCP-001 / HA-CAFCP-001. 단일 writer. 사용자 연속 구현 요청이 실행 권한이며 P2G 자체에서 승인하지 않았다.

## 구현 계약과 순서

1. `OwnedFragment { copy_uuid, definition_id, unlocked_forms, selected_form, equipped_to }`, UUID-keyed BTreeMap만 소유 truth. inventory count/정렬 목록은 파생한다. copy cap과 form cap, 문자열 길이는 유한 검증; checked 전용 UUID stream은 기존 namespace를 변경하지 않는다. 지급 실패에서 UUID를 소비하지 않고 정상 GrantExecutor/EconomyGrantState 후보를 commit한다.
2. ActiveSkill의 stable form map과 `FragmentForm { skill_id, compatibility, bloom_recipe }`, 명시 dismantle recipe를 catalog validation에 연결한다. base 필수, forms 중복/unknown/skill/material 참조/0 amount/checked 중복 합산 오류를 게시 전에 거부한다. baseline BasicAttackModifier는 intrinsic로만 사용한다. 기존 강화 skill map과 shared progress consumer는 제거한다.
3. exact copy를 받는 equip/unequip/transfer/awaken/select/dismantle 명령과 projected roster/inventory 검증. 양방향 binding을 먼저 검증한다. transfer는 실제 원 owner와 대상 owner를 한 후보에서 수정하고 occupied 대상은 거부한다. 개화는 해금만, 선택은 별도 Maintenance 명령, 중복 해금/동일 선택은 무비용 무변경. 장착 분쇄는 명시 confirmation이 없으면 거부하고 confirmed에서 exact 해제+삭제+산출+typed resource owner 제거를 함께 commit한다. dead 회수는 같은 copy owner만 해제한다.
4. 공통 form resolver→stat/profile→runtime unit→action/source capsule. `FragmentSourceSnapshot`에 실제 copy/definition/form을 전달하며 EquipmentCopy로 가장하지 않는다. 필요한 ResourceOwnerKind/Selector의 최소 FragmentCopy/SourceFragment와 exact source binding을 추가하고 R05 lifetime 계약은 그대로 적용한다. old delivery는 기존 source Arc와 Snapshot을 유지한다. 다음 battle의 Certified input은 새 copy/form/owner를 반영한다.
5. V8 checkpoint decode/restore에서 단일 inventory와 모든 loadout 양방향 binding, definition/form/unlock, UUID와 typed-resource owner를 검증한다. V7은 자동 로드하지 않고 명시 converter에서 raw canonical checksum과 caller가 지정한 원래 content identity를 먼저 검사한다. 원본을 수정하지 않고 새로운 bytes를 반환한다. sorted definition/count와 직원 UUID로 결정적으로 사본을 배정하고 공통 awakened는 모든 사본의 awakened 해금/선택으로 승계한다. 제거 rank/zero-count progress는 compatibility history로 보존하되 gameplay에서 읽지 않는다. target generation의 참조와 전체 checkpoint를 재검증한다.
6. DTO에는 사본 목록, 각 copy의 selected/unlocked/owner, 모든 form의 해금/선택 가능 여부와 recipe·불가 이유를 제공한다. 동일 resolver를 command/preview/explain/restore에서 쓴다. CloneShape/retained admission은 새 map/set/string/source payload를 빠뜨리지 않는다. 공개 default RON fixture→정상 지급→장착/개화/선택→전투→새 process 저장 경로를 durable tests와 release CLI로 증명한다.

## 정책과 제한

위임 정책은 P2G의 DR-CAFCP-E01–06-001이다. 최신 active1 슬롯/일반 강화 없음 우선, passive/channel/override 확장 없음. recipe 없는 기존 항목은 명시 unavailable이며 외부 공개 fixture에는 완전한 enkephalin+환상체 part recipe를 제공한다. 기존 보상 dust와 제거되는 강화 progress는 구별해 저장 원본을 보존한다. 보상 identity 전체 개편은 R19로 확장하지 않는다. 실제 코드 대조에서 dust 보상 경로 처리 결정을 EXPERIMENT_NOTES에 남긴다.

검토 locator 정정: player spawns 실제 경로는 `src/game/combat_setup/player_spawns.rs`, resource container는 `src/game/resources/`이며, P2G CRA의 일부 사람이 읽는 경로 표기의 `battle/combat_setup`/`resources.rs`를 문자 그대로 가정하지 않는다. RSB는 실제 filesystem 파일 목록과 hash를 소유한다.

## 검증과 종료

F01–F12 / T11·T18·T20·T27 / 10 validation dimensions를 actual nonzero tests와 원자료에 매핑한다. focused 교정 후 최종 current source에서 R06 전체 영향+기본/feature/check/release/clippy/fmt 및 PC36/R05를 재실행한다. 20분 시험 없음. concurrent peak9030188 단독 초과 경고 외 hard guard 유지. source/data/test/config/fixture membership과 before/after archive/정역 patch/currentness/격리 복원을 검증한다.

최종 REVIEW_HANDOFF는 R06 경계 target과 최종 통합 target을 연결한다. 종료: R06·R07 구현·자체 검증 종료 / 최종 통합 target 동결 / 독립 감사 요청 대기. 독립 GEX20/30 및 R08은 수행하지 않는다.


## GEX-00 인수 완료 / GEX-10 시작

R06 `r06_boundary_001` manifest SHA256 `0b6938d77cc6cded598b2b01e51d41bf7e651dd66e8acf2f695a1cf3b5fd44f1`, source `fa8c1233ccefc7c08ad08ba7b3b78d0dcb25b526236ba4817ed7966071231f85` currentness와 격리 정역 복원 확인. baseline/SOURCE_BEFORE.tar.gz에 같은 source bytes를 보존했다. 모든 RSB 파일 currentness 일치. `evidence/GEX00_INTAKE.json` 참조. 이제 위1–6 순서대로 R07 제품 cutover를 실행한다.

## 최종 통합 종료

GEX-10 구현·자체 검증 종료. COVERAGE.json 및 R06_IMPACT_COVERAGE.json이 모든 수용표의 실제 검증을 연결한다. target/r06_r07_integrated_001/MANIFEST.json과 REVIEW_HANDOFF.md가 최종 locator. R06 경계 보존. 독립 감사 요청 대기이며 GEX20/30 또는 R08을 수행하지 않았다.
