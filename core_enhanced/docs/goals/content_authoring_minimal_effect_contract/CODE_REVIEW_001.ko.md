# CA01 최소 작성·실행 지원 계약 — 코드 검토 본문

2026-09-09 · NEXT-0003 · P2G-20의 immutable 검토 본문. 연결 artifact가 이 파일의 SHA-256과 절을 선택한다. 정책 원본은 [P2G-10](P2G_10_POLICY_001.json), 조사 requirement는 [Source Plan](P2G_20_SOURCE_PLAN_001.json)이다.

## 1. 검토 의미와 원문 상태

검토 대상은 효과 주소·태그/property 작성 정보·unsupported 진단과 기존 상수 Damage/Heal/Instant 경로의 최소 연결이다. 콘텐츠 전체의 정책 동결·CA01 전체 DG·구현·실행 시험·GEX 감사가 아니다. `code_reviewed`는 아래 current actual, delta, 소비 경계와 검증 지점의 검토 상태다.

E01–06은 기존 사용자 결정 기록의 공통 조합, 엔진 작성/검증 기반, 효과 태그 판별, 새 효과의 태그·출처, 최신 규격 전환, 외부 코드 금지를 인수한다. R/F 콘텐츠 목록·태그 이름·수치·CA01/A02의 자료구조 제안은 승인 근거로 승격하지 않았다. 과거 대화 원본의 독립 획득은 없으며 Decision의 exact reference는 저장소에 명시된 사용자 결정 기록이다.

COV-R016/R020의 `합의`라는 조사 라벨만으로 원문의 기초 태그 **초안**을 확정할 수 없다. 같은 효과의 조건과 서로 다른 효과의 존재 조건을 구분하는 사례는 E03의 대상 판별을 점검하는 기술적 시험 후보로 사용한다. 현재 구현에 그 selector가 있다는 뜻도 아니다. F135–138은 피해/자기 회복을 분리할 표현력 사례로만 사용한다. 수도승의 공식 최대 HP 식·accepted hit·흡수 인정·개화/침식 의미는 이 검토에서 결정하지 않는다.

## 2. Current Actual — 생산자부터 실행 결과까지

| 경계 | 관찰한 실제 호출·소유 | 결과와 제한 |
| --- | --- | --- |
| B01 작성/공개 | `live_ron::GameplayRonSourceSet::embedded` → `parse_gameplay_ron_sources` → `RawSkillDatabase::try_into_unvalidated_database` → `into_validated` → `GameDataBase::load_live_sources`의 generated preview 검증 → `ValidatedGameData` → `CoreApplication::new` | manifest의 정확한 schema version을 검사하고 모든 검증 성공 후 token을 반환한다. 현재 source set은 embedded이며 외부 팩/재로딩/세대 pin을 제공한다는 claim은 하지 않는다. |
| B02 작성 정의 | `skill_data::RawSkillDef/RawSkillStepDef` → `ability::SkillDef/SkillStepDef` | range preset을 해석한다. skill/step ID는 있으나 `effects: Vec<SkillEffectDef>`의 각 효과에는 stable ID·태그·property descriptor가 없다. step target/delivery/when/repeat/presentation을 그 안의 효과가 공유한다. |
| B03 검증/오류 | `validate_skill_contracts`, `validate_skill_buff_references`, `live_domain_error`, `parse_live_ron` | 중복 step, 음수 Damage, Damage 없는 ModifyDamage, target/delivery 및 buff 참조 오류를 잡는다. `LiveRonError`는 source_file/domain/object_id/field_path/cause를 갖는다. raw range 해석 실패는 문자열로 전환되며 object/path가 빠질 수 있다. 효과를 내린 뒤에도 역추적할 compiled SourceMap은 없다. |
| B04 시전/실행 | `invoke_ability_with_accepted_target_tile`가 DB의 skill을 읽고 active cast와 step event를 만든다. `BattleEvent::SkillStep` 처리 → `resolve_skill_step` → `execute_skill_step` → Instant target 해석 → `build_skill_step_commands` → `process_commands_with_modifier_source` | 같은 step의 효과는 같은 targets를 사용한다. command 벡터는 효과 순서로 만들어진다. `ModifyDamage`는 step 안에서 먼저 모아 모든 Damage에 적용한다. 별도 self-target step이 있어야 피해와 자기 회복의 대상을 분리할 수 있다. |
| B05 결과/수명 | `ApplyDamage` → `execute_committed_damage` → `calculate_damage`/HP 기록. `ApplyHeal` → `apply_hp_delta_and_record`. `finalize_skill_step`/`previous_step_damage_gate`가 후속 step을 해제 | `actual_damage_target_count`는 HP 감소가 있을 때 증가한다. 결과가 terminal이 아니면 gate는 Pending, 피해가 없고 terminal이면 Unsatisfied다. 이는 보편적인 accepted-hit receipt가 아니다. 시전 종료 시 active state를 제거하고 철수 시 해당 cast를 취소한다. |
| B06 출처 | `DamageSourceSnapshot`의 unit/source enum/roll identity, `BattleEventCause`의 parent seq, 장비 `TriggerAbility`의 activation_source/binding_index → proc event → invoke_ability | 이미 출처의 일부가 있으나 효과 정의 ID·효과별 원형/부여 출처·영구 유물 사본·연쇄 경로를 모두 표현하지는 않는다. 이것을 그대로 최종 SourceBinding이라고 부르지 않는다. |
| B07 다른 현재 op | `build_skill_step_commands`의 stats/buff/resonance/interrupt/extra attack, `record_skill_step_live_signals`의 tactical resource | 빈 match만으로 ModifyTacticalResource나 ModifyDamage를 미지원으로 판정하면 틀린다. 전자는 signal, 후자는 step modifier로 소비된다. 현재 op들을 미래 미구현 op와 구분한다. |

