# Documentation–Code Alignment Corrections Master Goal

Status: complete

Implementation complete: true

Created: 2026-07-18

Last updated: 2026-07-18

Goal type: orchestration master. 이 Goal은 production code, live RON, tests, durable canonical 또는
external Unity implementation을 직접 수정하지 않는다.

## Objective

`documentation_code_alignment_reaudit`에서 확인된 비-P1 결함과 2026-07-18에 사용자가 승인한
정책을 독립적인 correction child로 구현·검증하여, current code/live data/server/Unity/canonical이
다시 같은 내용을 말하게 한다.

## User-Visible Outcome

- 잘못된 live RON authoring은 nested field에서도 조용히 무시되지 않고 startup/loading 단계에서
  명시적으로 거부된다.
- battle setup의 abnormality preload 목록에는 실제 non-empty abnormality ID만 들어간다.
- Endless의 모든 response 완료 분기는 direct regression test로 보호된다.
- 직원의 기본 block 수치는 class나 armor가 아니라 장착 무기 RON에서 결정된다.
- 임시 fragment/range/WhiteNight 콘텐츠와 정식 현재 계약의 경계가 명확해진다.
- Maintenance는 per-item authored recipe만 실행하며 generic fallback을 만들지 않는다.
- `GrenadeLauncher`는 current runtime schema에서 제거되고 future concept 문서에만 남는다.
- 확정된 구현과 canonical enum/source-count/용어가 일치한다.

## Source Of Truth

현재 동작은 다음 순서로 확인한다.

1. actual runtime mutation/validation code
2. production-loaded live RON/data
3. typed Core DTO와 serialization
4. game_server mapping/message order
5. actual Unity reader/store/presenter
6. current durable canonical
7. 이 Master와 child goal history

승인 목표 정책은 이 PLAN의 `Approved Policy Decisions`가 작업 gate를 제공하지만, child 완료 뒤
장기 정책은 가장 가까운 durable canonical로 이동해야 한다. 이 Master 자체는 gameplay/data/wire
canonical이 아니다.

## Audit Inputs

- `docs/goals/documentation_code_alignment_reaudit/AUDIT_REPORT.md`
- `docs/goals/documentation_code_alignment_reaudit/EXPERIMENT_NOTES.md`
- `docs/goals/documentation_code_alignment_reaudit/EXPERIMENTS.md`

Stable finding ID는 원래 `DCA2-*` 값을 유지한다. Child가 finding을 다시 발견해도 새 ID를 만들거나
중복 primary ownership을 주장하지 않는다.

## Included Finding Registry

| Finding | Current disposition | Primary child | Completion requirement |
| --- | --- | --- | --- |
| `DCA2-DATA-002` | correction approved | G01 | transitive strict parsing + production-shaped negative tests |
| `DCA2-WIRE-003` | correction approved | G02 | actual non-empty catalog refs + regression/probe |
| `DCA2-TEST-001` | evidence hardening approved | G03 | exact Endless completion branch test |
| `DCA2-GAME-001` | policy decided | G04 | weapon-authored block contract end-to-end |
| `DCA2-GAME-002` | placeholder/future disposition decided | G05 | current/future wording without live placeholder migration |
| `DCA2-BATTLE-001` | placeholder/future disposition decided | G05 | authored-range requirement scoped to formal content |
| `DCA2-CONTENT-001` | future/undecided disposition decided | G05 | WhiteNight excluded from current implementation contract |
| `DCA2-GAME-003` | policy decided | G05 | recipe-gated Maintenance wording and no fallback |
| `DCA2-BATTLE-002` | removal approved | G06 | current enum/code/tests/docs contain no GrenadeLauncher authority |
| `DCA2-DATA-001` | unambiguous document correction | G07 | source count matches loader/live test |
| `DCA2-DOC-001` | unambiguous document correction | G07 | historical goal labels replaced by domain terms |
| `DCA2-WIRE-002` | unambiguous external document correction | G07 | current enum inventories match Core and Unity display labels |

## Explicitly Excluded Findings

| Finding | Reason | Future trigger |
| --- | --- | --- |
| `DCA2-WIRE-001` | 사용자가 이번 correction 범위에서 P1 Unity battle presentation을 명시적으로 제외했다. | 별도 Unity presentation goal을 요청할 때만 활성화 |
| `DCA2-DOC-002` | `docs - 복사본`은 user-owned backup일 수 있으며 삭제 권한이 확인되지 않았다. | 사용자가 exact external tree 정리를 승인할 때만 활성화 |

제외 항목을 이 Master 완료 수치에 포함하거나 child 구현의 부수 정리로 수정하지 않는다.

## Approved Policy Decisions

### P01 — Weapon-Authored Block Capacity

