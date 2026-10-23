# U12 RQ06/RQ07와 원정 초기화 — 제품 담당 읽기 전용 보고 002

2026-10-12, 제품·준비된 시험 실행 담당 Sol6.1 xhigh. 기준 WR-CEXL-007 / RSP-CEXL-002. 메인 Astra에 실제 코드 사실과 전환 지점을 인계한다. 정책 결정·정식 P2G-20 Binding·제품 완료·시험 통과 판정이 아니다. runtime/제품 데이터/시험/fixture/oracle/comparator를 수정하거나 빌드·시험·런타임을 실행하지 않았다. 누적 dirty를 보존했으며 이 보고서만 저장한다. 기억·신뢰는 완전 제거된다는 현재 합의를 적용하고 보존/생성안을 제안하지 않는다.

## RQ06: 실제 콘텐츠 경로와 확인한 사실

- `game_resources/data`의 RON이 실제 카탈로그다. `live_ron.rs:273–300`에는 26개 논리 소스가 등록되어 있고 run policy·직원·장비·파편·조우·시설 template·assignment가 모두 포함된다. embedded loader는 test/debug/test-fixtures용 capability(`data/mod.rs:270–291`)이며 정상 release 소비는 pack loader다. `RunPolicyData::builtin`을 정상 release bootstrap으로 간주하면 안 된다.
- `content_pack/raw.rs:208–215,526–575,1055–1068`은 같은 run policy와 map 카탈로그를 파싱·assemble한다. 현재 `StandardSchedule`은 floor를 fixed key로, `EndlessSchedule`은 stages 전체를 한 fixed key로 소비한다. 모드/스케줄 DTO를 바꾸면 embedded 파싱뿐 아니라 이 raw definition 분해·조립·key 경계를 함께 바꿔야 한다.
- `node_assignment_profiles.ron:3–61,130–151`에서 Standard 1/2/3층은 고정 슬롯을 사용한다. 1층 ordinary combat/event/reward, 2층 SavePoint/maintenance/HQ/shop/event/포큐버스 Elite, 3층 maintenance/shop/HQ/reward와 `suppress_nothing_there`가 실제로 결속된다. `facility_templates.ron:17–96`은 고정 연결·FloorGate·최종 `StandardCompletion` terminal을 제공한다. `PveEncounter`의 아무것도 없는은 `NormalBoss`다. 고유 최종보스 구현 완료를 의미하지 않는다.
- `FacilityMapGenerator::generate_with_requirements` → schedule resolve → template assignment → fixed difficulty 경로가 실제 연결되어 있다. fixed payload에 encounter ID가 이미 있으면 `world/map_encounters.rs:32–40`은 이를 다시 추첨하지 않는다. fixed difficulty는 `target_power:0`이고 floor anchor는 checkpoint 무결성 witness로만 유지한다(`facility_generator.rs:973–1016`). `tutorial_fixed_v1`은 authored_fixed이며 stat/wave/reward multiplier가 모두100, extra wave는 비어 있어야 한다(`run_policy_data.rs:804–818`). preview는 이 profile의 tuning을 사용한다.
- 약화값 HP720/ATK53/DEF36/MR33은 `standard_floor_3/final_chamber`의 `actor_stat_overrides`다. 정상 World 전투는 Standard schedule에서 해당 슬롯 정책만 조회한다(`world/combat.rs:620–645`). enemy spawn은 abnormality ID가 일치하는 actor에만 exact override를 붙인다(`combat_setup/enemy_spawns.rs:146–170`). 전역 abnormality 정의를 약화하는 방식이 아니다.
- `world/node_flow.rs:284–354`는 fixed 배치 후 공용 encounter·boss omen 경로를 호출한다. `boss_omen.rs:341`에는 Endless 전용 gate가 이미 있다. `world/combat.rs:3247`의 encounter progression도 Endless 전용이다. 다만 `commit_encounter_registrations`는 모드 확인 없이 영구 profile 도감을 합치며 승리 확정 경로가 이를 호출한다(`combat.rs:782–786,3466–3507`). 현재 tutorial 전투 승리를 원정 영구 도감과 격리한 것으로 볼 수 없다.
- `GameMode`는 Standard/Endless이고 progression은 각각 bounded/unbounded다. `NodeAssignmentProfileDatabase::resolve`도 이 두 모드와 standard_schedule/endless_stages를 사용한다. live 시작 VN trigger도 두 모드 이름을 소비한다. 현재 튜토리얼 완료 전 Endless gate와 Standard 완료 flag는 WR007의 새 제품 모드 계약과 다른 실제 코드다.
- 초기 setup은 3층/선발3명/500이며 직원 정의는 총12명, starter_eligible6명이다. six loadout은 healing staff/shield/sword/gun/greatsword/area healing staff다. 현재 validator는 starter 수5–7만 확인한다(`employee_data.rs:10–11,94–109`). 역할 이름은 profile 문자열이며 현재 읽은 타입에는 딜러/탱커/힐러 각각2명의 typed 시작 후보 제약이 없다. 6명이라는 현재 데이터 사실을 역할별2 계약 구현으로 확대하지 않는다.
- 기존 `initialize_employees_with_loadouts`는 grant preflight → exact copy 지급 → 장비 배정 → active fragment copy compatibility/equip을 사용한다(`world/helpers.rs:616–760`). 시작과 채용이 재사용할 실제 기본 세트 seam이다. 이번 보고는 모든 grant 실패 경로가 검증됐다는 주장이 아니다.
- Endless 초기 스케줄은 홀수/짝수 profile의 가중치로 HQ를 뽑는다. `endless_loop_a/service_wing`, `endless_loop_b/service_hub`는 HQ를 허용하는 실제 연결된 슬롯이지만 HQ 고정 tag/assignment가 없다(`facility_templates.ron:98–156`, assignment profile63–151). 현재 데이터가 깊이1–4의 고정 유료 채용 기회를 보장하지 않는다. 기존 template/profile/slot assignment를 이용한 한정 개발 원정 콘텐츠 전환 지점은 있지만 U13 전체 시설 완성을 추정하지 않는다.
- `reference_cwbs_final_boss`는 명시적 ReferenceFinalBoss이며 reference-only 보상·bonus·progression을 parser가 거절한다. 튜토리얼/ST1–NT60 또는 개발 원정의 정상 지원 조우 대신 사용해 완료를 주장하면 안 된다.

