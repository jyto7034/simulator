# A랭크 환상체 스킬 구현 Goal 계획

이 문서는 `C:\Users\blast\Desktop\using-codex-goals-effectively-ko.md`의 원칙을 적용해, 당장 구현 가능한 A랭크 환상체 스킬을 실제 RON/테스트로 옮기기 위한 goal 문서다.

이 goal의 목적은 “많은 스킬을 한 번에 넣기”가 아니라, 현재 core 스킬 시스템으로 구현 가능한 환상체부터 원본 스킬과 직원용 스킬 파편을 안정적으로 데이터화하는 것이다.

## Goal 원칙

이 작업은 아래 기준을 만족할 때만 완료로 본다.

- A랭크 환상체의 원본 스킬과 파편 스킬이 서로 다른 `SkillDef`로 존재한다.
- 원본 스킬은 적 환상체가 쓰는 위협으로, 파편 스킬은 직원이 쓰는 안전화/왜곡된 액티브로 분리된다.
- 새 런타임 효과를 추가하지 않는다. 현재 `SkillDef`, `SkillStepDef`, `DeliveryDef`, `SkillEffectDef`, 기본 버프만 사용한다.
- 수치 밸런스 완성은 목표가 아니다. 대신 각 스킬의 전술 역할이 구분되어야 한다.
- live RON, 테스트 manifest, 문서가 같은 스킬 목록을 가리켜야 한다.
- 레거시 호환을 위해 의미 없는 adapter, compatibility layer, 임시 alias를 만들지 않는다.
- 설계와 충돌하는 기존 live 스킬이 있으면 보존보다 정리를 우선한다.

## 제외 범위

이번 goal에서 하지 않는다.

- B/C/D랭크 환상체 구현.
- 매혹, 강제 이동, 소환, 변신 페이즈, 시체 흡수, 부활, 조건부 BattleScenario 이벤트 구현.
- 개화 스킬의 실제 구현.
- 장비/기프트 전체 구현.
- 스킬 수치 밸런싱.
- 이미지, VFX, 애니메이션 리소스 제작.
- 클라이언트 UI 작업.

## 현재 근거 문서

- `docs/skills/lobotomy_abnormality_content_audit.md`
- `docs/skills/lobotomy_skill_system_feasibility.md`
- `docs/skills/lobotomy_a_rank_skill_design.md`
- `docs/skills/skill_fragment_game_system.md`

주의:

- `lobotomy_abnormality_content_audit.md`는 공식 데이터베이스가 아니라 1차 기획 조사다.
- 실제 구현 전에 개별 환상체의 이름, E.G.O 이름, 원작 행동 키워드는 필요 시 위키 개별 페이지로 재검증한다.
- 원작 IP 이미지를 직접 저장하거나 게임 리소스로 쓰지 않는다.

## 현재 코드/데이터 기준

주요 구현 지점:

- 스킬 데이터: `../game_resources/data/skills/base.ron`
- 환상체 데이터: `../game_resources/data/abnormalities/base.ron`
- 스킬 파편 데이터: `../game_resources/data/skill_fragments/base.ron`
- 보상 데이터: `../game_resources/data/events/rewards/base.ron`
- 스킬 스키마: `src/game/ability.rs`
- 스킬 검증: `src/game/data/skill_data.rs`
- 스킬 런타임: `src/game/battle/core/skill_runtime/`
- 라이브 스킬 감사 테스트: `tests/live_skill_catalog_audit.rs`
- 개별 스킬 행동 테스트: `tests/skill_test/`

현재 스킬 시스템으로 쓸 수 있는 효과:

- `Damage`
- `Heal`
- `ModifyResonance`
- `ModifyStats`
- `ApplyBuff`
- `ExtraAttack`
- `ModifyDamage`

현재 스킬 시스템으로 쓸 수 있는 전달 방식:

- `Instant`
- `Projectile`
- `Area`

현재 기본 버프:

- `poison`
- `stun`
- `freeze`
- `silence`

## 구현 대상 우선순위

### 1차 구현 묶음

먼저 아래 8개를 완성한다.

