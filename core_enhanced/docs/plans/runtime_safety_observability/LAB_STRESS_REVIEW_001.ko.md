# 기능별 실험실 스트레스 검증 — 1차 코드 검토와 정책 결정

2026-09-27 KST. 사용자 요청: “검토 및 계획 수립을 시작해줄래? 중간에 결정이 필요한 어려운 정책이 있으면 내게 물어보고.”
상태: **1차 코드 조사 기록 / 제작 주체 답변 반영 / 후속 상세 계획 진행 대상**.
상세 계획 완료·제품 구현·시험 통과를 뜻하지 않는다. 앞서 작성한 일반 RON/메트릭 계획은 유지한다.

## 확인한 현재 구조

| 근거 | 실제 기능 | 시험 설계에 반영할 사실 |
| --- | --- | --- |
| [SkillDef/SkillStepDef/SkillEffectDef](../../../src/game/ability.rs) | 단계별 target/targeting/condition/repeat/delivery/effects, EffectSite 출처 | 스킬 이름·사용자 분류 하나보다 실제 effect와 step 조합을 분석해야 함 |
| 같은 파일 DeliveryDef | Instant, Projectile, TileArea. projectile lifetime/range/pierce/kill과 area tick/duration/tracking | 유닛 수·실제 영향 대상 수·동시 carrier 수·tick 빈도를 구별 |
| 같은 파일 SkillStepRepeat/SkillStepCondition | Once, Times, ByBuffStacks; 이전 피해/버프 조건 | 반복·조건을 실제로 활성화한 coverage 필요. 단순 스택 기반 반복을 스택 전이라고 부르지 않음 |
| 같은 파일 SkillEffectDef | Damage/Heal 및 식, modifier, resource, buff, InterruptCast, ExtraAttack | 기존 게임 규칙상 InterruptCast는 유지. 안전 예산 초과 시 자동 스킬 취소는 계속 범위 밖 |
| [TriggeredEffect](../../../src/game/stats.rs), [장비 activation](../../../src/game/data/equipment_data.rs) | OnAttack/OnHit/OnKill/OnDeath/OnAllyDeath 등, AllAllies/AllEnemies, TriggerProc→ability_id | 단일 스킬뿐 아니라 장비·후속 effect 참조를 포함해 실제 비용 경로를 확인해야 함 |
| [value expression catalog](../../../src/game/value_expr/catalog.rs) | source에 결속된 compiled amount plan, runtime 요구 context | 집계·Snapshot/Live 비용은 delivery/실제 member cardinality와 함께 시험 |
| [TrialScenario](../../../src/authoring/scenario.rs) | ApplicationCommand 기반 Command, Snapshot, Clock, Assert | 독자적인 가짜 executor 대신 실제 명령/clock을 재사용할 여지가 있음. 대규모 spawn adapter는 별도 조사 필요 |
| [TrialSession](../../../src/authoring/trial.rs) | prepare/advance, receipt, content identity, input hash, 실패 위치와 마지막 accepted session | 증거 기반 trial 재사용 가능. run_to_end는 입력 step 끝까지이며 승리나 지정된 시험 부하 달성을 자동 보장하지 않음 |
| [author_content](../../../src/bin/author_content.rs) | validate/describe/explain/simulate CLI | 개발용 검증 진입점은 존재. 여기서 플레이어용 제작 UI의 존재를 추론하지 않음 |
| [publish_snapshot](../../../src/game/data/content_pack/mod.rs) | parse→resolve→semantic identity→validation→compiled catalog/preview→ValidatedGameData | 정적 유효성과 스트레스 시험 통과는 현재 서로 다른 의미. 시험 실행용 로딩 자체에 PASS를 요구하면 순환 의존이 생김 |

조사한 SkillEffectDef에는 범용 stack transfer/spread 연산이 없다. 기존 ApplyBuff/ByBuffStacks를 해당 기능의
구현 완료로 세지 않는다. 실제 전이/전파 기능이 후속 작업에서 추가되면 그때 feature detector·시험을 연결해야 한다.
아직 읽지 않은 전체 후속 기능의 부재를 증명한 것은 아니다.