- 직원에게 class를 추가하거나 employee profile로 block 역할을 고정하지 않는다.
- `block_capacity`의 authored base authority는 장착 무기 하나다.
- 모든 current weapon RON은 값을 명시적으로 작성한다. Runtime은 archetype/range role을 보고
  hidden fallback을 계산하지 않는다.
- 기본 authoring guideline:
  - `Shield`: 3
  - `Sword`, `Spear`: 1
  - `Bow`, `Gun`, `Staff`: 0
  - 무기 없음: 0
- Armor와 employee `CombatAptitude`는 block 개수를 변경하지 않는다.
- 이 guideline과 다른 개별 무기 값이 필요하면 child가 임의로 정하지 않고 content/balance
  decision을 요청한다.
- temporary runtime buff/skill block modification은 이번 승인에 포함되지 않는다. “무기로만”이라는
  원칙과 충돌하는 modifier를 새로 추가하지 않는다.

### P02 — Formal Skill Fragment Policy

- 정식 skill fragment는 active skill을 제공하거나 그 active skill 실행 결과로 효과를 낸다.
- 현재 다수 `BasicAttackModifier` fragment는 정식 콘텐츠가 아니라 temporary placeholder다.
- Placeholder를 정식 정책으로 승격하거나 이번 correction에서 active skill로 임의 변환하지 않는다.
- 실제 fragment 콘텐츠 authoring goal이 시작될 때 정식 skill/effect로 교체한다.

### P03 — Formal Abnormality Range Authoring

- 정식 Boss/Elite/특수 abnormality 콘텐츠는 자체 attack range를 RON에 명시한다.
- 현재 30개 generic range는 placeholder로 유지하며 이번 correction에서 수치를 만들지 않는다.
- Generic fallback을 formal encounter design의 정상 최종 상태라고 문서화하지 않는다.

### P04 — WhiteNight Future Status

- WhiteNight의 12-mark/apostle/third-faction/boss-clear fragment 세부 설계는 아직 미정이다.
- 현재 gameplay 완료 조건이나 이번 implementation scope가 아니다.
- One-step/research 기반 live shape는 temporary placeholder로 유지한다.
- 추후 WhiteNight 전용 goal이 시작될 때 정책부터 다시 승인한다.

### P05 — Recipe-Gated Maintenance

- Equipment dismantle/enhance는 해당 장비의 explicit RON recipe가 있을 때만 가능하다.
- Recipe가 없으면 typed disabled reason을 제공하며 generic/fallback recipe를 만들지 않는다.
- 현재 `standard_armor`만 equipment recipe를 가진 좁은 live 범위는 유효하다.
- 다른 equipment recipe와 수치는 추후 content/balance authoring이다.

### P06 — GrenadeLauncher Removal

- `GrenadeLauncher`는 current runtime/data/wire enum authority에서 제거한다.
- Future weapon concept 문서에만 명확히 future/non-current로 남긴다.
- 실제 grenade content를 시작할 때 range, targeting, delivery와 client presentation 정책을 함께
  승인하고 schema를 다시 추가한다.
- Compatibility alias, reserved current variant 또는 silent fallback을 남기지 않는다.

## Common Invariants

- Correction은 current behavior를 문서에 맞추기 전에 accepted policy와 exact owner를 확인한다.
- Strict schema correction은 compatibility alias/default/ignored stale field를 만들지 않는다.
- Core-owned gameplay 값을 game_server나 Unity가 다시 계산하지 않는다.
- Live RON에 새 필수 field를 추가하면 모든 current entry를 같은 change set에서 명시적으로
  authoring하고 strict loader를 통과시킨다.
- Current/future/placeholder 문구를 한 문단에서 혼용하지 않는다.
- Goal history label을 current domain vocabulary로 사용하지 않는다.
- 기존 dirty worktree와 범위 밖 사용자 변경을 보존한다.
- 각 child는 자신이 소유한 correction과 직접 필요한 tests/canonical만 수정한다.

## In Scope

- Included 12 findings의 child routing, activation gate, dependency와 terminal disposition 관리
- G01–G07을 통한 scoped Core/data/test/server/external canonical correction
- 승인된 P01–P06의 nearest durable canonical 반영
- Child별 focused→broad validation과 final combined review
- Removed/current/future/placeholder vocabulary exact search와 regression 기록

## Out Of Scope

- `DCA2-WIRE-001` Unity battle presentation implementation 또는 contract 축소
- `/mnt/f/unity projects/ark/docs - 복사본` 삭제·수정
- WhiteNight, formal abnormality range, formal fragment mechanics 또는 새 Maintenance recipe 제작
- Grenade weapon content를 현재 schema에 다른 이름으로 유지하는 compatibility 작업
- 관련 없는 refactor, balance 변경, save migration, external deployment 작업
- Master 문서 자체에서 production artifacts 직접 수정