## RQ07: 정상 U11 승인 구성 소비 경계

- 정상 server `main.rs:63–82`는 `bootstrap::load_approved_host` → AdmissionSession → ApprovedPersistence → approved actor를 사용한다. 승인 설정 누락을 static loader로 우회하지 않는다(`bootstrap.rs:74–79`). `consume_content_pack`은 별도의 static runtime probe이며 정상 승인 launcher 증거가 아니다.
- 정상 local `mod_launcher.rs:38–69,108–129`는 AdmissionSession의 host → approved persistence → profile hydrate → `prepare_with_future_generation` → `ApprovedPersistence.publish`를 사용한다. start 명령만 미래 승인 generation/required configuration으로 교체되고 기존 active application은 기존 required를 유지한다(`mod_validation/host.rs:95–133`). server handler도 start에서 미래 generation/required를 staged 준비하고 bound publication으로 넘긴다(`handlers.rs:356–375,3956–3970`).
- ContentContract 현재 값은 pack1/catalog5/authoring1/execution2/fingerprint1이다. pack manifest의 complete contract equality와 unsupported capability 거절은 실제 loader 경계다(`content_pack/schema.rs:10–25,79–151`). 공식 packaging은 현재 contract와 capability 목록을 생성한다. U12 capability 이름/버전의 선택은 메인의 설계 사항이며 이 보고에서 임의 확정하지 않는다.
- semantic content identity는 contract + pack identities(required capabilities 포함) + writer receipts + 최종 정의의 canonical bytes를 해시한다(`resolver.rs:58–67`, `content_pack/mod.rs:291–307`). Candidate는 source receipt와 runtime safety, RequiredConfiguration을 결속하며 changed/removed definitions를 fingerprint로 구분한다(`candidate.rs:183–235`). U12 DTO/콘텐츠 변경을 이전 semantic identity로 유지하는 경로는 재사용 대상이 아니다.
- LabProfile context는 suite 전체·revision·반복·timeout·output한도·safety bytes·실제 worker pair를 포함한다(`lab.rs:317–330`). issuer cache key는 issuer/candidate view/suite/profile/pair를 포함한다(`issuer.rs:280–293`). build identity는 Core src·game_resources/data·server src와 manifest/lock/config/compiler/target/feature 입력으로 만들어진다(`build.rs:108–177`, `identity.rs:18–29`). 따라서 실제 U12 입력 변경 후 U11 과거 승인/cache/criterion을 새 구성의 승인으로 표기하면 안 된다.
- issuer-owned ApprovedGeneration의 foreign owner/stale revision은 Apply 경계에서 거절한다. ApprovedHost Apply와 save acceptance는 installation lock으로 묶인다. `ApprovedStoreGate`는 installed pinned/future configuration만 허용하고 required/content identity를 검사하며 실제 store constructor 이전 read-only preflight를 수행한다. accepted old/new 기록 복구 의미는 메인이 별도 검토하는 경계다.
- `PreparedApplication`은 profile/checkpoint/result를 prepared 상태로 유지하고 commit을 한 번 소비한다. ApprovedPersistence는 현재 profile과 prepared profile을 `merge`한 뒤 `accept_application_with_requirement`에 전달하며 accepted revision 뒤에만 Core commit을 소비한다(`persistence.rs:67–113`). 기존 profile merge는 set union/OR/max다. 대여·보관 자산처럼 감소 가능한 소유 상태와 이야기 소비/새 instance 수명에는 그 merge를 그대로 적용할 수 없다.
- 현재 U11 normal prefix와 public smoke generator는 실제 `game_mode:"Standard"`를 입력한다(`normal_preparation.rs:64–107`, `prepare_public_entry.py:72–77`). 정상 공개 enum 전환 후 이 입력·suite·required definition 선택은 테스트 담당 Astra가 새 계약에 맞춰 갱신해야 한다. 기존 helper의 결과 binding/정상 VN·starter prefix와 LabPreparation grant/plan 경로는 재사용 가능한 seam이다. Lab의 유한 bulk preparation은 player command나 승인 자체가 아니다.