현재 workspace에서 명시적인 laboratory/skill-lab UI 경로는 확인하지 못했다. 공개 authoring 시험 API가 있다는
사실과 사용자가 말한 제품의 실험실 기능 범위는 구분한다. Certified combat power는 성능 검증 인증과 무관하다.

## 해결된 정책 질문과 후속 변경

**질문:** 실험실에서 스킬을 제작하는 주체는 누구인가?

- 최초 답변: **“게임을 플레이하는 모두.”** 이후 사용자는 **실험실이 모드 제작자 도구이며 스킬 추가는 모드 추가**라고 구체화했다.
- 사용자 후속 제안: 첫 실행 또는 스킬 레지스트리 변경 시 모든 스킬을 실험실에서 검증하고 캐시로 실행 허용을 판단한다.
- 검토자 권고: 데이터 내용/종속/엔진/검증 정책에 결속된 캐시, 실패 시 편집·진단 가능한 실험실 유지,
  실제 전투 진입과 등록의 차단 경계를 구분한다. 이후 확정 정책은 다음 항목을 따른다.
- 후속 사용자 결정: 실패 모드의 본편 적용을 거부한다. 레지스트리 전체 일괄 거부 추천은 채택되지 않았다.
- 검증 주체는 core 실행 주체(로컬/서버)를 따른다. 저사양을 이유로 기준을 완화하지 않는다.
- 현재 pack 의존성·writer 검사를 재사용하되 스트레스 승인 게이트는 새 연결이 필요하다.
  [정책 원본과 적용 경계](LAB_STRESS_PROPOSAL.ko.md)에 확정 사항 및 기술적 도출 사항을 구분했다.
- 사용자 후속 확정: **20분 검증을 전체 단계에서 제거**하고 기반 마련 뒤 본인이 직접 수행한다.
- 추가 추천안 세 가지도 사용자 동의로 확정했다. 조합만 실패하면 해당 조합 적용을 보류하고 사용자가 구성을 변경한다.
  제외된 모드가 필요한 세이브는 원본을 보존하며 로드를 차단한다. runtime 안전 오류는 해당 전투를 중단하고 정상 결과 반영을 막는다.
  [정책 원본](LAB_STRESS_PROPOSAL.ko.md)에 목표 동작과 미검토 복원 경계를 구분했다.

이 결정에 따라 플레이어 제작 artifact·검증 실행·캐시·실게임 사용 경계를 설계한다.
기존 “실게임은 시험 통과를 전제”와 기능별 시험 요구는 유지한다. 제작 주체 질문은 더 이상 대기 사유가 아니다.

## 재개 위치

상세 설계 전에 [별도 로드맵 편입안](../../roadmaps/mod_validation_safety/ROADMAP.ko.md)을 작성했다.
이 후속 작업을 선택하면 첫 상세 계획은 MS01의 최신 경계 확인이며, 아래 실험실 상세 항목은 MS03/MS04/MS05에서 순서대로 구체화한다.

다음 상세 계획은 실제 지원 기능별 시험표와 조합/coverage 규칙, draft→검증→등록의 권한/identity 경계,
환경 실패와 콘텐츠 실패 판정, 시험 profile 및 반복 기준의 측정 계획, 구현 순서·지시서를 구체화한다.
전투 오류의 host 전달·복원·결과/보상/저장 반영 경계와 세이브의 필수 모드 판별도 코드로 확인한다.
최종 메모리·시간 숫자는 측정 전에 임의로 정하지 않는다. 500/1,000은 부하 profile 후보로 유지한다.
스킬 자동 취소 제거, 최소 runtime 안전 검사 유지, R06 별도 후속 작업이라는 기존 결정도 유지한다.

1차 조사는 코드 열람과 문서 기록이며 질문 시점에 중단했다. 이후 사용자 답변과 시험 범위 변경을 이 문서에 반영했다.
제품 변경·빌드·시험·서비스 실행·설치는 없다. 기간 시험 제외 전 기록은 revisions/long_duration_test_removed_001,
모드 정책 정정 전 기록은 revisions/mod_admission_policy_001에 보존했다.
