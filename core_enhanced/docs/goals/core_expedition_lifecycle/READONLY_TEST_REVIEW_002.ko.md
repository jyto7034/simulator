# U12 RQ05/RQ08 읽기 전용 시험 경계 검토

2026-10-12. 담당 `/root/core_main_astra/u12_tests` — 독립 시험 설계/작성 역할, 최종 감사자 아님. 현재 basis는 WR-CEXL-007 / RSP-CEXL-002다. HANDOFF_LATEST 최소 읽기, 개정 연속 지시, Working Policy, P2G10 policy/correction/result 및 RSP의 RQ05/RQ08을 확인했다. 제품·시험·fixture 수정과 빌드/시험 실행은 **0**이며 이 문서만 작성했다. P2G20 formal Binding 또는 GEX 승인을 대신하지 않는다.

## 실제 관찰과 최소 동시 전환

1. `EmployeePsychology`는 trust score, memories, recent events를 직렬화하고 기본값을 생성한다. 사건 kind는 빈 enum이며 새 기억 생산 근거가 없다. 제거 합의가 최종이며 이를 채우거나 호환용 빈 상태로 남기지 않는다. `Employee` 필드·constructor·clone shape, `game/mod.rs` export, `clone_shape.rs` serialized leaf, snapshot DTO 세 종류/employee 멤버와 projection/label, node completion 네 호출·advance helper를 같은 제품 변경에 포함해야 한다.
2. `Employee` 자체에는 `deny_unknown_fields`가 없다. 필드만 삭제하면 serde가 nested `psychology`를 무시할 수 있다. 현형식과 SavePoint 양쪽에서 구필드를 명시 거절하고 유효 checksum도 확인하는 시험이 필요하다. 단순 version 거절만으로 nested strictness를 증명하지 않는다. Continue version11과 checkpoint-only version12는 다른 계약이며 server `core_version`도11을 확인한다. 형식 변경을 한다면 둘과 checksum oracle/host gate를 원자적으로 맞춘다.
3. `application.rs`에 공개 V7/V8 converter가 남아 있다. V8은 원본 checksum/identity 검사 뒤 새 `RunCheckpointPayload`로 deserialize한다. 심리 필드를 조용히 버려 지원 상태로 바꾸는 경로가 되어서는 안 된다. 기존 converter 성공 시험은 삭제를 자동 승인한 것이 아니라 변경 대상 정책·회귀 의미를 메인이 결정할 지점이다. 구입력 거절과 원본 보호를 새 기대값으로 명시하고 frozen 과거 fixture/evidence는 보존한다.
4. 고정 성격 위험3종/사회3종, `EmployeeProfile`, `condition`의 physical damage/trauma, breakdown·성장·개인 identity는 별도다. `RunCheckpointPayload`와 `BattleEntrySnapshot`는 roster 전체를 capture/restore한다. 따라서 해당 clone graph를 유지하면서 제거 필드만 없애야 한다. 전역 `memory`/`trust` 문자열 삭제는 allocator 계량·trusted host authority를 손상시킨다.
5. `generate_employee_profile_contract_fixture.rs`는 실제 Standard 명령과 VN, required_count만큼 선발한 뒤 출력은 `required_count:1`/후보1/직원1로 축소하고 `employee["psychology"]`를 다시 넣는다. projection 미수정 시 null key가 남는다. fixture serializer 계약과 U12 정상 6중3 경로 증거를 구분해야 한다. 이번 범위에서는 checked-in 외부 UI fixture 위치를 특정하지 않았고 Unity 실제 검증을 주장하지 않는다.
6. 테스트 파일의 psychology 참조 제거와 보존 oracle는 같은 컴파일 전환에서 처리한다. `facility.rs`의 trauma15%/신체손상 불변/SavePoint 생성, load count·peak 비되돌림; `map_flow.rs`의 Gate trauma10%/사망자 불변; `equipment.rs`의 사망 roster 유지·장비/fragment 원자적 반환을 삭제하면 안 된다. 빈 memories 또는 점수 보존 assertions는 U12 통과 증거가 아니다.
7. `tests/mod.rs:76–99` checksum helper는 version==11만 continuation으로 간주한다. format bump 후 그대로 두면 구필드 시험이 잘못된 checksum에서 먼저 실패할 수 있다. helper 등록, explicit checksum-positive 대조, Core/server version 및 public example을 함께 갱신하는 것이 최소 시험 변환이다.

## 숨은 준비 전제