## seed·run identity와 전체 초기화

- `GameCore.run_seed`는 생성자 입력이며 `RunProgression`도 같은 seed를 보유한다. `UuidManager`는 seed+namespace별 counter만 있고 별도 expedition instance identity가 없다. 같은 seed/namespace/counter는 같은 UUID를 만든다. scenario seed로 queue/map 선택을 재현하는 기능과 여러 원정의 owned copy identity를 분리하는 전환 지점이다.
- 정상 start는 roster/run/node/battle/Continue/checkpoint/VN을 지우지만 enkephalin/inventory/fragments/relics/crafting/run power authority/UUID counter는 지우지 않는다(`world.rs:857–900`). starter 선택은 500을 checked_add한다. 실제 반복 실행 검증 없이 중복 예산/자산 누수를 확정 시험 결과로 말하지 않지만 현재 초기화 코드가 원정 자원 수명 전체를 닫지는 않는다.
- abandon은 `GameCoreState::new(self.run_seed)`로 전체 root를 재생성한 뒤 player/profile만 보존한다(`world.rs:902–913`). 이는 전체 reset seam인 동시에 같은 scenario seed에서 UUID counter를0으로 되돌리는 경계다. profile vault에 이전 copy를 추가하면 이 경계와 새 copy namespace를 함께 바꿔야 한다.
- SavePoint는 inventory/roster/fragments/UUID manager를 전체 capture/restore하며 VN namespace counter만 max 보존한다(`state.rs:545–657`). Continue는 검증 후 saved run seed를 채택하고 fresh UUID manager를 생성한 뒤 payload counter를 복원한다. current/SavePoint의 소속 검사는 현재 같은 run_seed만 비교한다(`continuation.rs:763–831`, `node_flow.rs:1368–1386`). 새 instance identity는 current/savepoint/Continue/rollback에 함께 결속해야 한다. checkpoint 재생을 다음 원정 생성으로 세면 안 된다.
- `BattleEntrySnapshot`은 철수/재진입용 같은 원정 전체 자원을 복원한다. GameCoreState·RunState·checkpoint·BattleEntrySnapshot·ActiveRunTransactionRollback과 clone_shape는 새 필드의 수명별 capture/restore 표면이다. rollback의 현재 영구 profile 단조 merge 역시 감소 자산 상태를 추가하기 전에 검토해야 한다.

## 최소 동시 전환 표면과 남은 한계

