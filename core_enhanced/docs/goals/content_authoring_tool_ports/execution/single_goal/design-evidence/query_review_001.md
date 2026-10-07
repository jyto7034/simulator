# R03 query narrow source review 001

읽기 검토 결과: 최초 입력에서 구체적 결함 3건을 확인했다. QR01은 검토 중 root가 수정했고 실제 successor guard를 읽어 source 수준 교정을 확인했다. QR02·QR03은 확인 시점의 구현에 남아 있었다. 이 문서는 INPUTS023/024 테스트 결과나 완료 감사가 아니다. source/tests/docs 변경·test/build 실행 없이 지정한 새 `/tmp` 보고서 두 개만 작성했다.

## QR02 — Effect 검색의 반복 문자열 비교 비용 누락 (medium)

근거는 최초 query.rs172–201 및 검토 중 successor `ba0cdc...`의 181–207이다. explain_request는 query를 한 번 serialize하여 비용을 부과하지만, 이후 `skill.steps.iter().find(|step| step.id == site.action.action_id)`와 effects find 직전 charge는 각각 `steps.len()+1`, `effects.len()+1`뿐이다. 반면 같은 함수의 Definition 검색은 `(writers.len()+definitions.len()+1)*(key.id.len()+1)`으로 반복 key 비교까지 보수적으로 가격을 매긴다.

Skill validation은 action/effect ID의 nonempty/uniqueness를 검사하지만 ID 길이 cap을 두지 않는다(skill_data.rs686–707). 따라서 공통 긴 prefix를 가진 N개의 서로 다른 ID와 동일 길이의 부재 ID를 조회하면 반복 String equality는 O(N·L) bytes를 비교하지만 추가 work는 O(N)이다. 부재 action/effect는 곧바로 Request error여서 result serialization에서도 이 비용이 보전되지 않는다. 예컨대 수백 개의 8 KiB action ID는 기존 file byte 상한 안에서도 수 MiB의 비교를 만들 수 있다. 실제 adversarial fixture를 실행한 결과는 아니며, source상 가격 계산 누락이다. OS 전체 heap·정확 CPU 계측 문제로 확대하지 않는다.

최소 수정: 두 find 전에 queried ID의 byte 길이를 곱한 보수적 checked/saturating charge를 적용하거나 각 비교 전에 비교 상한을 charge한다. Effect의 owner `metadata.source_for(&key)`도 owner ID 길이를 포함해 가격을 매긴다. 특히 Added effect에서는 origin ID가 짧아 origin_source의 기존 charge가 긴 owner key 검색 비용을 대신하지 못한다. 적절한 검증은 긴 공통-prefix IDs의 missing action/effect 조회가 비교 loop 진입 전에 work limit로 거부되는 경계다.

## QR03 — query의 invalid-limit 오류가 Acquisition phase로 표시됨 (low)

control.rs98–148 `ExecutionLimits::validate`는 invalid 값에서 `phase: ExecutionPhase::Acquisition`을 고정한다(137–144). run은 requested phase를 받지만 178에서 `limits.validate()?`를 먼저 호출한다. describe/explain은 Inspection scope(query.rs242 부근)를 요청해도 `max_result_bytes=0` 또는 hard ceiling 초과값이면 loader를 호출하지 않은 query를 Acquisition failure로 반환한다.

최소 수정: private limits validator에 requested phase를 넘긴다. 기존 content validate는 Acquisition을 그대로 쓰고 query는 Inspection으로 표시한다. `describe(valid_content, max_result_bytes=0)`의 code/site/limit/requested를 유지하면서 phase만 Inspection으로 확인하는 검증이면 충분하다. 이는 결과를 성공으로 바꾸는 문제가 아니라 실제 producer phase의 정확성 문제다.

## QR01 — CompiledGraph의 selected Remove를 graph 성공으로 반환 (검토 중 source 수정 확인)

최초 query SHA `1de308ee9f97326419110ba484bf6393084f561eeedfda9f560fc946966ef2a6`, query.rs126–161은 Definition/CompiledGraph를 같은 분기로 처리한다. BossPattern/Actor/Encounter kind gate를 통과한 selected Remove는 source=None이나 writers는 존재하므로 definition::document를 호출한다. definition.rs54–56은 `Ok`인 removed/present=false/value=null 문서를 반환한다. Definition Remove explanation으로는 맞지만 actual graph가 없는 CompiledGraph 요청의 성공은 부정확하다.