| 경로 | 실제 준비와 증명 한계 |
|---|---|
| `tests/mod.rs:453–510` | Endless일 때 tutorial_completed를 내부에서 true로 바꾼다. 기본 helper는 Standard, VN 1회, starter policy 수만큼 선발한다. 첫 실행 원정 허용·튜토리얼 무관성 증거가 아니다. |
| `checkpoint_validation.rs:111–178` | 내부 grant/equip, dust/material/consumable 지급, 첫 노드 SavePoint 강제, test-only save를 쓴다. graph/codec 시험에는 적합하지만 공개 획득/실제 맵 진행의 증거는 아니다. |
| `u06_starter.rs:184–340` | HQ를 candidate_count9로 강제하고 여러 번 무료 채용하여10/11, 전투 TP를99로 직접 주어8/9 배치를 검사한다. cap/ownership 경계는 보존하되 U12 가격·최대2회·깊이1–4 준비를 새로 맞춘다. |
| `normal_save_support.rs:27–101` | 실제 AdmissionSession Passed, 승인된 구성과 profile 로드, prepared command→ApprovedPersistence.publish가 필요하다. VN을 최대128회 처리하고 required_count 선발 후 roster≥2만 검사한다. 6후보/3명/예산1회 oracle는 없다. |
| `normal_save_tests.rs:124–135` | 교체 시작은 durable revision을 유지한 새 NotStarted application에서 수행한다. 실행 중 app의 StartNewGame을 합법이라고 가정하면 U11에서 이미 교정한 준비 오류를 반복한다. |
| `combat.rs:12652–12712` | controlled clone 회귀는 테스트 data, 넉넉한 clone/work 한도, 강제 SavePoint/전투를 쓴다. 새 정상 시작 증거와 별개로 clone/entry/retreat 보존 시험이다. |

## GEX 이후 필요한 독립 시험 묶음

아래는 계획이며 실행 결과가 아니다. 공개 흐름 준비를 한 번 확인한 뒤 관련 사례를 묶고 Sol이 단독 실행한다. 정확 selector/count·원시 결과를 남기고0시험/skip/timeout은 PASS가 아니다.

| 묶음 | 입력과 기대 oracle |
|---|---|
| 제거·보존 | 생성·채용·노드 완료·퇴각·Continue·SavePoint·snapshot의 직원에 psychology/trust_score/trust_band/memories/recent_events가 **키 자체 없음**. null/빈 객체도 실패. 비기본 위험·사회 성격과 손상/trauma/성장/UUID는 같은 원정 복원에서 정확 보존; 사망자를 새 개인으로 대체하지 않음. 새 원정 같은 seed/같은 정의는 새 개인이며 condition 초기화. |
| strict 저장 입력 | 유효한 현형식의 current와 독립 SavePoint snapshot에 각각 psychology의 null/빈 객체/실제 구구조 및 flat trust/memory 잔존을 넣고 유효 checksum 재계산. 먼저 변조 없는 양성 대조가 통과해야 함. 명시 거절·동일 retry 거절·입력 bytes 불변, run/profile/UUID allocator/encoded continuation/자산/loads/peak 불변. 새 대상 외에 비어 있지 않은 목적지의 허용 가능한 초기 상태를 구성해 atomicity 확인. 거절이 phase gate에만 막힌 경우 codec 검증 통과로 세지 않음. |
| 구저장 원본 | 이전 Continue·checkpoint·V7/V8 공개 경로의 지원 정책을 명시하고 미지원이면 거절. current와 backup 각각에 구형을 놓고 load/publish/invalidate/restart가 원본을 덮어쓰지 않는지 실제 파일 bytes로 비교. manifest/version/checksum 오류를 각각 분리. |
| 정상 첫 실행·튜토리얼 | 빈 profile에서 원정 바로 시작, 후보 역할 각2/총6 중 정확3. 0/2/4명·중복·unknown·기후보외 입력은 예산/개인/queue를 바꾸지 않음. 시작 VN/선발 retry·Continue로 예산/기본세트 재지급 없음. 선택/skip/반복 튜토리얼과 실제3층 승리는 vault·도감/탐사·첫귀환 story를 바꾸지 않음. |
| 채용·개인·한도 | 원정 고정 queue/offer/roster의 합집합 정확성, offer 재열기/복원 고정, 거절 후 원래 순서, 미선발 tail, 두 번 유료채용 및 세 번째 거절, 가격−1/가격/가격+1 잔액과 roster9/10/11·배치7/8/9. 모든 장비 기본세트와 fragment copy identity, 사망 기록 보존·중복 정의 미생성. |
| 대여·귀환·실패 | 총0/1/2 허용·3 거절, 방어구 포함 equipment/equipment·fragment/fragment·혼합, 같은 copy 중복·타 run/stale ID·없는 copy 거절. 성공 Gate 깊이2/3 경계와 억제력 소진 선택, 계속 탐사·전투 퇴각은 반환하지 않음. 정상 획득+대여 사본은 귀환에서 UUID·파편 개화/해금·선택 form을 보존하되 일반 무기 강화는 초기화한다. 같은 원정 Continue/SavePoint는 해당 시점 강화를 복원한다. 실패는 휴대분만 잃고 미휴대 vault 보존. 다음 원정 동일 seed에서도 copy/개인/run identity 충돌과 예산 승계 없음. |
| 회복·story | 실제 사용 가능 치료제/가격을 낼 수 있는 채용/장비+배치 가능한 직원/선택 가능한 회복·귀환 경로 각각 양성, 소유만 하고 사용 불가·도달 불가·예산 부족·배치 불가 음성. 첫 성공 귀환만 story pending, 표시 전 restart와 표시 후 ack/skip·중복 명령에서 정확1회 이력; 실패·계속 탐사·튜토리얼은 미발생. |
| 영구 자산 publication | 아래 원자성 시나리오를 승인된 host 및 unbound 정상 consumer에서 동일 소유 oracle로 확인. 단조 지식 union과 vault exact 이동을 혼동하지 않음. |

