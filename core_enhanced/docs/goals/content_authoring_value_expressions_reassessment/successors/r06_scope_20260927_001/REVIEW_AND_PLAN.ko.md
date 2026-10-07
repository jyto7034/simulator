# R06 범위 재대조와 후속 계획

2026-09-27 · 사용자 정책 결정 반영 · 계획/문서 작업. 제품 구현·시험·완료 감사 없음.

개발 진입점은 [ROADMAP_STATUS](../../../../roadmaps/content_authoring/ROADMAP_STATUS.md) 하나다.
이 디렉터리는 기존 R06의 **부분 구현을 이어받는 successor 계획**이다. 기존 동결 문서를 수정하거나
이미 구현한 기능을 처음부터 다시 만들지 않는다. 다음 구현 범위는 [구현 지시서](IMPLEMENTATION_REQUEST.ko.md)다.

## 1. 이번에 확정한 범위

R06은 **스킬이 사용하는 수치를 안전하고 일관되게 계산하는 기반**이다.
typed 수식, release Snapshot/Live, 실제 완료 결과 읽기, 유한 집계, Attack/SpellPower·무기 성장의
일관된 합성, 같은 문맥의 preview/explain, 저장·fingerprint·회귀 검증을 소유한다.

programmable skill 전체, 구조화 Flow/임의 조회, 상태·연결망, 본편 전용 스킬 모듈,
편집기 UI, 실험실 스트레스·모드 적용·메트릭은 R06 전체 완료 조건으로 추가하지 않는다.
모델은 구현하며 개선한다. 일반 피해식은 명시적 `Source.Attack`이며, 대상 기반 집계 예시는
계산 기능의 시험 사례다. 실제 스킬에 타겟 공격력 기반 피해를 강제하는 요구가 아니다.

### 사용자 결정 원문과 적용

| ID | 사용자 답변 | 계약 |
| --- | --- | --- |
| U01 | “실제 직원만 직원별 집계의 입력으로 허용한다. 소환물은 자동으로 A에게 합치지 않는다 (추천)” | 실제 Employee identity만 직원 키를 생산한다. 소환자/소유자 UUID가 있다는 이유로 직원으로 취급하지 않는다. |
| U02 | “두 관계를 별도 기능으로 명시한다. 효과는 출처 장비별로, 착용 장비 조회는 지정한 슬롯의 실제 사본별로 묶는다 (추천)” | `효과→출처 장비 사본`과 `유닛→명시한 착용 슬롯의 실제 사본`은 별도 관계다. definition ID, 첫 장비, 시전자 장비로 대체하지 않는다. |
| U03 | “제작자가 제외 조건을 명시해야 한다. 조건 없이 섞으면 검증 또는 실행 오류로 처리한다 (추천)” | 적용 불가능한 정상 member를 조용히 제거하지 않는다. 제작자가 제한된 명시적 조건으로 먼저 제외한다. 잘못된 identity/권한/문맥은 필터로 숨기지 않는다. |
| U04 | “R06은 유닛 집계까지, 효과·상태 집계는 조회 기반 단계에 배정 (추천)” | R06의 실제 집합은 이미 resolve된 유닛 목록이다. Side/ActualEmployee/EquippedSlot과 명시적 필터를 구현한다. 효과·상태의 출처별 조회·집계는 후속 owner에 배정한다. |

U01–03은 `call_B1MgDUyofPJvz7cjmrH3DpJj`의 질문 0–2에 대한 답변,
U04는 `call_zGeOg6UmK47KW0NMMuBjo2ee`의 질문 0에 대한 답변이다.
이전 “실제 사용처가 정해질 때 후속 구현” 선택은 사용자의 “아니다 지금 확실히 정하고 가자”로 철회됐다.
따라서 유닛 ByOwner 전체를 보류하는 계획을 선택하지 않는다.

## 2. 현재 코드와 증거

비교 기준은 [D04-F01 successor manifest](../../execution/single_goal/corrections/wp02_d04_f01_001/target/d04_f01_correction_001/MANIFEST.json)다.
manifest SHA-256은 `e4f0d77b1bd3066493d605271605d7a4b6e16317a6d540d303909b076b480104`.
이번 계획 조사에서 manifest의 source 333개를 현재 파일의 크기·SHA-256과 읽기 전용 비교했고 차이가 없었다.
이는 제품 시험·양방향 복원·독립 감사를 이번에 재실행했다는 뜻이 아니다.