## Child Goal Catalog

Child 디렉터리는 실제 착수 시 최신 runtime/data/dirty baseline을 다시 조사한 뒤 생성한다. 미리
빈 디렉터리나 stale PLAN을 만들지 않는다.

| ID | Planned child goal | Primary ownership | Dependency | Initial state |
| --- | --- | --- | --- | --- |
| G01 | `gameplay_ron_nested_strictness_correction` | `DCA2-DATA-002` | none | complete |
| G02 | `battle_setup_catalog_ref_correction` | `DCA2-WIRE-003` | none | complete |
| G03 | `endless_completion_regression` | `DCA2-TEST-001` | none | complete |
| G04 | `weapon_block_capacity_authoring_contract` | `DCA2-GAME-001`, P01 | G06 recommended first | complete |
| G05 | `placeholder_content_policy_canonicalization` | P02–P05 | G04 contract wording available | complete |
| G06 | `grenade_launcher_schema_removal` | `DCA2-BATTLE-002`, P06 | none | complete |
| G07 | `documentation_alignment_drift_cleanup` | `DCA2-DATA-001`, `DCA2-DOC-001`, `DCA2-WIRE-002` | G04–G06 final shapes | complete |

`not_started`는 Master 표의 진행 표기이며 child Goal Status가 아니다. Child가 생성되면
`active`, `policy_decision_required`, `externally_blocked`, `correction_required`, `complete` 중 실제
상태를 사용한다.

## Implementation Plan

### Recommended Execution Order

```text
Technical corrections: G01, G02, G03
  -> schema cleanup: G06
  -> weapon block contract: G04
  -> placeholder/current policy canonicalization: G05
  -> final document drift cleanup: G07
  -> combined review
```

- G01–G03는 서로 다른 owner이며 독립적으로 착수할 수 있다.
- G06을 G04 전에 수행하면 current weapon archetype inventory가 확정된 상태에서 모든 weapon RON
  block 값을 authoring할 수 있다.
- G05는 G04의 최종 block wording을 반복 소유하지 않지만 같은 gameplay/content canonical을 만질
  수 있으므로 G04 뒤에 수행해 dirty overlap을 줄인다.
- G07은 최종 current enum/source/용어를 반영하는 문서-only cleanup이므로 구현 child 뒤에 수행한다.

## Child Scope And Gates

### G01 — Nested RON Strictness

In scope:

- live-reachable shared nested gameplay type의 unknown-field rejection
- actual outer production database를 통과하는 negative regression
- current live RON/fixture strict migration이 필요한 경우 정확한 field correction

Completion gate:

- 감사 probe가 확인한 `MovementDef`, `ResonanceDef`, `TileRangePattern`, `DeliveryDef`,
  `SkillEffectDef`, `StatModifier`, `MapNodePayload`와 battlefield wrapper reachability를 재조사한다.
- 모든 demonstrated stale field가 production-shaped parse에서 실패한다.
- `ron_loading`, data-focused와 Core broad suite가 통과한다.

### G02 — Battle Setup Catalog Refs

In scope:

- `enemy_briefing`/wave kind-aware actual abnormality ID collection
- empty/non-catalog/corroded profile sentinel rejection or filtering
- exact setup DTO test, server pass-through confirmation과 focused live probe

Completion gate:

- Corroded-only setup은 empty `abnormality_ids`를 보낸다.
- Mixed/abnormality encounter는 sorted/deduped actual catalog IDs만 보낸다.

### G03 — Endless Completion Regression

In scope:

- all-response-complete state를 직접 구성해 exact next completion outcome을 assert하는 focused test

Completion gate:

- Branch를 간접 조건이나 broad smoke가 아니라 named test 하나가 직접 보호한다.

### G04 — Weapon Block Capacity

In scope:

- required weapon RON authoring field와 strict validation
- equipped weapon → final player unit block projection pipeline
- armor/employee profile duplicate authority 제거
- no-weapon 0 behavior
- snapshot/preview가 block을 노출한다면 같은 Core-computed 값 사용
- gameplay/data/external wire canonical과 focused/broad tests

Completion gate:

- 모든 live weapon entry가 explicit block value를 가진다.
- archetype guideline은 authoring review 기준이고 runtime fallback이 아니다.
- armor 변경만으로 block이 바뀌지 않고 weapon 변경은 다음 authoritative draft/projection에서
  정확히 반영된다.
- Shield/Sword/Spear/Bow/Gun/Staff와 unarmed representative tests가 있다.

### G05 — Placeholder Content Policy Canonicalization

In scope:

- formal fragment policy와 BasicAttackModifier placeholder 구분
- formal abnormality authored-range requirement와 current placeholder 구분
- WhiteNight future/undecided status와 current placeholder 구분
- recipe-gated Maintenance와 current `standard_armor` coverage 명시