### 영구 자산 publication의 필수 부정 대조

메인 조사에서 ApprovedPersistence.publish/unbound handlers의 merge까지 확인했고, 이 검토는 `PlayerProfileState::merge`와 `application_intent.rs:273–360`의 accept/recovery 재merge를 직접 읽었다. 현재 구현은 OR/max/도감 union만을 전제로 한다. vault를 단순 merge에 추가하면 대여 감소가 되살아날 수 있다. authoritative state의 expected revision 및 accepted intent에 의한 정확 교체가 필요하다.

- vault A/B/C 중 A/B 출발 대여→preintent 실패: active/profile/vault bytes 및 actor 상태 모두 원상태, 재시도 때 한 번만 이동.
- accepted intent rename 직후 종료, profile/active/backup 각 publication 직후 종료→새 프로세스 복구: A/B는 run 한 곳, C는 vault 한 곳. 중복 복구·재명령에도 사본/예산 재발급 없음. 원래 승인 old/new configuration·active revision의 일치 검사를 유지.
- stale actor가 동일 checkpoint/no-change로 보이는 명령을 제출하더라도 이전 profile revision으로 vault 감소/반환을 우회하지 못함. 완전한 양성 최신 actor와 비교.
- 귀환 후 과거 SavePoint/Continue 재사용으로 이미 반납한 A/B를 run과 vault 양쪽에 되살리지 않음. 영구 지식과 첫귀환 이력도 되감지 않음. 정식 새 원정은 same seed에도 새로운 수명이다.
- 실패 후 A/B만 소실, C 보존. 성공 후 A/B의 사본 UUID·파편 개화/선택은 유지하고 일반 무기 강화만 정책대로 초기화. 임시 journal에는 권위가 없으며 unsupported current/backup/profile이나 제3 configuration을 자동 복구 대상으로 취급하지 않음.

보고 교정: T01–T07 대조 중 PE09의 일반 무기 강화 반출 초기화를 확인하여 위 두 귀환 oracle의 잘못된 무조건 속성 보존 문구를 정정했다. 원정 내 복원과 원정 간 반출을 구분하며 제품/시험 변경은 없다.

## 기존 증거의 적용

U11 `post_audit/u11_current_001_pass001/EVIDENCE_REVIEW.json`의 정확 scoped claim을 읽었다. 이번에 로그/시험을 재실행하거나 U11 source archive와 현재 workspace 전체를 비교한 것은 아니다.