현재 Heal의 `amount: i32`는 signed HP delta로 전달된다. 음수이면 HP가 줄며 Damage와 같은 경로가 아니다. 이 actual을 새 ‘회복’ 정책으로 승인하거나 임의로 금지하지 않는다. 최소 정상 fixture는 양수 Heal을 사용하고 음수 처리 변경은 별도 의미 검토 없이 넣지 않는다. 태그 이름이 `회복`이라는 이유로 executor의 모든 signed 입력을 회복으로 설명해서도 안 된다.

## 3. 직접 소비자·저장·실제 선행

| 경계/관계 | 확인한 소비 지점 | 최소 계약에서의 처리 |
| --- | --- | --- |
| 파편 → skill, 기능 필수 | `SkillFragmentLoadout::apply_to_profile`가 종류별 progress로 imitation/upgrade/awakened SkillId를 고르고 profile에 넣는다. employee → stat_pipeline → battle build/runtime가 이를 소비한다. `validation::validate_skill_fragment_skill_references`가 참조를 검사 | 기존 SkillId 조회 seam을 재사용한다. 효과 정의 주소는 사본 소유 ID가 아니다. CA10 사본화 전체가 첫 metadata/Instant slice의 필수 선행은 아니다. |
| 장비 → skill, 기능 필수 | equipment activation 검증 → `dispatch_item_trigger_occurrence` → `activation_commands_from_bindings` → `TriggerAbility` → `invoke_ability` | skill 내부 표현을 바꿀 때 이 경로와 proc 출처를 함께 연결한다. 효과 주소를 추가했다고 기존 OnAttack/OnHit/OnKill 분류를 바꾸지 않는다. |
| range preview, 동시 전환 | `range_preview::active_skill_preview`가 `get_by_id`, cast targeting/selection을 읽는다 | compiler가 실행 정의를 바꾸면 이 읽기 seam도 같은 publication에서 일치해야 한다. 서버/Unity 구현 성공은 별도 비범위다. |
| projectile/area, 직접 공유 소비 | 두 delivery의 hit helper도 `build_skill_step_commands`를 호출한다 | 기존 carrier와 효과 순서·타격 수를 유지한다. 최소 Instant 지원을 추가하려고 projectile을 복제하거나 다른 spatial 동작을 바꾸지 않는다. 공통 effect wrapper를 교체하면 두 호출자의 metadata 전달과 기존 동작 회귀를 함께 확인해야 한다. |
| 보스, 후속 소비 | raw/compiled boss task에는 Route/Acquire/Move/Rejoin/Wait/Summon/EmitSignal이 있고 명시 skill invocation task는 없다. future Repath는 진단으로 거부된다. 소환 정의의 skill 참조는 기존 검증에 연결 | K04/K09 전체 graph나 ActionHandle의 구현을 현재 선행으로 요구하지 않는다. CA16은 향후 skill 실행 handle의 완료/취소 의미를 별도 검토해야 한다. |
| 저장, 조건부 동시 전환 | `RunCheckpointPayload::capture/restore_into_state`, `BattleEntrySnapshot::capture/restore`가 inventory/roster/fragments를 소유한다. active-run writer는 V5, reader는 V4/V5. hash는 checkpoint payload의 무결성 | 정의 metadata/compiled index 자체는 이 checkpoint의 필드가 아니다. 기존 SkillId와 gameplay 상태를 유지하는 최소 주소 추가에는 새 영구 상태가 필요 없다. 사본 ID·content generation·형태 선택 상태를 실제 도입하면 같은 owner가 capture/restore/validation을 동시 전환해야 한다. |
| 기존 검증, 검증 선행 | CRAR Master Attempt 3 `Complete / High`와 NEXT-0001의 target 해시 대조 기록을 인수. 이번 request baseline의 기존 비교 텍스트가 review 종료 전 모두 일치 | 과거 1,159 PASS를 이번 시험으로 계산하지 않는다. 기존 publication/command seam을 다시 만드는 작업은 필요 없다. 신규 descriptor·compiled source linkage의 실행 증거는 아직 없다. |