Out of scope:

- fragment/abnormality/WhiteNight live mechanics, values, rewards 또는 RON redesign
- 새 Maintenance recipe authoring

Completion gate:

- Durable 문서가 placeholder를 implemented formal design 또는 regression으로 오해하게 하지 않는다.
- Current live behavior 설명이 future design promise를 대체하지 않는다.

### G06 — GrenadeLauncher Schema Removal

In scope:

- Core enum/match/test/fixture/current external enum inventory 제거
- live data absence 확인과 strict removed-value negative test
- future content 문서의 non-current concept 보존

Completion gate:

- production/current contract 검색에 `GrenadeLauncher` authority가 없다.
- Future note만 남고 current parser는 removed variant를 거부한다.

### G07 — Documentation Drift Cleanup

In scope:

- data source count 23→actual loader count
- `G15`/`G24` historical labels를 current domain terms로 교체
- external current `MapNodeCategory`와 `TargetingProfile` inventory 갱신
- G04/G06 이후 final weapon/current enum wording 확인

Out of scope:

- external `docs - 복사본` 삭제 또는 수정
- P1 battle presentation promise 축소

Completion gate:

- Path/link/enum/count exact search가 current code/data와 일치한다.
- 문서 수정이 implementation drift를 숨기거나 future placeholder를 current로 승격하지 않는다.

## Test Requirements

각 child는 focused test를 먼저 실행하고 자신이 실제 영향을 준 범위만 broad validation한다.

Minimum combined menu:

```text
cargo test -p game_core game::data --lib -- --test-threads=1
cargo test -p game_core --test ron_loading -- --nocapture
cargo test -p game_core game::world::tests::combat --lib -- --test-threads=1
cargo test -p game_core game::world::tests::equipment --lib -- --test-threads=1
cargo test -p game_core --all-targets -- --test-threads=1
cargo check -p game_server
cargo test -p game_server -- --test-threads=1
```

External DTO/current contract를 바꾸는 child는 실제 Unity compile/EditMode와 필요한 PlayMode/probe를
추가한다. Docs-only G05/G07은 production test를 의식적으로 생략할 수 있지만, exact source/link/
term search와 affected canonical cross-check를 기록해야 한다.

## Completion Conditions

- Included 12 findings이 각각 exactly one primary child에서 `complete`, `not_applicable` 또는
  사용자 승인 terminal disposition을 가진다.
- P01–P06이 가장 가까운 durable canonical과 actual implementation/current placeholder 상태에
  반영된다.
- G01–G07 child가 자신이 선언한 focused/broad gate와 독립 review를 통과한다.
- No compatibility alias, hidden fallback, dual authority 또는 stale current enum wording이 남지 않는다.
- P1 Unity presentation과 `docs - 복사본`은 손대지 않았으며 excluded 상태가 최종 보고에 명시된다.
- Combined Core/server/필요 Unity 검증과 exact legacy search가 통과한다.
- Master completion review가 audit finding registry와 최종 code/data/docs를 독립적으로 대조한다.
- 사용자와 의논하여 정해야 할 새 정책이 있을 경우 해당 child를 종료하고 Master를 완료로 표시하지
  않는다.

## Stop Conditions

다음이 발견되면 affected child를 `policy_decision_required` 또는 `externally_blocked`로 전환하고
임의 결정하지 않는다.

- weapon별 guideline 예외 수치가 실제 콘텐츠에 필수인 경우
- block change가 save migration 또는 승인되지 않은 dynamic skill modifier 정책을 요구하는 경우
- strict nested parsing이 현재 live field를 obsolete로 판정하지만 삭제/대체 의미가 불명확한 경우
- WhiteNight/fragment/abnormality placeholder를 실제 mechanics로 바꾸어야만 문서를 정리할 수 있는 경우
- external deployed consumer가 removed GrenadeLauncher value compatibility를 요구하는 경우
- excluded P1 Unity presentation 또는 `docs - 복사본` 정리가 correction에 필요해진 경우

## Repository Safety

- 현재 worktree는 광범위한 user-owned dirty 상태다.
- Child 착수 전 `git status --short`와 수정 대상 diff를 다시 확인한다.
- `git restore`, `git reset`, 과거 파일 copy, unrelated cleanup을 수행하지 않는다.
- Audit goal 문서와 archived report를 current runtime authority로 편집하지 않는다.
- External Unity 파일 수정은 해당 child 범위와 실제 current canonical에만 한정한다.

## Completion Result

Independent review: [COMPLETION_REVIEW.md](COMPLETION_REVIEW.md)

Result: Complete / High. All 12 included findings are complete under exactly one primary child; no
immediate correction, follow-up refactor or policy decision is required for this scope.