| 근거 | 확인한 현재 동작 | 남은 변경 |
| --- | --- | --- |
| `src/game/value_expr/mod.rs`, `compile.rs` | AuthoredMembers/ResolvedTargets, WholeSet/ByOwner/ByProjectedTile. EvalMember의 owner는 한 개. authored owner domain 혼합은 거부 | resolved 관계 선택자와 명시적 필터. 같은 식에 여러 관계를 써도 키가 덮어써지지 않아야 함 |
| `src/game/battle/core/value_expr_runtime.rs` | 실제 WholeSet/타일 집계, Unit identity 중복 제거, 명시적 Attack/HP 읽기. resolved ByOwner는 declared domain 부재로 오류 | 실제 Side/Employee/선택 슬롯 키 생산. 유닛을 implicit scalar로 바꾸지 않음 |
| 같은 파일 `ActionResultWitness`와 capture | 완료 효과 결과를 step·delivery에 걸쳐 누적하고 release Snapshot 고정 | 새 필터/관계 키도 같은 moment를 따름. 기존 witness를 새로 설계할 필요 없음 |
| `src/game/battle/types.rs` | BattleEquipmentBinding에 base UUID·실제 사본 UUID가 있으나 슬롯 주소는 없음 | 슬롯 관계가 전투까지 보존되도록 producer/consumer 동시 변경 |
| `src/game/resources/item_slot.rs`, `combat_setup/player_spawns.rs` | ByType/Any3 layout. iter()는 빈 슬롯을 생략하고 spawn/profile projection이 슬롯을 버림 | 명시적 layout+slot 주소를 전달. iter 위치를 장착 슬롯으로 오인하지 않음 |
| `src/game/battle/core/resource_runtime.rs` | resource owner 해석에는 source_owned_uuid를 쓰는 경로가 있음 | 새 직원 grouping은 실제 Employee variant 확인. 기존 resource 의미를 이번에 일괄 재정의하지 않음 |
| `src/game/battle/stat_pipeline.rs`, `stats.rs` | 강화 projection의 실제 copy 연결·SpellPower 등 WP03 과제 잔존 | WP03 유지. 슬롯 정보를 붙인 것만으로 성장/강화 문제가 해결됐다고 표시하지 않음 |
| `src/authoring/query.rs`, `preview.rs`, `value_expr/catalog.rs` | explain/preview는 주로 정적 구조·공간. 새 amount production publication은 단계 구현 때문에 차단 | D07 및 최종 공개 경로 검증 유지. D04 부분 검토에서 publication 차단 해제 금지 |

위 경로는 `core_enhanced/` 기준이다. 전체 제품 시험 1,509개, 교정 focused 9개, PC 36조건 등은
이전 동결 구현/검토의 결과다. 이 계획의 실행 결과로 재기재하지 않는다.

## 3. 기존 요구와 모델의 대조

기존 [CODE_REVIEW_003](../../CODE_REVIEW_003.ko.md),
[Goal Set](../../P2G_50_GOAL_SET_001.ko.md)의 D01–D10, E01–E09, C01–C08을 대조했다.

| 기존 요소 | 이번 처리 | 구현·검증 owner |
| --- | --- | --- |
| E01 / D01 typed 식·checked 연산·단위·rounding·유한 compiler | 유지. 관계/필터를 bounded typed AST에 추가, 범용 스크립트 VM 도입 없음 | WP01 잔여 접속 + WP02, 최종 WP04 |
| E02 / D02–03 release/Live·소비 경계·동일 source/action/target | 의미 유지. 필터 판정/관계 키/값을 같은 frame에서 읽는 설계 추가 | WP02 |
| E03 / D04 유한 source·명시적 owner·타일·빈 집합 | **정책 구체화 및 scope 명시**: U01–04. 기존 authored 입력/WholeSet/타일 유지. 실제 유닛 관계는 이번에 닫고 효과·상태 조회는 후속으로 배정 | WP02; 후속 계약은 §6 |
| E04 / D05 Attack100에 weapon40 한 번·SpellPower 분리·실제 사본 강화 | 유지. 장착 슬롯 projection과 실제 copy enhancement는 관련 있지만 별도 완료 조건 | WP03 |
| E05 / D06 자원 lease·결과 출처·수명·stale 거부 | 유지. grouping key가 읽기/소비 권한을 새로 발급하지 않음. 출처 장비 단건 읽기와 효과 집합 조회 구별 | WP02 |
| E06 cadence·AS·point/ms | 유지. 고정 배치·전술 이동 합의로 cadence 의미가 바뀌지 않음 | WP03 |
| E07 / D07 같은 frame amount/trace·저장·effective fingerprint | 유지. 관계/필터/슬롯의 의미 입력도 fingerprint에 포함. 정적 설명에 문맥이 없으면 그 사실을 표시 | WP02 D07 + WP03 |
| E08 / D08–09 실제 공개 실행·typed 거부·bounded safety | 유지. 실험실 계획이 현재 hard guard를 대체하지 않음. 비용 시험의 특정 약9MB peak 판정만 기존 사용자 변경 적용 | WP01–04 |
| E09 / D09–10 literal/save 호환·실제 시험·독립 감사·후행 제외 | 유지. R06 부분 구현을 R06 완료로 승격하지 않음 | WP04 + 별도 요청의 독립 감사 |

