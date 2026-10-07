# 정정002 — F01 자동 스킬의 정상 지원 경계

이 addendum은 원 pass001 보고서·closure·조건표를 보존한 채 **F01의 자동 스킬 부분과 최소 교정1번의 잘못된 전제**를 정정한다. 제품·시험 설계/작성/실행은 하지 않았다. source438에 속한 관련 파일의 현재 hash가 frozen config와 같은지 확인하고 정책·producer·consumer를 읽었다. 정확한 참조는 `AUTO_APPLICABILITY_REVIEW_002.json`에 있다.

## 독립 판단

**현재 정상 CoreApplication/공개 RON 경로에 “유물을 장착한 직원이 AutoCastStart로 스킬을 자동 발동하는” 양수 적용 경로가 있다는 제 전제는 잘못됐다.** 따라서 이 경로를 기존 지원 소비자의 누락 시험이라고 분류하거나, 그 양수 피해시험을 F01 폐쇄의 필수조건으로 요구한 부분을 철회한다. 새 activation API/게임플레이를 추가하거나 runtime unit의 mode를 시험에서 바꾸라는 요구로 대체하지 않는다.

원 P2G의 “자동” 문구가 반드시 새 직원 자동 발동 기능을 요구한다고 단정할 근거도 부족하다. 인용된 상위 H04 A14-03/04는 개인별 실제 피해·preview·평가 입력 및 cast/queued/projectile/area/readiness를 요구하지만 직원 Auto activation 신설을 지정하지 않는다. P2G 결정의 authority도 기존 사용자 합의가 우선하는 AI 구체화이며 기존 선행 계약 보존을 명시한다. 따라서 **확인된 오류는 감사자의 positive-path 전제이고, P2G에는 자동 관련 기존 경로별 적용/비적용을 분명히 연결해야 할 해석상의 공백이 있다.** 이를 원 사용자 요구가 틀렸다고 확대하지 않는다.

F01 중 **정지 상태에 접수된 정상 수동 직원 시전의 재개 후 유물 수치 실제 소비 증거 공백**은 이 사실과 독립적으로 유지한다. F02도 유지한다. 본 addendum은 F01 전체 폐쇄나 Complete 판정이 아니다.

## 정상 producer에서 실제 binding까지

1. `employee.rs:128–136`의 strict `EmployeeDefinition`은 skill/activation 입력을 받지 않는다. `from_definition():514`는 `UnitCombatProfile::for_employee`를 사용한다. starter/recruit projection은 `world/helpers.rs:630,693`, HQ 채용은 `world/headquarters.rs:70`에서 이 생성자를 사용한다.
2. `battle/types.rs:575–618`의 직원 기본 profile은 `skill_id=None`, `fragment_source=None`, `skill_activation_mode=Auto`다. 여기의 Auto 기본 enum 값은 **실행 가능한 직원 자동 스킬이 존재한다는 뜻이 아니다.** skill이 없다.
3. `skill_fragment.rs:355–417`의 `SkillFragmentLoadout::apply_to_profile`은 baseline ActiveSkill을 거절하고, 정상 active copy를 적용할 때 `skill_id`와 exact fragment source를 설정하면서 **Manual을 강제**한다. `data/skill_fragment_data.rs:143`의 strict form schema에도 activation mode가 없다. 대상 선택이 `automatic_unit`인 SkillId를 장착해도 이 Manual 지정은 바뀌지 않는다.
4. `battle/core/mod.rs:1049–1063`의 `can_start_autocast`는 Auto mode만 허용한다. `add_resonance():1102`의 자동 시작 예약 역시 Auto를 요구한다. 정상 직원은 skill없음/Auto 또는 active skill있음/Manual의 두 상태이므로, 위 정상 생산 경로에서 skill있음/Auto 조합을 얻지 못한다.
5. 자동 발동 가능한 abnormality, corroded employee, summoned unit, independent object 등은 `BattleUnitSourceIdentity`가 일반 Employee와 구별된다(`battle/types.rs:448–510`). `bind_skill_relic_source`의 `relic_numeric.rs:483–502`는 일반 **Employee identity에서만** 직원의 copy/slot을 찾아 positive 기여를 결속한다. 자동 actor가 다른 identity이면 원 source를 유지하여 반환한다. 이름에 employee가 포함된 CorrodedEmployee도 일반 Employee 슬롯 소유자로 취급하지 않는다. 이미 합법적으로 캡처된 지연 binding의 수명은 이 신규 결속 판정과 별도 기존 계약이다.
6. `corrosion.rs:419`의 직원 mode Auto 변경과 AutoCastStart 직접 enqueue는 `#[cfg(test)] mod tests` 내부다. public native 구조체로 Employee profile을 직접 조립하는 가능성도 정상 등록/명령 경로의 증거가 아니다. P2G50:79의 정상 경로 우선 조건을 우회하는 근거로 사용할 수 없다.