U04는 새 규격 도입을 위해 구형 별칭/이중 실행/자동 저장 변환을 추가하지 않도록 한다. 현재 V4 reader의 존재는 actual이며 그 제거를 이번 metadata 작업에 끼워 넣지 않는다. 저장 형식을 바꾸지 않는 최소 slice와 향후 incompatible 저장 cutover를 구분한다. 기존 개인 저장 파일은 읽거나 삭제하지 않았다. 파일 삭제·자동 변환을 해결책으로 제안하지 않는다.

## 4. Implementation Delta와 Proposed Implementation

아래는 승인 의미를 보존하는 **R/L 제안**이다. 타입명·RON 문법·API를 구현된 계약이나 사용자 확정 사항으로 표시하지 않는다.

### L01 책임·데이터·주소

`skill_data`의 raw→validated publication 사이에서 작성 효과의 identity/tag/operation을 검증하고, 기존 실행 정의와 읽기 전용 metadata/index를 한 publication에 묶는다. 작성 효과는 정의/형태 안에서 안정적으로 식별하고 compiled index는 publication 내부 주소로만 쓴다. index나 벡터 위치를 영구 사본 ID로 저장하지 않는다. 기존 `SkillId`를 정의 조회 seam으로 재사용할 수 있다.

EffectSite는 **어느 작성 효과인가**, ActionSite는 그 효과들이 공유하는 **target/delivery/flow owner가 어디인가**를 식별한다. 하나의 source locator에는 source 논리 경로, 정의 ID, step/action ID, effect ID, 원문 field 위치를 남긴다. 자료형 이름과 내부 container는 구현자가 정할 수 있지만 작성 주소와 runtime 발생/소유 사본을 같은 key로 합치지 않는다.

원형 정의의 출처는 compiler가 부여하고, 실제 시전에는 기존 caster/cast seq 및 적용 원천을 묶는 별도 실행 문맥을 전달한다. 장비 proc event의 출처를 잃지 않는다. 새 유물 효과의 생성/사본/연쇄 상태가 아직 없으므로 해당 런타임까지 완성됐다고 하지 않는다. CA11–13은 이 주소 계약을 소비하며 자기 수명·생성 owner를 동반해야 한다.

### L02 지원 정보·타입·실제 실행

descriptor는 consumer, operation, typed input/property, 단위, 변경 가능한 범위, 지원 조건과 diagnostic 위치를 묶는다. 첫 검증 대상은 현재의 literal Damage(amount, damage_type), literal Heal(amount), Instant 전달이다. Damage는 기존 음수 거부와 0 처리를 유지하고 Heal은 현재 signed 입력 의미를 숨기지 않는다. 최소 fixture에서만 양수 Heal을 사용한다. 공식 수치나 새로운 HP 계산식을 만들지 않는다.

지원 정보는 실제 validator와 executor 등록에 연결해야 한다. 목록만 노출하고 runtime은 빈 성공으로 처리하는 방식은 금지한다. 기존 다른 op의 actual 지원은 B07과 별도로 보존한다. 첫 slice 검토를 이유로 현재 spatial/다른 op를 삭제하거나 그것들의 전체 변형까지 지원한다고 광고하지 않는다. 이후 구현자가 새 공개 지원 목록을 내기 전에 해당 consumer 조합의 완전성을 확인해야 한다.