1. **공개 모드/시작:** GameMode/progression, behavior/application/DTO, lifecycle/start/VN, fixed tutorial 선설정과 expedition six starter 선택. 옛 Standard/Endless의 자동 alias 여부를 제품 구현자가 정하지 않는다. 메인은 Tutorial/Expedition 전환을 현재 설계 제안으로 전달했으며 아직 제품 작성 시작 지시가 아니다.
2. **현재 카탈로그:** run policy/employee role·loadout, 시설 schedule/profile/raw definition 키, live source schema/complete content contract/capability/packaging. 고정3층과 현재 연결된 활동·전투 실행기를 보존하며 실제 초기 HQ 기회를 한정 authored profile로 연결하는 표면이다.
3. **상태 수명:** player/profile·vault와 원정 root를 분리한 reset/종료/대여, scenario seed와 instance UUID namespace, SavePoint/Continue/rollback/clone_shape. 선택/대여/시작 예산과 영구 profile 확정은 기존 prepared commit 경계에 포함되어야 한다.
4. **모드 격리:** 전투 영구 도감 등록, floor/first story 진행과 튜토리얼 완료/skip/repeat/실패/restore. 공용 보상은 튜토리얼의 임시 소유 상태에서 정상 실행할 seam이며 새 copy를 영구 보관에 흘리는 경로는 별도 차단 지점이다.
5. **승인/정상 저장:** ApprovedHost future generation/required configuration, ApprovedStoreGate preflight/acceptance, ApprovedPersistence와 accepted journal, 실제 launcher/server normal handler. U11의 승인 권한을 보존하면서 감소 자산에 정확 교체를 연결해야 한다.
6. **독립 시험 후속:** 새 공개 모드를 소비하는 정상 prefix/suite/fixture/query 사례와 영향받는 U11 회귀를 Astra가 작성·선정한다. 현재 보고는 static 읽기이고 정상3층 승리·원정 귀환/실패·crash 복구·새 capability 승인에 대한 새 실행 증거가 없다. 후속 승인 계획 전에는 빌드/시험을 시작하지 않는다.

아래는 실제 읽기 명령의 범위와 보고 발행 직전 파일 bytes의 SHA-256이다. 검색으로 찾은 파일 전체를 의미 검토했다고 주장하지 않는다. 범위 표는 읽기 대상으로 요청한 부분이며 긴 출력의 일부가 도구 표시 한도에 잘린 경우 핵심 주장에 필요한 부분을 다시 읽었다. JSON 문서는 표시한 payload projection만 의미 검토했다. 현재성 정식 Binding·저장소 전체 동결은 메인의 P2G-20 작업이다. 보고 생성 후 같은65개 파일 해시를 다시 대조한다.

## 읽기 범위와 source bytes