공동 WP에 속한다는 이유만으로 정책 9개를 전부 재결정하지 않는다. E03의 명시적 의미 변경을 정책 owner에서 반영하고,
E02/E05/E07/E08/E09는 그 의미를 보존한 접속/검증 영향으로 기록한다. E04/E06의 수치 정책은 재개방하지 않는다.
Phase 1개·Goal 1개·4 WP와 atomic group은 유지한다. 기존 구현 증거를 보존한 successor P2G 산출물로 연결한다.

## 4. D04 입력에서 소비까지

| 입력 | 읽을 값 | 그룹 키 | 시점 | 소비/상태 |
| --- | --- | --- | --- | --- |
| AuthoredMembers | 명시적 typed scalar | 기존 명시적 owner 또는 tile | 식의 Snapshot/Live 계약 | 기존 수식 입력 유지. 다른 domain을 조용히 섞지 않음 |
| ResolvedTargets | `AggregateMember.Attack`, HPCurrent/Max/Missing 등 명시적 지원 ref | WholeSet, 실제 관측 타일, Side, ActualEmployee, EquippedSlot(address) | Snapshot은 release 확보 가능 범위, Live는 effect 변경 전 | Damage/Heal amount 등 기존 소비자. 일반 Source.Attack과 별개 기능 |
| 효과·상태 member | 상태 값/스택/자원 및 identity | **그 효과의 출처 장비 사본** | 해당 producer가 보장하는 생명주기·frame | R11 기반 → 실제 소비와 함께 접속, 원천별 종합 집계 R17. R06에서 지원 완료로 표시하지 않음 |
| 타일 자체 | 타일 값/자원 | 타일 identity | 객체/공간 producer 계약 | 빈 타일 포함 조회는 R13/R17. 유닛 관측 타일별 집계와 다름 |
| 완료 결과 prefix | 실제 HP 손실/회복·자원 변화 등 지원 result field | 기존 action/source/occurrence 범위 | Snapshot 고정 또는 Live prefix | WP02 기존 읽기 유지. 임의 효과/상태 전체 조회 기능을 뜻하지 않음 |

명시적 필터는 **집계의 입력**에만 적용한다. 직원만 합산하더라도 스킬의 원래 타격 대상 중 적을 제거하지 않는다.
직원 A가 부여한 효과의 출처 장비가 X이고 대상 B가 장비 Y를 착용했다면, 출처별 키는 X,
`EquippedSlot(B, 지정 슬롯)`의 키는 Y다. 서로 대체하거나 자동 fallback하지 않는다.

## 5. 구현 순서와 완료 판단

1. **다음 단계검토:** WP02 D04의 명시적 유닛 관계·필터·슬롯 전달·Snapshot/Live·실제 소비 경로를 구현한다.
   필요한 compiler/schema/clone/fingerprint 접속은 같이 닫는다. 기존 WholeSet/타일/F01 회귀를 유지한다.
   [구현 지시서](IMPLEMENTATION_REQUEST.ko.md)의 검증·target 동결 후 독립 검토 요청에서 중단한다.
2. 단계검토 결과를 반영하고 D07의 정적 설명/실제 frame preview와 WP03 stat/copy/save를 연결한다.
   D07 전부를 WP03 전에 완료한 것으로 취급하지 않는다. WP03의 최종 effective input·legacy save까지 함께 닫아야 한다.
3. WP04에서 전체 공개 parser→실행→관측, literal/V7·Certified 입력, 비용/회귀 및 적용 가능한 T06/T18/T27 증거를 취합한다.
4. 별도 요청에 따른 독립 완료 감사 후에만 WP02/R06 전체 완료를 판정한다.