| U11 evidence ID | 이번 판단 |
|---|---|
| `registry`, `required_config`, `calibration_authority` | registry/authority 알고리즘 비영향이면 과거 범위의 증거로 유지. U12 gameplay 완료 증거는 아님. |
| `entry_sharing_clone_restore001` | owned clone·entry·checkpoint·retreat 회귀. Employee/새 영구 상태 clone graph 변경 시 영향받으므로 해당 selector 재검증 대상으로 유지. |
| `deployment_boundary` | 실제 DeployUnit8/9 한도 수용 증거이며 내부 TP 전제 명시. 채용 가격/queue/setup 변경으로 준비 영향이 있으므로 최소 fixture 변환 후 같은 cap oracle 보존. |
| `same_original_save`, `durable_b_after_c`, `accepted_v2_crash` | 승인 구성·같은 원본·accepted old/new 복구의 유효한 선행 계약. U12 profile/vault publication 변경은 직접 영향이 있어 관련 save/crash 사례와 새 exact-ownership oracle로 검증. 구결과를 새 자산 보존 PASS로 승격하지 않음. |
| `approved_lifecycle` | 정상 승인 컨텍스트와 시험 준비를 재사용할 근거. 변경된 owner/binary/fixture는 기존 calibration을 자동 인계받지 않으며 필요 범위의 새 binding/관측 판단은 메인에게 있음. |

원시 receipt/source ID는 기존 EVIDENCE_REVIEW의 위 ID에서 추적한다. runtime source frozen identity `638eab584d31b2a98890a44bd9292126cf6cb1f0c03a812dd36060d535b89746`는 U11 최종 기준이며 아래 파일 identity와 같은 뜻이 아니다. 기존 psychology/빈 기억 assertions, hardcoded1명 fixture, 오래된 converter 성공은 최신 제거/원정 계약의 증거로 쓰지 않는다.

## source identity와 실제 읽은 범위

아래는 repository-relative path, 직접 본 line span, 보고 시점 SHA-256이다. `rg` 검색만 한 파일 전체를 읽었다고 간주하지 않았다. 지침/정책 문서의 내용 선택 읽기와 U11 evidence JSON의 선택 ID는 위에 별도 명시했다. scoped git status에는 다수의 선행 dirty가 있었고 보존했다. writer가 없다는 팀 배정 전제 아래 정적 검토했으며 OS 전체 프로세스 부재를 인증하지 않는다. 소스 currentness는 P2G20 owner가 아래 bytes에 정식 결속해야 한다.