## “자동” 관련 실제 적용 구분

| 소비자/용어 | 현재 정상 지원 | U08 유물 적용 의미 |
| --- | --- | --- |
| 직원 자동 기본공격, `AttackKind::Auto` | 지원 | 직원 copy/slot의 basic-damage 조건에 맞으면 positive 적용. 기존 U08 melee/projectile Auto 시험이 이 경로를 검증한다. 스킬 AutoCastStart와 혼동하지 않는다. |
| 직원 authored 추가 기본공격, Triggered | 지원 | basic damage의 별도 정상 trigger 소비자. 기존 Auto/Triggered 시험이 두 종류를 구별한다. |
| 직원 수동 시전의 `SkillCastSelection::AutomaticUnit` / `automatic_unit` | 지원 | **발동은 Manual**, 대상 선택을 Core가 수행한다. 유물의 skill-damage 조건은 해당 직원/fragment/SkillId/effect에 적용될 수 있다. 이것을 AutoCastStart의 양수 증거로 이름 바꾸지 않는다. `skill_target_contract.md:221,247` 및 `game_rules/combat.md:171–185`도 이 구분을 명시한다. |
| 정상 직원의 active skill `SkillActivationMode::Auto` / AutoCastStart | 위 정상 경로에는 없음 | 정상 positive 유물 소비시험의 대상이라고 전제할 수 없다. 새 지원을 만들지 않고 “현재 정상 입력으로 도달 불가”를 명시해야 한다. |
| abnormality/corroded/summoned/object의 AutoCastStart | 지원 | 직원 유물 소유자로 재해석하지 않는다. 현재 identity guard상 새 직원 유물 positive binding 비적용이며 기존 자동 시전 결과/권한을 보존하는 경로다. generic 자동 스킬 증거는 이 계약의 근거가 될 수 있으나 유물 직원 자동 발동의 근거는 아니다. |
| Pause 중 접수된 직원 Manual 시전→Resume→방출/피해 | 지원 | 직원의 exact 유물 수치와 source 보존이 적용되어야 한다. 기존 generic pause 시험과 running U08 manual 시험을 나란히 놓는 것만으로 이 조합의 실제 수치 소비를 증명하지 못한다는 F01의 나머지 판단은 유지한다. |

## 원 요구와의 관계 및 후속 판단 경계

“현재 지원되는 모든 실제 consumer에 해당 소유자의 개인 계획을 적용하며, 소유하지 않는 actor에는 새 기여를 만들지 않는다”는 해석은 기존 수동 직원 계약·유물 사본 소유·H04의 개인 plan 요구와 양립한다. 이 해석에서는 자동 기본공격은 positive 적용, 수동 automatic_unit은 Manual skill 적용, nonemployee 자동 스킬은 직원 유물 신규 결속 비적용이며, **직원 Auto activation 신설은 요구되지 않는다.**

다만 동결된 PER07/PE07.2와 P2G50/CodeReview에 이 applicability 구분이 명시되지 않았으므로, Root가 상위 사용자 요구와 기존 지원 계약을 근거로 후속 인수에서 해석을 기록하고 해당 coverage 연결을 일관되게 해야 한다. 이는 시험을 통과시키기 위해 자동을 기본공격과 동일시하거나 기대값을 완화하는 일이 아니다. 반대로 frozen “자동”을 반드시 **유물 직원 AutoCastStart positive**라고 유지하려면, 이는 기존 지원 누락 시험이 아니라 정상 producer가 없는 **요구/설계 경계 불일치**다. 그 경우 권한 있는 요구·설계 검토가 먼저이며 감사자가 API 확장을 교정으로 지시하지 않는다.

이 정정의 결과로 원 F01 최소 교정1번은 적용하지 않는다. F01의 자동 부분은 위 applicability 인수 대상으로 분리하고, 실제 지원되는 paused manual 소비 증거 공백은 그대로 남긴다. 원 `AUDIT_CLOSURE.json`의 역사적 문구는 덮어쓰지 않으며 후속 검토는 본 addendum과 함께 읽는다. F01/F02의 전체 폐쇄 여부는 후속 target의 실제 근거로 판단한다.