| 환상체 | 원본 스킬 | 파편 스킬 | 이유 |
| --- | --- | --- | --- |
| One Sin and Hundreds of Good Deeds | `one_sin_penitence` | `fragment_one_sin_penitence` | 원본/파편 분리 기준이 이미 잡혀 있음 |
| Scorched Girl | `scorched_explosion` | `fragment_scorched_spark` | 폭발/화염 파편의 기준 |
| Spider Bud | `spider_bud_poison_stack` | `fragment_spider_bud_red_eyes` | 독/지속 피해 기준 |
| The Red Shoes | `red_shoes_berserk` | `fragment_red_shoes_impulse` | 평타 강화/폭주형 기준 |
| Der Freischutz | `freischutz_magic_bullet` | `fragment_freischutz_black_round` | 관통/오사 위험 기준 |
| The Funeral of the Dead Butterflies | `funeral_butterfly_eulogy_volley` | `fragment_funeral_butterfly_eulogy` | 다단 원거리/처형형 기준 |
| All-Around Helper | `all_around_helper_grinder_mk4` | `fragment_helper_grinder_trace` | 근접 주변 광역 기준 |
| Alriune | `alriune_flower_burial` | `fragment_alriune_faint_aroma` | 지속 장판/구역 장악 기준 |

### 2차 구현 묶음

1차가 닫힌 뒤 아래 후보를 이어서 처리한다.

- Old Lady
- Forsaken Murderer
- Punishing Bird
- Fragment of the Universe
- Ppodae
- Warm-hearted Woodsman
- Laetitia
- Porccubus
- The Dreaming Current
- Il Pianto della Luna
- Clouded Monk

2차 묶음은 수치보다 “현재 효과만으로 원본/파편 차이를 만들 수 있는가”를 먼저 검증한다.

## Goal 체크리스트

### G1. Live 스킬 감사

목표:

- `docs/skills/lobotomy_a_rank_skill_design.md`의 A랭크 목록과 live RON 스킬 목록을 대조한다.
- 이미 구현된 원본/파편 스킬, 누락된 스킬, 이름만 있고 역할이 어긋난 스킬을 분류한다.

완료 조건:

- 1차 구현 묶음 8개에 대해 `원본 스킬 존재 여부`, `파편 스킬 존재 여부`, `환상체 skill_id 연결 여부`, `스킬 파편 imitation_skill_id 연결 여부`가 표로 정리된다.
- 기존 스킬 중 설계와 이름/역할이 충돌하는 항목이 있으면 수정 대상에 포함한다.

빠른 검증:

```bash
rg -n "one_sin|scorched|spider|red_shoes|freischutz|funeral|helper|alriune|fragment_" ../game_resources/data/skills/base.ron ../game_resources/data/skill_fragments/base.ron ../game_resources/data/abnormalities/base.ron
cargo test -p game_core --test live_skill_catalog_audit
```

### G2. 1차 묶음 원본 스킬 정리

목표:

- 1차 구현 묶음의 환상체 원본 스킬을 live RON에 명확히 둔다.
- 원본 스킬은 적 환상체가 쓰는 스킬이므로 파편보다 더 직접적이고 위험한 역할을 가진다.

완료 조건:

- 1차 구현 묶음 8개 모두 `AbnormalityMetadata.skill_id`가 원본 스킬을 가리킨다.
- 각 원본 스킬은 최소 1개 이상의 행동 테스트 또는 manifest 커버리지를 가진다.
- 원본 스킬은 파편 스킬과 같은 `skill_id`를 공유하지 않는다.

빠른 검증:

```bash
cargo test -p game_core --test live_skill_catalog_audit
cargo test -p game_core --test skill_refactor_validation
cargo test -p game_core --test skill_test_suite
```

### G3. 1차 묶음 파편 스킬 추가

목표:

- 1차 구현 묶음의 직원용 스킬 파편을 별도 `SkillDef`로 만든다.
- 기본 파편은 원본의 단순 하위 호환이 아니라 전술 역할이 다른 안전화/왜곡 스킬이어야 한다.

완료 조건:

- 1차 구현 묶음 8개 모두 직원용 파편 스킬을 가진다.
- 각 파편은 `SkillFragmentMetadata.effect.ActiveSkill.imitation_skill_id`로 연결된다.
- 파편 스킬명은 원본 스킬명과 구분된다.
- 파편 스킬은 원본 스킬보다 안전하거나 전술적으로 다른 사용법을 가진다.