Root는 검토 중 source lookup 직후 CompiledGraph+source=None 거부를 추가했다고 알렸다. 실제 읽은 successor query SHA `ba0cdc267762b3b3da94d0834816e195ced35508d4f4442db93f36e1f405d3a3`의149–155는 `authoring_definition_not_found`를 반환한다. Definition Remove 성공 분기는 유지된다. successor tests SHA `d4daf4e7fe1503f32bf36b8960c5e6cddd8b93a6323324bc3b4ee0351b245548`의441–457,504–508,540–542에는 실제 added/removed BossPattern negative가 추가되어 있다. 이 변경의 source closure만 확인했으며 테스트 통과를 주장하지 않는다.

## 확인한 정합성과 finding으로 삼지 않은 사항

- Definition은 final source를 먼저 확인한다. source가 없고 writer만 있으면 removed 설명, writer도 없으면 not-found다. source가 있는데 DB typed record가 없으면 `authoring_definition_projection_missing`으로 실패한다. 내부 default catalog 값이 final-membership gate를 우회하는 경로는 해당 exhaustive match에서 찾지 못했다.
- final source/actual writer record를 그대로 serialize하고 nonwinning/Remove payload 또는 child authored 위치를 재구성하지 않는다. Effect origin은 현재 origin definition의 SourceTable 위치이며 과거 원본 payload라는 주장을 하지 않는다.
- DefinitionKind::ALL은 기존 definitions! macro에서 같은 variants로 생성된다(raw.rs143–155). current literal-edit descriptor는 기존 EFFECT_SUPPORT이며 whole runtime execution support와 혼동하지 않는다.
- ActorView의 PatternCall candidate는 pattern_id만, EncounterView의 binding은 behavior_id만 내보낸다(compiled.rs85–104,143–155). 해당 Arc의 whole target graph를 재귀 serialize하지 않는다. Pattern의 직접 Serialize leaf는 DefinitionId/compiled index/값으로 구성되고 cross-definition Arc가 없다(boss_runtime_data.rs520–640). Summoned compiled profile도 ID와 실제 profile을 내보낸다. borrowed wrappers는 전체 JSON Value/DB/graph를 먼저 복제하지 않는다.
- output.rs9–19는 모든 writer chunk의 byte admission과 work를 통과한 뒤에만 Vec에 append한다. 실패해도 query result를 게시하지 않는다. 최종 `String::from_utf8(Vec)`는 두 번째 전체 내용 복사를 하지 않는다. Vec capacity/allocator overhead가 logical encoded-byte cap와 정확히 같다는 약속은 없으므로 전체 heap proof 누락을 finding으로 삼지 않았다. writer references Vec 등 작은 임시 구조는 publication metadata/work 범위 안에 있으며 whole payload clone은 찾지 못했다.
- run_query는 control::run의 마지막 sticky failure를 먼저 `?`로 반환하고 그 뒤 body Request error를 검사한다. 따라서 output encoder가 중간 Request wrapping을 했어도 latched result/work/cancel failure가 우선한다. 사용량은 성공 결과 document 밖에 별도로 보유한다. 이 query 구현에는 receipt 자기 usage의 직렬화 순환이 없다.
- Query는 immutable token만 받고 파일을 다시 열거나 gameplay를 호출하지 않는다. indexed lookup의 OnceLock은 production publication이 validate_indexes를 미리 수행한다(data/mod.rs940–961). 반환 String은 generation lifetime을 추가 보유하지 않는다.
- EquipmentMetadata.triggered_effects는 기존 HashMap이며 그대로 serialize된다(equipment_data.rs177–194, stats.rs165–166). 별도로 재로드한 동일 semantic content의 JSON object key **transport byte order**는 달라질 수 있다. 하지만 object의 semantic JSON equality 및 성공한 전체 encoded byte 길이/총 byte charge는 같으므로 현재 약속에 대해 이 자체를 결함으로 판단하지 않았다. 동일 불변 token의 반복 query는 같은 map order를 유지한다. 실패 시 first overflowing chunk의 requested 수치/first observed failure는 order의 영향을 받을 수 있으므로 canonical transport/failure-diagnostic byte equality까지 증명했다고 보고하면 안 된다. 현 source/PLAN은 그런 약속을 하지 않는다.