| 파일(repository 상대 경로) | 읽기 범위/선택 | SHA-256 |
| --- | --- | --- |
| `core_enhanced/docs/roadmaps/core_first/execution/CONTINUOUS_EXECUTION_INSTRUCTIONS_REVISED.ko.md` | 1–711 | `446c3ad0b94fd451aaf86031c4aca2c214a8b0e134a8c21027fcbe162e32bac5` |
| `core_enhanced/docs/roadmaps/core_first/execution/HANDOFF_LATEST.ko.md` | 1–53 | `c76ac120aa16f5a7857ef2326616fd2eecaa37b494b6a2db9001279927fefcec` |
| `core_enhanced/docs/roadmaps/core_first/execution/u12_psychology_removal_20261012_001/RECORD.ko.md` | 전체 | `df4ac657d766d401202db43eb6de008cf79f8b47364abd58f890e83eefea4b84` |
| `core_enhanced/docs/goals/core_expedition_lifecycle/WORKING_POLICY.ko.md` | 전체 | `5ca806f311051547854fab793c773a54d9ff374fdd3699d8c51885d5bae292fd` |
| `core_enhanced/docs/goals/core_expedition_lifecycle/P2G_20_SOURCE_PLAN_002.json` | artifacts[0].payload.Requirement Set: RQ06/RQ07 | `96e0e0db4a314a5d8c2fee9cd5a0cb9e5607449e80685458e4f0c3548b62c405` |
| `core_enhanced/docs/goals/core_expedition_lifecycle/P2G_10_POLICY_001.json` | Policy statement/Observable Policy Acceptance 요소 projection | `5d6fae6b2449063fd99f39635a8c0f5d69d4bd38a7ad2b97d4249890fa4fb0cc` |
| `core_enhanced/docs/goals/core_expedition_lifecycle/P2G_10_CORRECTION_001.json` | Policy statement/Observable Policy Acceptance 요소 projection | `3cdb18cb9dc2b5a14d1d88017befb2514ce461a814747bcb1879a47a6690cffc` |
| `core_enhanced/docs/goals/core_expedition_lifecycle/P2G_10_RESULT_002.json` | status/result_working_revision_ref/open_policy_question_ref_set/limitations/next | `a153fec31cbdb99a7f7eaed96c7b17d08b33d341ac7f46eac88a5d33bd4ca5d3` |
| `game_resources/data/run/policy.ron` | 1–170 | `96227e7cbc9af67b6ab28192910e732cfd881a4650d20b4cbfe4996f2eb16726` |
| `game_resources/data/map/node_assignment_profiles.ron` | 1–153 | `9389e0644e4f4c4ca75a861c07f423cb484c6f11a5867cf24ee6e8b769f7d8d3` |
| `game_resources/data/map/facility_templates.ron` | 1–157,202–210; schedule ID 검색 | `6387e53807aec270e74c40cec9c3a47baf6a4eb7c7476c1b4a5070f373c37d3c` |
| `game_resources/data/employees/definitions.ron` | 1–184 | `ec80a91cf21bd101d257270fe81a136e5a87b70aa0025a180e5357bac71bdc70` |
| `game_resources/data/visual_novel_triggers/base.ron` | 1–22 | `4216706ac783ee5d11662c616360cd0054564c3297db6953245d61ca2c038875` |
| `game_resources/data/pve/encounters.ron` | 1–66,259–293,1917–1951,2758–2799; encounter ID 검색 | `8b2472db6750dcbf883bd0d826320e5a139b9a2ef47f3f460efae378c22fecde` |
| `game_resources/data/bundle.ron` | schema_version 검색 | `b01f95eb8eb1f3fa3b081150ba6ada09bd288e38b55a283f5b08af06d4a102ad` |
| `core_enhanced/src/game/data/run_policy_data.rs` | 1–205,340–420,664–724,783–818,835–934 | `2857ef73a097be23ea77ee25afc383b52bd50b1121e2e32d9bf71dcd08b78373` |
| `core_enhanced/src/game/data/employee_data.rs` | 1–145 | `bed7d6f4bebf981a2bab274f323e6cd53d89feec39507fc95c4fcc939f3a25d4` |
| `core_enhanced/src/game/data/live_ron.rs` | 1–50,254–327,387–433,1125–1220,1312–1423,1680–1735,2480–2555; loader/reference 검색 | `2137876160f9dd4ea53d8d73ddebc1594e4f975da7a5129252d54ca5389956a8` |
| `core_enhanced/src/game/data/mod.rs` | 262–306; loader symbol 검색 | `483f6adbaecb5b0d9b710e14fa0721e7f708876ce71bb16d80eb78025a39170e` |
| `core_enhanced/src/game/data/content_pack/raw.rs` | 194–218,526–579,1038–1078 | `b152d5fe64ed391ab2c4758d280b948bbd0871577a6403581ff8c56f781c65ea` |
| `core_enhanced/src/game/data/content_pack/mod.rs` | 262–312; fingerprint 검색 | `1155196933f7844f63d09381da4e09aea2fc9f323e57c2f985033c16ed574b40` |
| `core_enhanced/src/game/data/content_pack/schema.rs` | 1–166 | `a43422dc725c57987225d00f264deccd33d5542f4d109d3524e81f712989d131` |
| `core_enhanced/src/game/data/content_pack/resolver.rs` | 1–77,316–365 | `984b7c782eafe3aadbc244b56570505386ffe0acb9b461f56102687682569399` |
| `core_enhanced/src/game/data/content_pack/packaging.rs` | 1–68 | `05b7436793f569a608b93e5a8d0529ab46914d0609e86b23aea7d03affe13b54` |
| `core_enhanced/src/game/data/pve_data.rs` | 352–376,909–942; reference-only 검색 | `81a3bcc3d6819bb49e3d4549c858495453d1cf8de018db946608791b2c635a19` |
| `core_enhanced/src/game/map/progression.rs` | 1–165; mode/terminal 검색 | `798c04a1fe8dce510e67832884ae5c32b45a1ea49556fe324caf52e055bfe014` |
| `core_enhanced/src/game/map/facility.rs` | 101–205,339–408 | `2762c1aa50a21c14ecd9b9f99bfc4409c863b40238a7a101c36d3cca560f949b` |
| `core_enhanced/src/game/map/facility_generator.rs` | 45–205,318–379,760–806,973–1026; fixed/tagged difficulty 검색 | `65045baf30051e3a2bca8861b50907db959192864d973c660f4e3820219db5a4` |
| `core_enhanced/src/game/combat_preview/mod.rs` | 413–473; difficulty 검색 | `9e184b5ca43efa0e46f669e4d46c47d898943454185033ec903f8851fba37dfc` |
| `core_enhanced/src/game/combat_setup/enemy_spawns.rs` | 128–177; actor override 검색 | `c75b68c9cb100440d076ba3231843376ff55ab972f2641bf29e9ed23d0e8f73e` |
| `core_enhanced/src/game/world.rs` | 55–170,830–1020; reset/seed 검색 | `a80724cda27845f2d5136c73c603644b3e3e39b88232ca7462c3f7ac586b8ffc` |
| `core_enhanced/src/game/world/state.rs` | 180–208,540–657,753–815,839–858,892–912,1755–1835,2180–2315 | `4aadbd4d28f995fcdc96d23a55b074ada9fbb9462e609342717de7f23436417b` |
| `core_enhanced/src/game/world/node_flow.rs` | 284–380,618–640,1212–1257,1368–1396; completion/seed 검색 | `6af28e7a3981b0ff899ce387914bc225bab9ba3b1d35510ab7828d421c718cd2` |
| `core_enhanced/src/game/world/helpers.rs` | 543–590,583–768; loadout symbol 검색 | `7a11ea149e108fc56fdb1de48ebec9e51787a85c003c96bb1157f20262932974` |
| `core_enhanced/src/game/world/map_encounters.rs` | 1–146 | `86d1a53cc437447538029b18b5af5c5feccb667db0486e94363d8caa80707d2d` |
| `core_enhanced/src/game/world/combat.rs` | 620–645,714–796,3242–3255,3466–3523; profile/mode 검색 | `bf18e05534bfdfe9218b96d551d1ff3659286e7e099578935d0fe7de54dd80f0` |
| `core_enhanced/src/game/world/active_run.rs` | 15–41,143–174,203–212; seed 검색 | `b1c6050cec18630b254a6559a443d1e4a7c84b25cf34e9fca61c1aef8cfdd948` |
| `core_enhanced/src/game/world/active_run/continuation.rs` | 763–845; seed 검색 | `5558f49c6f7a7c2f8e7f761aaf9d526918dd32d83d262a287613a116d315f06b` |
| `core_enhanced/src/game/world/application.rs` | 53–78,126–167,167–225,379–408; owner/start 검색 | `e76032b69779d220ff5cd97fd6dcae4c45690be8cbc0d05baa5fa8354463a613` |
| `core_enhanced/src/game/recruitment.rs` | 1–180 | `647beeb970d657d699247a0bdb5279d3d488f8d3bd936cca70ced305b18f6245` |
| `core_enhanced/src/game/managers/uuid_manager.rs` | 1–180 | `2802db75245dda76b6472d87f748415809ac72a2d0938b69ad06e8db90b7395f` |
| `core_enhanced/src/game/boss_omen.rs` | 331–350; mode guard 검색 | `5ce03277c5d56eb5e4d9d615cff4d3fa2f671bb0e962c552b600e1c96f65d7af` |
| `core_enhanced/src/game/encounter_compendium.rs` | 12–87; profile/merge 검색 | `a340cd035b775a4c56ea6cbcfb3900098c9d3147da94993521788d4571fbde4c` |
| `core_enhanced/src/game/employee.rs` | 16–98 | `c98ffac82213d35e9ed7230e37583474b7876283a5c6c767b4871c90bf191bea` |
| `core_enhanced/src/application.rs` | 954–1011,1182–1263; public API 검색 | `917c4afd7a3cea97b2ecaebe5891a936fb7b65fb9a5010080c9a62bb232d2e9d` |
| `core_enhanced/src/mod_validation/host.rs` | 1–143 | `546efcc2d6d6b75321642f6dc8100e67497de2e60b571fd3ac3e3b1ce9aa8b76` |
| `core_enhanced/src/mod_validation/identity.rs` | 1–97 | `538722d49137f5bfc7dfe0a340b755ac01d96b257b8a10a9b8df2f13d55d1e63` |
| `core_enhanced/src/mod_validation/candidate.rs` | 90–275 | `00580345e1ce68fc544f6cec80aa21a90376b79570812630f44de825b976ed9d` |
| `core_enhanced/src/mod_validation/schema.rs` | 210–285 | `9d668e11bf6a3d8e352d4e7718a09dc2d7b93533b0d9310234ffaee9f01fb9b4` |
| `core_enhanced/src/mod_validation/issuer.rs` | 90–150,280–315,463–535 | `ef7f21b03258fee433b334f5f8065efb683f362f68ae0b6580667343a8b78c55` |
| `core_enhanced/src/mod_validation/cache.rs` | 1–84,110–172 | `a6372c2fac5aff4b4593716840df3bbca4fda38913dece394df6bc9677f1c725` |
| `core_enhanced/src/mod_validation/lab.rs` | 22–96,293–343,800–846 | `4d6542e2c88cf4cc9e22586b09e0b5450b61e3236001a145dc8c1f4dee6cc7bf` |
| `core_enhanced/src/authoring/trial.rs` | 577–618; preparation 검색 | `22d2bb941c1900350cf9e071a37f738aee80ee4be54f718ecfa85eb0df99ce64` |
| `core_enhanced/src/authoring/lab_preparation.rs` | 1–125 | `3feb9fd9d3f7543eb6344e7cee5f5e188ef8f3c89e684cf5c5af0c6ac0bc525f` |
| `core_enhanced/build.rs` | 1–106,98–177 | `691768cdb33b1de97ed98f957f9c0454284531f12e9b974da8efe9e1a410d1e7` |
| `core_enhanced/src/bin/consume_content_pack.rs` | 1–107 | `e9aa82e8683537d3172a815b139c2ce8c7eedc527a0c815bc10ac7542f5ace53` |
| `game_server/src/bootstrap.rs` | 1–82 | `582f02e7342922be1dabc2f60c1fd1fedea0bd974b2a831cefd80a7e89cc7fe4` |
| `game_server/src/main.rs` | 48–102 | `5dca52be8a094d0a2b5e24d4c2d66fc338f363e758b997944deb85dd918be112` |
| `game_server/src/bin/mod_launcher.rs` | 1–142 | `1995fd10661d021906769062638c4fc042995652c6559640631785911d51d826` |
| `game_server/src/mod_admission/session.rs` | 1–230,330–472 | `1ebf74c8b266841821e167b29bcd43ecbd40ca9335720e8cbf586b5e14d29b3d` |
| `game_server/src/mod_admission/persistence.rs` | 1–115 | `76bed5aa6e0ad5c0749fefbd3371731ff21e35938f9637c5821a8b0384f49f89` |
| `game_server/src/persistence/mod_requirements.rs` | 1–247 | `6972e2c7847bdb69f9a1069f6840a7c652296b8e95c2b1de1937d1bcad962250` |
| `game_server/src/game/player_game_actor/handlers.rs` | 295–392,3953–3971; normal publication 검색 | `27cb9826b432b6b720cbc18fd20def81ea85a13a458e45c312c2deb96d7f4a74` |
| `core_enhanced/docs/goals/core_mod_validation_apply/execution/single_goal/test_parts/runtime001/probe/src/normal_preparation.rs` | 64–109 | `6f005146b2f54635575ccd0f7ff70b5ddee9fcf2a7a5211ff085d971fdeed35e` |
| `core_enhanced/docs/goals/core_mod_validation_apply/execution/single_goal/test_parts/runtime001/prepare_public_entry.py` | 67–86 | `5698d575705673039fa56142ac87815436dcb4f2355d00143302ae1d1021872c` |

보고 저장 후 currentness 재대조: 65개, 불일치0개. 해시는 원본 전체 bytes이고 의미 검토 범위는 위 선택으로 한정된다.