빠른 검증:

```bash
rg -n "imitation_skill_id|fragment_" ../game_resources/data/skill_fragments/base.ron
cargo test -p game_core --test live_skill_catalog_audit
cargo test -p game_core --test live_item_skill_activation
```

### G4. 원본/파편 차이 검증

목표:

- 같은 환상체에서 나온 원본 스킬과 파편 스킬이 데이터적으로 독립되어 있고, 플레이 역할도 다르다는 것을 테스트로 고정한다.

완료 조건:

- 각 1차 묶음 파편은 대응 원본과 다른 `SkillDef.id`를 가진다.
- 대응 원본/파편의 step 구성, 효과, 전달 방식 중 최소 하나 이상이 다르다.
- 완전히 같은 효과를 낼 경우, 그 이유가 문서에 명시되어야 한다. 기본 정책은 “동일하지 않음”이다.

빠른 검증:

```bash
cargo test -p game_core --test live_skill_catalog_audit
```

필요하면 `live_skill_catalog_audit.rs`에 원본/파편 독립성 테스트를 추가한다.

### G5. 보상/연구 연결 최소화

목표:

- 새 파편을 실제 런에서 획득 가능한 경로에 최소한으로 연결한다.
- 단, 파편은 고가치 보상이므로 모든 환상체 격파 시 확정 지급하지 않는다.

완료 조건:

- 1차 묶음 파편은 `SkillFragmentMetadata.sources`에 격파/격리/희귀 보상 원천을 가진다.
- 보상 RON에는 최소 1개 이상의 직접 지급 또는 연구 진행도 지급 샘플이 존재한다.
- 연구 완료 수령 정책은 기존 pending research delivery 정책을 따른다.

빠른 검증:

```bash
rg -n "GrantSkillFragment|GrantSkillFragmentResearch|fragment_" ../game_resources/data/events/rewards/base.ron ../game_resources/data/skill_fragments/base.ron
cargo test -p game_core game::world::tests::
```

### G6. 문서 동기화

목표:

- 구현된 스킬 목록과 문서가 어긋나지 않게 한다.

완료 조건:

- `docs/skills/lobotomy_a_rank_skill_design.md`에 구현 완료/미완료 상태가 반영된다.
- `docs/skills/lobotomy_skill_system_feasibility.md`의 A랭크 판단이 실제 구현과 충돌하지 않는다.
- `docs/current_handoff.md`에는 “A랭크 스킬 구현 현황”이 간단히 남는다.

빠른 검증:

```bash
rg -n "A랭크|fragment_|one_sin|scorched|freischutz|alriune" docs
```

## 작업 기록 규칙

Goal mode 또는 장시간 작업으로 진행할 경우 아래 세 파일 역할을 이 문서 안에서 유지한다.

### PLAN

- 현재 goal 단계.
- 다음에 수정할 파일.
- 정책 질문이 필요한 항목.

### EXPERIMENTS

- 어떤 스킬을 어떤 데이터 구조로 구현했는지.
- 어떤 테스트가 실패했고 왜 수정했는지.
- 기존 live 스킬을 삭제/수정/유지한 이유.

### EXPERIMENT_NOTES

- 수치 조정 아이디어.
- 개화 방향 아이디어.
- 지금은 보류한 B/C랭크 확장 후보.

장시간 작업 중에는 이 문서 하단의 진행 로그에 짧게 누적한다.

## 중단 조건

아래 상황에서는 코드를 계속 수정하지 말고 사용자와 의논한다.

- 원본 스킬과 파편 스킬의 차이를 현재 데이터만으로 만들기 어렵다.
- 스킬 구현을 위해 새 런타임 효과가 필요해진다.
- 원작 행동 해석이 모호해 어떤 전술 역할로 옮길지 결정이 필요하다.
- live RON에 이미 있는 스킬이 현재 문서 정책과 충돌하지만 삭제하면 기존 테스트가 크게 흔들린다.
- 파편 획득 경로가 보상 정책과 충돌한다.

## 최종 종료 조건

이 goal은 아래가 모두 만족되면 완료다.