| 완료 조건 | 이번 범위 해석 |
| --- | --- |
| C01 | 전체 공개 authored 식·typed 거부. 다음 구간의 시험 전용 접근만으로 충족되지 않음 |
| C02 | release/Live, 실제 source/action/target, 기존 정상 cancel 경계. 새 필터/키도 함께 검증 |
| C03 | authored 유한 입력 + 실제 유닛 WholeSet/타일/Side/Employee/지정 슬롯 + 명시적 필터/거부 + 결정론/예산. U04로 배정한 효과·상태 조회는 후속의 별도 완료 조건 |
| C04 | Attack·weapon·growth·actual copy 강화의 단일 합성 및 SpellPower 분리. 유지 |
| C05 | 자원·결과 read authority/lifetime 및 원자적 실패. grouping과 소비 권한 혼동 없음 |
| C06 | 같은 frame의 실제 amount/trace, old V7 round-trip/owned data 및 Certified effective input. 유지 |
| C07 | 실제 nonzero 시험·적용 가능한 T06/T18/T27·유한성·회귀. 특정 concurrent peak만 §7의 경고 정책 |
| C08 | 최종 source에 대한 별도 독립 감사. 계획/부분 검토로 대체 불가 |

## 6. 후속에 배정한 계약

기존 R 순서·기능 선행은 변경하지 않는다. 후행이 필요한 입력을 R06에 가짜 구현하지 않는다.

| owner | 이번에 넘기는 요구 | 해당 기능의 완료 전 필요한 증거 |
| --- | --- | --- |
| R11 | effect/status instance의 host와 실제 source equipment copy를 분리해서 보존하는 ledger/membership 조회 기반 | 사본 X/Y의 같은 definition 상태 격리, source-local 제거, 퇴장/만료/stale 및 저장 경계 |
| R14 등 상태 집계의 최초 실제 소비자 | 조회 기반이 준비된 상태 집합을 수식에 연결. 필요한 producer와 evaluator adapter를 함께 구현 | 실제 상태 값/스택으로 계산→실제 소비까지. 기존 literal member를 상태 조회인 것처럼 사용하는 fixture 금지 |
| R17 | source-scoped status/자원 집계·공간/일제 소비의 최종 통합 owner | X가 부여한 여러 효과끼리 묶이고 Y와 분리됨. 대상의 착용 장비 변경으로 source key가 바뀌지 않음. 명시적 필터, 유한/결정론, Snapshot/Live·stale·동시 소비·실패 원자성 |

R11이 모든 상태 종류와 R17 기능을 먼저 구현해야 한다는 뜻은 아니다. producer를 처음 도입할 때 출처 identity를 잃지 않고,
지원하는 상태 종류를 공개한 순간 그 조회/집계 계약도 실제 경로로 검증한다. 정의만 남기고 '지원'으로 표시하지 않는다.
이 배정은 후행 실행 승인이나 기존 R11/R17 동결 카드의 소급 수정이 아니다. 해당 R의 계획 인수 때 이 요구를 소비한다.

## 7. 안전성·비용·별도 계획

[기존 비용 정책 변경](../../execution/single_goal/handoffs/R06_COST_POLICY_UPDATE_20260927.ko.md)을 그대로 계승한다.
64-session concurrent peak 9,030,188 bytes 초과만 경고/관찰이다. operations/retained/parity·hard runtime guard는 유지한다.
원 도구가 exit101을 내면 원인을 확인해 원 결과와 정책 판정을 따로 기록한다. 모든 exit101을 성공으로 바꾸지 않는다.
과거 실패를 지우거나 평균으로 가리지 않는다. 3MB 등의 새 상한도 정하지 않는다.

실험실/설정 RON/메트릭/모드 검증은 별도 후속 계획이며 현재 재개의 선행이 아니다.
안전 초과 시 스킬 자동 취소를 추가하지 않는다. 20분 시험은 현재와 후속 자동 검증 요구에서 제외한다.

## 8. 계획의 한계와 인계

이번 검토는 현재 구조를 읽고 정책·설계·검증 책임을 정한 것이다. 새 Rust 구현·제품 시험·동결 복원·독립 감사는 수행하지 않았다.
타입·필드의 정확한 이름은 구현 담당자가 기존 코드와 맞출 수 있지만 사용자 결정·오류 의미·identity/시점을 바꾸면 정책 owner로 반환한다.
원본 문서·동결 target·실패 기록은 보존한다. 변경되는 계획의 exact basis와 검증 결과는 이 디렉터리의 successor 산출물에 귀속한다.