## 범위와 drift

주요 구현 파일의 첫 SHA를 읽기 시작에 캡처했다. 검토 중 root가 query/tests를 바꾸었고 위 QR01에 한해 명시 successor spans를 다시 읽었다. 마지막 hash가 추가 successor를 가리키면 그 전체 구현을 재검토한 것으로 간주하지 않는다. 관련 query tests를 source로 읽었지만 실행하지 않았다. 아직 없는 session/CLI, 전체 runtime leaf 또는 전체 heap proof는 검토 결함 목록에 넣지 않았다.

JSON manifest는 source spans, 최초/종료 SHA 및 drift를 담는다. 부수적으로 검색만 한 파일을 exhaustive review source로 나열하지 않았다.

| Source | Close SHA-256 | Capture |
| --- | --- | --- |
| `core_enhanced/src/authoring/query.rs` | `c77981195623d173986bd95b5fa904078a1f61fbfb9450a55087179f73b6c82a` | changed_since_initial_capture |
| `core_enhanced/src/authoring/definition.rs` | `4bc42035f377358f9a7a40173c1192634291aa9261e6ff9711db8a44ddfecc29` | unchanged_between_hashes |
| `core_enhanced/src/authoring/compiled.rs` | `c72e2bc6f9fe59319270a929d3f2f41dd50f2416b26e8c2ed44afc9df89734d3` | unchanged_between_hashes |
| `core_enhanced/src/authoring/output.rs` | `cc12ba475d6d68a84e620f7ec1b3300288f55fa208463a135c1036173db3fdf8` | unchanged_between_hashes |
| `core_enhanced/src/authoring/control.rs` | `39d5aa1869a16c2d62d4e74b64c6cac5fbd39553ecceca36b72f9cba1832d6c9` | changed_since_initial_capture |
| `core_enhanced/src/authoring/mod.rs` | `8f6c95a935adcac03b9516df0040f014703ce6d8584898540017490348fd8ed7` | unchanged_between_hashes |
| `core_enhanced/src/authoring/tests.rs` | `e2a625bb2a5ac7614fa36c110bf63ec7d5837d420e840fbee016337c281e73ab` | changed_since_initial_capture |
| `core_enhanced/src/game/data/content_pack/raw.rs` | `72c6fa4917831d76c87c0f9a0cd58c8be3c8e5342235c8ef01a80a94c94852e7` | unchanged_between_hashes |
| `core_enhanced/src/game/data/boss_runtime_data.rs` | `05b2fa00aa7aed65becf8dbc20da866b35d2dfe4dca4318f4b8cc8fc1013d4d3` | unchanged_between_hashes |
| `core_enhanced/src/game/data/summoned_unit_data.rs` | `a944d1dd4f5f562049900f35c07fc94a32b64ee97a05986fc77a7a7423bf61f1` | unchanged_between_hashes |
| `core_enhanced/src/game/data/skill_data.rs` | `8a1cccf346cc3b45755a92a45e41be92deb5a0e641a0f519977dc6a2c7c62fb5` | single_close_hash; no initial atomic snapshot |
| `core_enhanced/src/game/data/equipment_data.rs` | `a9a6b06fccd5c7bf78879b010531533a56f6c1264bd02989ae6d3d9de8173e1e` | single_close_hash; no initial atomic snapshot |
| `core_enhanced/src/game/stats.rs` | `9c3d49b44c773288da84e35bc0411292e423583a4b481a1e044622c138d10269` | single_close_hash; no initial atomic snapshot |
| `core_enhanced/src/game/battle/types.rs` | `e388e9eb7d882dbcbd81eb9291a3cf77f1b8841fc060201da7735c750666075f` | single_close_hash; no initial atomic snapshot |
| `core_enhanced/src/game/data/mod.rs` | `84d12f3cd0a9d3bbe7df5910ce4e83964fd3152a7409b1f0bbcb23b46cae4644` | single_close_hash; no initial atomic snapshot |
| `core_enhanced/docs/goals/content_authoring_tool_ports/execution/single_goal/PLAN.md` | `fa19e70c6ab98ce741235e106308569eb3a0698763ef19f5b673acb1250871e2` | single_close_hash; no initial atomic snapshot |