- 1차 구현 묶음 8개 환상체의 원본 스킬이 live RON에 존재한다.
- 1차 구현 묶음 8개 환상체의 직원용 파편 스킬이 live RON에 존재한다.
- 1차 구현 묶음 8개 파편이 `SkillFragmentMetadata`에 등록되어 있다.
- 각 환상체의 `AbnormalityMetadata.skill_id`는 원본 스킬을 가리킨다.
- 각 파편의 `imitation_skill_id`는 파편 스킬을 가리킨다.
- 원본과 파편이 같은 `SkillDef`를 공유하지 않는다.
- 신규/수정 스킬은 live skill catalog audit에 반영된다.
- 필요한 개별 행동 테스트가 추가되거나 기존 테스트 manifest에 명확히 포함된다.
- `cargo test -p game_core --test live_skill_catalog_audit`가 통과한다.
- `cargo test -p game_core --test skill_test_suite`가 통과한다.
- 문서의 구현 상태가 최신 코드와 맞다.

## 진행 로그

### 2026-05-19: G1~G5 1차 묶음 진행

G1 live 스킬 감사 결과:

| 환상체 | 원본 스킬 | 파편 스킬 | 환상체 skill_id 연결 | 파편 imitation_skill_id 연결 | 상태 |
| --- | --- | --- | --- | --- | --- |
| One Sin and Hundreds of Good Deeds | `one_sin_penitence` | `fragment_one_sin_penitence` | 완료 | 완료 | 기존 구현 유지 |
| Scorched Girl | `scorched_explosion` | `fragment_scorched_spark` | 완료 | 완료 | 기존 구현 유지 |
| Spider Bud | `spider_bud_poison_stack` | `fragment_spider_bud_red_eyes` | 완료 | 완료 | 파편 추가 |
| The Red Shoes | `red_shoes_berserk` | `fragment_red_shoes_impulse` | 완료 | 완료 | 기존 구현 유지 |
| Der Freischutz | `freischutz_magic_bullet` | `fragment_freischutz_black_round` | 완료 | 완료 | 기존 구현 유지 |
| The Funeral of the Dead Butterflies | `funeral_butterfly_eulogy_volley` | `fragment_funeral_butterfly_eulogy` | 완료 | 완료 | 파편 추가 |
| All-Around Helper | `all_around_helper_grinder_mk4` | `fragment_helper_grinder_trace` | 완료 | 완료 | 원본/환상체/파편 추가 |
| Alriune | `alriune_flower_burial` | `fragment_alriune_faint_aroma` | 완료 | 완료 | 파편 추가 |

수정한 데이터:

- `../game_resources/data/skills/base.ron`
  - `all_around_helper_grinder_mk4`
  - `fragment_spider_bud_red_eyes`
  - `fragment_funeral_butterfly_eulogy`
  - `fragment_helper_grinder_trace`
  - `fragment_alriune_faint_aroma`
- `../game_resources/data/abnormalities/base.ron`
  - `t-05-41_all_around_helper`
- `../game_resources/data/skill_fragments/base.ron`
  - `fragment_spider_bud_red_eyes`
  - `fragment_funeral_butterfly_eulogy`
  - `fragment_helper_grinder_trace`
  - `fragment_alriune_faint_aroma`
- `../game_resources/data/events/rewards/base.ron`
  - 신규 파편 직접 지급 샘플 4개
  - `a_rank_abnormality_research_reward`

수정한 테스트:

- `tests/live_skill_catalog_audit.rs`
  - live delivery manifest 갱신.
  - behavior coverage manifest 갱신.
  - `a_rank_original_and_fragment_skills_are_independent_live_contracts` 추가.

검증:

```bash
cargo fmt
cargo check -p game_core
cargo test -p game_core --test live_skill_catalog_audit
cargo test -p game_core --test live_item_skill_activation
cargo test -p game_core --test skill_test_suite
```

진행 판단:

- 1차 구현 묶음은 원본/파편/연결/감사 테스트 기준으로 닫혔다.
- `skill_test_suite`도 통과했지만, 신규 파편 4종의 개별 행동 테스트는 아직 manifest/독립성 감사 수준이다.
- 다음 작업은 신규 파편 중 플레이 감각 확인이 중요한 항목부터 개별 skill behavior test를 보강하는 것이다.