SourceMap은 raw 검증과 lowering 양쪽에서 원문 주소로 돌아갈 수 있게 한다. 기존 `LiveRonError`의 구조화된 위치와 원인을 재사용하되 raw→resolved 문자열 변환에서 위치를 잃는 지점을 보완한다. byte span을 제공할지 field locator로 시작할지는 기술 선택이다. 정확한 줄 번호를 알 수 없으면 만들어 내지 않는다.

### L03 효과/공유 carrier와 최소 실행 연결

태그는 effect별 metadata이고 opcode 지원 권한이 아니다. 같은 effect에 대한 조건의 AND는 그 effect에서 평가하며, 서로 다른 effect의 존재는 별도 query 구조로 표현한다. 첫 metadata 계약은 이 둘을 표현·검증할 주소를 제공하고 유물 활성 closure/변형 전체를 실행하는 계약으로 확대하지 않는다.

effect amount처럼 독립 payload인 속성은 해당 효과에 한정한다. target/delivery/반경처럼 action carrier가 소유하는 값은 effect 전용 scalar로 노출하지 않는다. carrier를 공유하는 다른 효과까지 영향을 받는 요청은 관련 EffectSite/ActionSite를 diagnostic에 표시해야 한다. 아직 승인·지원하지 않은 변형은 진단 대상으로 남긴다. 투사체 복제나 타격 수 변경으로 조용히 성공시키는 대안은 채택하지 않는다.

최소 positive 구조는 서로 다른 action/step에 놓인 Damage와 Self Heal이다. 기존 step scheduling과 target 해석을 재사용하고 metadata를 제거한 실행 결과가 기존 op 경로와 같아야 한다. 아직 없는 일반 Branch/Parallel/Repeat VM을 먼저 만들 필요는 없다. F135의 accepted-hit/최대 HP 식을 이 구조만으로 충족했다고 하지 않는다.

### L04 순서·실패·취소·전환

parse → 주소/타입/참조/consumer 지원 검사 → 실행 정의 및 SourceMap 구성 → 기존 domain/cross-reference/generated preview 검증 → 하나의 `ValidatedGameData` 공개 순서를 제안한다. 실패하면 candidate publication을 반환하지 않는다. 게임 state 변경이나 부분 live 교체를 이 compiler가 소유하지 않는다.

런타임은 현재 effect 순서, step-level ModifyDamage 합성, damage/HP mutation 및 후속 event 경로를 보존한다. compiler index를 써도 cast_seq/step 결과의 Pending·terminal·철수 취소를 유지한다. 별도 공유 carrier를 새로 만들거나 비동기 ActionHandle을 도입하여 cancellation 의미를 바꾸는 작업은 현재 최소 설계가 아니다.

raw wrapper나 필수 identity를 도입하면 최신 parser, embedded manifest/RON, runtime 소비자, range preview와 fixture를 동시에 맞춘다. 새 문법을 성공 처리하고 구형 executor로 의미를 버리는 중간 상태는 허용하지 않는다. old/new authoring grammar의 장기 병렬 지원은 만들지 않는다. 정확한 Atomic Group·중간 상태·실행 순서는 P2G-40이 이 경계를 근거로 정한다.

### L05 대안과 후속 상세 선택

기존 vec index만 효과 ID로 쓰면 재정렬에 약하고 작성 출처와 실행 주소가 섞인다. 문자열 property 경로를 임의 평가하면 타입/지원 보장과 E06에 맞지 않는다. 전체 범용 VM을 먼저 만드는 안은 현재 Damage/Heal 소비 경로에 불필요한 의존을 추가한다. 따라서 현재 domain executor에 연결되는 typed 작성 metadata와 좁은 lowering을 제안한다.

구현자는 정확한 Rust 타입/파일, duplicate key의 자료구조, field locator encoding, source metadata 전달 방법을 결정한다. 공개 의미·출처·취소·타격 수·저장 수명이 바뀌면 P2G-10으로 반환한다. 최신 schema의 exact RON 예시와 새 parser 검증, 모든 직접 call site의 컴파일·회귀는 구현 owner의 완료 조건이며 이번 문서가 대신하지 않는다.

## 5. 검증 seam과 미실행 recipe