| Source | 실제 읽은 범위 | SHA-256 |
|---|---|---|
| `core_enhanced/src/game/employee_psychology.rs` | 1–103 | `fc130cf2cb5bb3182962f6feae485078b8d63cd7dbbb4c73971d364a01ef9f69` |
| `core_enhanced/src/game/employee.rs` | 1–35,40–125,210–245,420–575 | `c98ffac82213d35e9ed7230e37583474b7876283a5c6c767b4871c90bf191bea` |
| `core_enhanced/src/game/mod.rs` | 1–18 | `b6f7885787f16b94b6061a74dd01624f0c3920819786e873b919376b56925150` |
| `core_enhanced/src/authoring/clone_shape.rs` | 375–405 | `0a2655ed95b092c4e61fe6fdeaeaf17f09a44f89da5f51cac791dd78433ab70f` |
| `core_enhanced/src/game/behavior.rs` | 1070–1110,1150–1185 | `fc10ddf39664cc898b6f7f02510987e15c22621e2523406f30eb26344b92b652` |
| `core_enhanced/src/game/world/snapshot.rs` | 1–25,215–255,830–887,1500–1522 | `aff77cec6fcdbc40248630c365bdf0ce52d6e585ca00531b1e64a4beb4731561` |
| `core_enhanced/src/game/world/node_flow.rs` | 1265–1290; 호출행738/1133/1182/1228 검색 | `6af28e7a3981b0ff899ce387914bc225bab9ba3b1d35510ab7828d421c718cd2` |
| `core_enhanced/src/game/recruitment.rs` | 1–257 | `647beeb970d657d699247a0bdb5279d3d488f8d3bd936cca70ced305b18f6245` |
| `game_server/examples/generate_employee_profile_contract_fixture.rs` | 1–122 | `f1573dd845882cf520528c5fa579dc8f4b83f864686e66eedec907e2dcd4dc93` |
| `core_enhanced/src/game/world/tests/mod.rs` | 70–101,430–510 | `6de7e8055b919696bf6d43aab21f774dd02c1c3fc61c499de41dd9a882c3d6d6` |
| `core_enhanced/src/game/world/tests/checkpoint_validation.rs` | 1–260,658–706,771–866,998–1027,1087–1120,1204–1250,1310–1370 | `7345a3f6fc87e59824eafb372f01ac5090c975f5cb7204b7cc9523290f368b84` |
| `core_enhanced/src/game/world/tests/snapshots_and_start.rs` | 770–825 | `8930f9f144123c7983c794c7384b4bd46a4089b644875943a10b33e2ffece6af` |
| `core_enhanced/src/game/world/tests/facility.rs` | 1–110,394–520 | `5f8d7431b53186926e8f3c5128eb2874a562f7181eea24c39149a44ead174e91` |
| `core_enhanced/src/game/world/tests/equipment.rs` | 1640–1720 | `4a86cf95fd66a490da91bfa79420f4c4764da5cfb1ed55750b7885c7ccbbd11b` |
| `core_enhanced/src/game/world/tests/map_flow.rs` | 545–640 | `d963ebc489380869a5223c69af5366b408ac85dc3e46e623d67e9beb82235aa5` |
| `core_enhanced/src/game/world/tests/u06_starter.rs` | 1–75,180–340 | `a82bcdd730f3832f9554eacfc53c62a85be44996189b6465378032e2a12b2540` |
| `core_enhanced/src/game/world/tests/combat.rs` | 12652–12712 | `eaec12049c4a9fe79a65f01caf193dde1a5b85972e92f57105417cec58e0fbb1` |
| `core_enhanced/src/game/world/state.rs` | 545–661,835–905 | `4aadbd4d28f995fcdc96d23a55b074ada9fbb9462e609342717de7f23436417b` |
| `core_enhanced/src/application.rs` | 1680–1712 | `917c4afd7a3cea97b2ecaebe5891a936fb7b65fb9a5010080c9a62bb232d2e9d` |
| `core_enhanced/src/game/world/active_run.rs` | 1–60 | `b1c6050cec18630b254a6559a443d1e4a7c84b25cf34e9fca61c1aef8cfdd948` |
| `core_enhanced/src/game/world/active_run/continuation.rs` | 105–130,733–824; VERSION296 검색 | `5558f49c6f7a7c2f8e7f761aaf9d526918dd32d83d262a287613a116d315f06b` |
| `core_enhanced/src/game/world/active_run/v8.rs` | 50–101 | `8fffa46c8137b90c9d88274e1f4dc1f5d8ea70aa60496a412f007d491b37a710` |
| `core_enhanced/src/game/encounter_compendium.rs` | 20–62 | `a340cd035b775a4c56ea6cbcfb3900098c9d3147da94993521788d4571fbde4c` |
| `game_server/src/persistence/active_run_store.rs` | 180–198,850–903 | `44bbdb57a331f0140765993327eb999a5c74e325d3ffaa42a136407f31d3e31c` |
| `game_server/src/persistence/player_profile_store/application_intent.rs` | 273–366 | `b964127e6ef336aea60bf1cb1fc919a9a98ea915112e1939f6bce864756ed618` |
| `game_server/src/mod_admission.rs` | 1–10 | `a4f3cb36ed5d0192f9d659933faee87c436ea62d004d98375c9f384d8a695d80` |
| `core_enhanced/docs/goals/core_mod_validation_apply/execution/single_goal/test_parts/runtime001/shared/normal_save_support.rs` | 1–103 | `9ce1be557d5b07ad780aac03f6c84f3a670fb85969db409b3bf4e199969fb0ce` |
| `core_enhanced/docs/goals/core_mod_validation_apply/execution/single_goal/test_parts/runtime001/shared/normal_save_tests.rs` | 124–135,246–269,304–337 | `6af5b078d76daabac1fbfbd59be8f07721ee83c1a1094b4a6b87277eca01b9f3` |
| `core_enhanced/docs/goals/core_mod_validation_apply/post_audit/u11_current_001_pass001/EVIDENCE_REVIEW.json` | limitations/currentness 및 evidence ID별 scope/disposition; 선택7 ID command/raw/source | `de7eae1d44a6a09f1af209423ef9ec51722044697b05b5586563ded52e16b550` |
| `core_enhanced/docs/goals/core_expedition_lifecycle/P2G_20_SOURCE_PLAN_002.json` | RQ05/RQ08 전체 구조와 현재 basis | `96e0e0db4a314a5d8c2fee9cd5a0cb9e5607449e80685458e4f0c3548b62c405` |
| `core_enhanced/docs/goals/core_expedition_lifecycle/WORKING_POLICY.ko.md` | 전체 | `5ca806f311051547854fab793c773a54d9ff374fdd3699d8c51885d5bae292fd` |
