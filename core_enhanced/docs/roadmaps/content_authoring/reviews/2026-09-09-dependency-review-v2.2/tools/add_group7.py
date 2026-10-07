from record_sources import *
p='core_enhanced/'
items=[
('docs/goals/content_authoring_minimal_effect_contract/P2G_30_FREEZE_001.json',[(1,260),(330,565),(588,705)],['FPF-CAMEC-P2G30-001','TR30-02'],'formal'),
('docs/goals/content_authoring_minimal_effect_contract/P2G_60_HANDOFF_001.json',[(1,213)],['HOF-CAMEC-P2G60-001','Implementation Authorized: No'],'formal'),
('src/game/data/live_ron.rs',[(1427,1586)],['parse_gameplay_ron_sources','validate_gameplay_source_inventory'],'source'),
('src/game/data/validation.rs',[(118,270)],['validate_pve_reward_source_references','validate_ordinary_reward_source_exclusion'],'source'),
('Cargo.toml',[(1,52)],['autobins = false','event_log_checks = []'],'source'),
('tests/ron_loading.rs',[(1,185)],['live_abnormality_ron_only_contains_catalog_roster'],'test_source_not_executed'),
('tests/skill_runtime_contract.rs',[(1,150)],['fn battle_scenario','fn spawn_group'],'test_source_not_executed'),
('src/game/battle/recording.rs',[(1,91)],['record_event_log','result_stats_accumulator.observe(&entry);'],'source'),
('src/game/battle/event_log.rs',[(320,580)],['pub struct BattleEventLog','fn append_with_encoded_len'],'source'),
('src/game/encounter_progression.rs',[(115,285)],['plan_encounter_reward','commit_reward_plan'],'source'),
('src/game/data/pve_data.rs',[(550,700)],['pub struct PveEncounterRewardSource','a fragment may belong to exactly one reward source'],'source'),
('docs/skills/abnormality_skill_fragments_design_notes.md',[(1,210)],['최신 기록을 목표 정책으로 사용한다.'],'policy_source'),
('docs/skills/run_relic_content_design_notes.ko.md',[(266,400)],['### 효과 단위로 분해하는 원칙'],'policy_source'),
('docs/game_rulebook.md',[(545,590)],['### Pause와 직렬 전투 명령'],'policy_source'),
('docs/roadmaps/content_authoring/records/CA00_NEXT-0002_COVERAGE.json',[(1,245),(37065,37127)],['COV-R-001','BOSS-COV-127'],'inherited_baseline_partial_read'),
]
for x in items:add(p+x[0],x[1],x[2],x[3])
add('game_resources/data/skills/base.ron',[(1,185)],['id: "unknown_distortion_strike",','id: "little_red_hunt_the_prey",'],'actual_ron_partial')
for name,last in [('01_SYSTEM',90),('02_AUTHORING_AND_COMPILER',88),('03_RUNTIME',119),('04_COMPOSITION',87),('05_BOSS_AUTHORING',137),('06_PACKS_AND_TOOLS',69),('07_CUTOVER_AND_REUSE',56),('08_WORKED_EXAMPLES',130)]:
 add(p+'docs/roadmaps/content_authoring/roadmap/architecture/'+name+'.ko.md',[(1,last)],[], 'design_source_full_read')
add(p+'docs/tmp/ego_corrosion_system_design_notes_2026-08-19.ko.md',[(30,165)],['최종 보스 파편은 제외'])
with (ROOT/'evidence/RESEARCH_LOG.ko.md').open('a') as f:
 f.write('''\n## 조사 묶음 7 — formal 한계·architecture·실제 RON·등록/관찰\n\nFreeze와 Handoff 본문에서 최소 계약이 신규 영구 사본/generation 및 저장 형식 변경을 제외하고, 기존 signed Heal·순서·수명·reader를 보존함을 확인했다. 초기 verdict A를 지지한다. 현재 세부 contract를 다시 발행하거나 GEX를 실행하지 않았다.\n\nA01–A08을 전체 읽고 논리 설계·K01–10·typed recipe·source binding·Action milestone·fault boundary를 보존했다. 기존 A07/CA19/CA22도 저장과 예산을 각 기능 도입부터 요구한다. 이를 기존 로드맵의 미발견 정책처럼 비판하지 않는다. 새 개정은 현재 코드로 구체 owner/원자 단위를 정한 것이다.\n\n실제 RON 표본은 기존 SkillDef/steps/effects 직접 구조다. Cargo는 autobins=false와 4개의 test-fixtures 전용 fixture generator이며 공개 authoring CLI는 여기에 없다. ron_loading의 fixed roster는 공식 catalog 회귀 검사로 남길 수 있으며 모든 외부팩의 허용목록으로 쓰면 안 된다. 시험 소스의 존재는 이번 실행 PASS가 아니다.\n\nrecording은 stats accumulator를 bounded diagnostic log append 앞에서 관찰한다. log Parent seq만으로 새 chain 권한을 만들면 안 된다. sequence exhaustion/retention과 게임 인과는 별도 계약으로 인수한다.\n\nencounter_progression의 첫 완료는 source.fragment_id 하나를 GrantSkillFragment로 만들며 pve reward source의 uniqueness와 abnormality source fragment 일치 검증이 연결된다. 복수 파편/결과 선택은 R19에서 이 실제 plan/commit/validator와 함께 바꾼다.\n\n침식은 최종 보스 *파편* 제외이며 보스 전투 자체 금지가 아니다. 옛 파편 문서의 세트4종/별도입력/효과범위 open 항목은 최신 침식 정책이 우선한다고 원문 자체가 지정한다. 룰북은 Pause 중 일반 cast 시작을 즉시 반영하므로 침식의 실제 시전과 admission/release를 의도 없이 동일시하지 않는다.\n\n현재 로컬 GOALS.json에는 R00–R24와 기존 모든 49개 edge disposition이 기록됐다. 단위 전체가 아닌 명시 산출물 의존이고 기본 순차 진행과 구별된다. 보고서·카드·정합성 검사는 이어서 작성한다.\n''')