| recipe | 실제 seam과 기대 관찰 | 이번 상태 |
| --- | --- | --- |
| V01 상수 피해→자기 회복 | `skill_runtime_contract::hit_gated_self_heal_and_buff_stack_repeat_attack_work_together`와 `run_battle` helper는 BattleCore를 돌려 자기 회복 1회·후속 반복 3회를 assert한다. 신규 wrapper fixture는 effect 출처까지 대조해야 한다 | 소스만 확인; 실행 0 |
| V02 작성 오류 | `skill_data`의 duplicate step/negative Damage/dangling ModifyDamage fixture; `live_ron::bundle_validation_returns_typed_skill_contract_errors`의 source/object/path assert에 효과 주소·unsupported 사례 추가 | 기존 시험 소스 확인; 신규 시험 작성/실행 0 |
| V03 효과/대상 분리 | 서로 다른 두 effect의 태그를 하나의 effect AND로 합치지 않음. 피해 scalar를 바꿔도 self Heal payload/대상이 변하지 않음. 공유 carrier 변경은 영향 주소가 있는 진단 | 신규 recipe 제안; parser 지원·PASS 주장 없음 |
| V04 공개 원자성/목록 일치 | 유효 집합은 한 publication, invalid op/reference는 token 미반환. 공개 descriptor의 각 지원 조합은 실제 compiler/executor에서 같은 결과 | 정적 seam 확인; 신규 실행 0 |
| V05 직접 소비 회귀 | live catalog reference ownership, 장비 activation, fragment SkillId selection, range preview, projectile/area의 공유 command 호출; cast 종료/철수 후 늦은 step 처리 보존 | 읽은 경계까지만 확인; 통합/게임 테스트 미실행 |
| V06 저장/권한 | 최소 metadata가 checkpoint에 새 영구 key를 만들지 않음. 향후 상태 추가 시 capture/restore/거부 실패의 원자성 시험. 외부 스크립트/임의 property 실행 경로 없음 | 설계/정적 검사; 개인 저장 파일·OS/네트워크 실행 시험 없음 |

이번 자동 검증은 문서 payload의 typed reference, current selection, decision/acceptance 보존, plan↔observation/claim↔binding coverage와 source 해시 검사다. 게임 구현이나 성능·배포 검증을 뜻하지 않는다. 기존 P2G-40/50 validator는 현재 단계용 도구가 아니므로 실행하지 않았다.

## 6. Finding 분류·미확인 범위·종료

| ID | Primary classification | 발견 사항과 처리 |
| --- | --- | --- |
| F20-01 | implementation_finding | effect stable ID/tag/descriptor/compiled SourceMap 부재. B01–03/L01–02의 delta이며 구현 완료로 표시하지 않음 |
| F20-02 | technical_decision | 기존 typed executor와 opaque publication을 재사용하고 effect 주소/carrier owner를 구분. 정책 의미·타격 수·기존 순서를 보존하는 L01–04 제안 |
| F20-03 | implementation_finding | HP-loss gate와 signed Heal을 관찰. accepted-hit/공식 회복 정책으로 재해석하지 않고 기존 실행 의미 보존. 해당 의미를 바꾸려는 후속 작업만 PG03/04·P2G-10으로 반환 |
| F20-04 | implementation_finding | skill/장비/파편/preview/공유 spatial helper는 직접 소비자. 문법·공통 wrapper 전환 시 동시 연결 필요; P2G-40의 Atomic Group 입력 |
| F20-05 | technical_decision | 정의 metadata와 영구 사본·generation state를 분리. 최소 slice에는 새 저장 상태가 필요 없으며 새 상태 도입 owner가 저장/복원을 동반 |
| F20-06 | out_of_scope | 전체 외부 팩, 사본 개화/유물 closure/연쇄, 보스 invocation graph, 공식 콘텐츠 밸런스·고급 변형은 각 CA/PG owner. 현재 최소 인터페이스 검토에 이들의 완성 구현은 필수 아님 |
| F20-07 | out_of_scope | release/deploy/서버/Unity compatibility는 선택 scope에 없음. source plan RQ07은 N/A로 평가하며 실제 환경은 미확인 |

필수 workspace source는 접근·식별했고 baseline과 일치한다. 이 경계 내에서 새 사용자 의미 선택이나 해결 불가능한 필수 source gap은 발견하지 않았다. 미래 기능과 미실행 시험은 pass로 처리하지 않는다. 기존 정책·acceptance를 변경하지 않고 exact result binding의 완전성이 검증될 때만 P2G-20 결과로 선택한다.

선택 요청의 종료는 P2G-20 결과 기록까지다. P2G-30 Freeze, Phase Plan, Goal Set, Handoff, GEX 실행 및 CA01 전체 구현은 남아 있다. 새로운 다음 작업은 이번 요청에서 실행하지 않는다.
