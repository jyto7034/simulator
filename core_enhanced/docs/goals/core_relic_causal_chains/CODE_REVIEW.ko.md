# U10 Code-Informed Review

검토 입력은 `WR-CRCE-002`, 범위는 `RSP-CRCE-001`이다. Root Astra가 아래 실제 source를 읽었고 독립 시험 Astra의 읽기 전용 경계·시험 제안을 보조 입력으로 사용했다. 현재 구현의 부재와 승인된 목표를 구분한다. 이 검토에서 제품·시험 코드 변경과 시험 실행은 없다. 정확한 bytes와 artifact 결속은 P2G-20 JSON 및 source archive가 소유한다.

## 현재 상태와 최소 동시 전환

| 경계 | 현재 실제 | 필요한 변경과 검증 |
| --- | --- | --- |
| RON→유물 publication | `RelicDefinition.rules`는 Grant/Edit/Recipe만 받는다. `validate_contract`→`validate_rules` 및 `GameDataBase` publication→`validate_publication`이 실제 진입점이다. Inline/FiniteFlow grant는 있다. | 사건 구독·독립 periodic·실제 join·쐐기를 실행까지 이어지는 typed 공개 작성 기능으로 추가한다. 필드 추가만으로 성공하지 않는다. identity/reference/type/interval/target/payload 부적합은 명시 진단한다. |
| 활성 plan→World→Battle | `compile_effective_plan`은 copy/variant와 provider를 받아 immutable generation plan을 만든다. `capture_structural_battle_owner`→`World::combat`→`BattleCore::set_effective_owner`가 실제 소비 경로다. query는 같은 plan의 owned DTO다. | 사건 구독은 적용 대상 효과/제공자 조건과 해당 사본에 결속한다. 기본 공격·사망·독립 주기처럼 특정 active fragment 없이 가능한 사건을 단순히 provider가 없다는 이유로 누락하지 않는다. 생성 피해에도 tags/source와 적격 modifier를 연결한다. 새 기능은 DTO/capability 및 retained proof에도 나타나야 한다. |
| 실제 피해→사건 | `execute_committed_damage_inner`가 `admit_damage_strike` 후 formula/protection을 계산하고 `prepare_damage_fact`를 거쳐 `apply_damage_result_and_record`에서 HP와 actual fact를 commit한다. subscriber는 없다. | commit된 fact의 실제 HP 감소·actual killer를 사용해 subscriber를 예약한다. formula 예상량, 로그 sequence, 조회는 발동 authority가 아니다. 실제 성공을 기록한 transaction 안에서 child enqueue까지 처리한다. |
| source→queue→dispatch | `RuntimeFactSource`에는 action/delivery/reaction/effective/resource 정보가 있지만 causal ancestry/join은 없다. `enqueue_finite_effect`는 lease를 예약하고 queue completion으로 넘긴다. `process_owned_event`가 source를 설치하고 완료 lease를 닫는다. | immutable 경로 ancestry와 명시 억제 의미를 source에 담고 queue/실제 carrier를 통해 전달한다. 기존 delivery/resource 권한과 별개다. source 관찰 DTO가 실행 권한을 만들지 않도록 한다. `commands.rs`의 수동 재구성 및 TriggerProc의 `Default` 재시작을 함께 수정해야 한다. |
| fork/join | Flow Sequence/Parallel은 `SkillStepResult.merge`를 수행하지만 causal participant receipt/union은 없다. 구조형 side flow는 기존 cast source와 finite lease를 보존한다. | 공개로 실제 가지들을 생성하고 정해진 참여 결과를 소비해 후속 효과를 만드는 명시 join producer가 필요하다. 일반 Parallel·같은 timestamp·합산 표시를 자동 join으로 바꾸지 않는다. duplicate receipt, nonparticipant, partial completion, cancel/retry를 검증한다. |
| periodic/파생 | BuffTick 재예약은 `runtime_fact_source.observation()`을 넘긴다. 이 함수는 delivery/resource 실행 권한만 제거하고 관찰 provenance를 남긴다. 독립 유물 periodic producer는 없다. | 독립 periodic은 battle/owner-bound producer가 각 발생을 새 원인으로 발행한다. 기존 파생 buff tick/projectile/area/delay는 원인 ancestry를 유지한다. 작성자가 timer flag로 원인을 위조할 수 없다. |
| 장비 counter/death | committed BasicAttack 분기만 OnAttack/OnHit를 수집한다. `death_trigger_commands`는 OnDeath→actual killer OnKill→정렬된 living ally OnAllyDeath다. Corrosion은 `finalize_prepared_death_with_dispatch(..., false)`로 필수 death만 처리한다. | 유물 사건을 이 실제 경계에 연결하고 기존 counter 및 새 피해자 사망 연쇄를 유지한다. suppression은 실제 소비 지점에서 검사한다. `reaction` 하나를 모든 종류의 사망 연쇄 금지로 확대하지 않는다. |
| 공명 | `modify_resonance_inner`의 양수는 `add_resonance`→`resonance_gain_blocks`로 가고 음수는 별도 감소한다. Employee use obligation이 실제 양수 gate다. | 새 payload도 같은 명령 경로로 보낸다. 잠긴 양수를 보관하거나 나중에 지급하지 않는다. |
| 쐐기 | R06의 `StrikeScope`가 실제 target별 `StrikeProgress`를 공유하여 같은 strike의 여러 segment를 구분한다. `first_positive`도 존재한다. `ValueRef::Result`는 EarlierEffect/CurrentActionPrefix 집계만 제공한다. 쐐기 target/cast별 runtime consumer는 없다. | 원본 actual strike를 쓴 cast+target 누적이 필요하다. 전체 action prefix+1, 예정 step, global actor counter는 부적합하다. formula 전에 후보 n을 적용하고 HP 감소 성공 시 단 한 번 확정한다. 같은 strike 후속 segment는 확정 n을 공유한다. bound copy별 r과 기존 고정 증가/비율 합산 순서를 보존한다. |
| 자원/원자성 | `runtime_transaction`/`runtime_timestamp_reusing`은 candidate clone→operation→retained proof→commit 구조다. `BattleCore`의 명시 CloneShape/clone, source retained visitor, queue slot/payload, runtime native preflight가 분리돼 있다. | 모든 새 mutable root를 candidate 안에 두고 clone 독립성·allocation 전 admission·retained/scan/copy 비용·실패 회수를 닫는다. source Arc 공유를 이유로 새로운 가변 전역 상태를 만들지 않는다. |
| 저장/복원 | `RunCheckpointPayload`는 relic inventory와 run state를 저장하며 `ActiveBattleSession`은 별도 transient owner다. Continue는 전투 중 pre-entry immutable root를 보존하고 stable phase에서 갱신한다. | ancestry/join/periodic 진행/쐐기 누적은 battle transient다. run copy/variant/plan identity는 기존 durable 경로를 유지한다. 새 mid-battle durable save 계약은 도입하지 않는다. fresh Continue, attempt/retry/retreat와 in-memory clone은 각자의 실제 경계를 검증한다. |

## 제안 구조와 중요한 설계 조건

기존 relic structural compiler와 battle fact/source/queue 사이에 작은 causal authoring/runtime 모듈을 둔다. 독립적인 두 번째 전투 실행기를 만들지 않고, 원래 damage/resource/target/lease consumer를 호출한다. 추가 공개 표현의 정확한 Rust/RON 이름은 구현 상세이며 다음 계약을 고정한다.

1. **정체성:** 방문 key는 persistent relic copy와 authored effect identity다. employee UUID, form, plan fingerprint, Arc 주소는 방문 identity의 대체물이 아니다. 동일 effect는 variant/owner 재결속으로 방문을 초기화하지 않는다. generation/실행 권한 검사와 방문 identity 검사는 별개다.
2. **사건 문맥:** actual event, subscriber copy/effect, 원본 actor/target/killer/source와 immutable ancestry를 구분한다. 필수 source와 suppression은 모든 child 재구성에 전달한다. 새 child의 damage source를 자기 실행 소유자에 결속하되 원인 provenance를 잃지 않는다. explicit target이 더 이상 유효하지 않으면 기존 의미대로 취소/skip하고 임의로 다른 대상을 고르지 않는다.
3. **queued continuation:** subscriber는 현재 call stack에서 무한 재귀하지 않고 admission을 거친 queue continuation을 만든다. fork는 ancestry를 값으로 분기한다. join은 자기 producer가 만든 participant identity/receipt만 받아 정확히 한 번 후속 결과를 예약한다. 가지 결과를 실제 소비하지 않는 helper-only 합집합은 완료 근거가 아니다.
4. **공개 join 작성:** 명시 producer의 가지와 후속 효과를 RON으로 정의할 수 있어야 한다. authored stable effect를 식별할 수 있고 반복 invocation은 같은 효과 identity를 유지한다. participant마다 실제 실행/실패/취소가 관찰되며 join의 own damage는 합집합에 의한 subscriber 차단과 별개로 실행한다. 구체 문법·자료구조는 이 의미 안에서 구현자가 선택한다. Flow Parallel 전체를 의미 변경시키는 방식은 선택하지 않는다.
5. **독립 periodic:** positive interval의 독립 owner-bound 예약을 사용한다. 빈도는 콘텐츠 값이고 gameplay 공통 cap은 아니다. active owner가 사라지면 다음 독립 발생 예약을 끝내며 이미 release한 child는 원래 수명을 따른다. derived duration/tick은 같은 causal context다.
6. **쐐기:** direct multihit eligibility와 per-cast/per-target positive-strike count는 typed runtime modifier로 구현한다. ordinary numeric modifier와 같은 산술 지점에 합산해야 한다. generic prefix result에 의미를 몰래 더하거나 피해를 계산한 뒤 소급 수정하지 않는다. first-positive 실제 strike witness를 재사용하고 released attack이 끝날 때까지 필요한 count를 보존한다.
7. **수명/회수:** ancestry 값은 살아 있는 carrier edge가 소유한다. join registry는 실제 미완료 참여자·receipt만 보유하고 complete/cancel/expiry에서 닫는다. periodic registry는 살아 있는 producer만 보유한다. 쐐기 cast root는 active/released direct strike consumer가 끝나면 회수한다. battle 종료에서 모두 비워진다. 공개 diagnostic은 owned observation이며 mutation authority가 아니다.
8. **원자성/계량:** ancestry insert/union/clone, subscriber scan/fanout, queue payload, join receipt/commit, periodic reservation, cast-target counter의 실제 비용을 기존 두 budget 계층에 반영한다. 한도 부족·cancel·overflow는 기존 typed fault로 전파한다. 삼켜진 queue 실패 뒤 성공 표시나 부분 방문 commit을 허용하지 않는다.

위는 승인 정책을 구현하는 제안이며 현재 작동한다는 주장이 아니다. 실제 source 읽기에서 정책 충돌은 발견하지 않았다. 새 API 이름·분해 방식·primitive reuse의 세부는 GEX 설계와 Sol 구현에 맡길 수 있다. 정책/공개 의미·상태 소유·수명·acceptance를 바꿔야 하는 발견은 P2G owner로 반환한다.

## 정책 요소별 검토와 검증 연결

| 요소 | source requirements | 현재 delta와 수용 seam |
| --- | --- | --- |
| PE01 actual subscriber | RQ01/02 | 정상 RON DamageHpLost·child 피해/자원, HP-loss vs shield/overkill, query 비발동 |
| PE02 copy/effect | RQ01/02 | A1.e→B1.e→A1.e, A2/A1.f, 새 basic root, 동일 copy 재결속 |
| PE03 fork | RQ02 | 공통 조상 보존·형제 이후 방문 독립, 같은 시각 별도 hit |
| PE04 join | RQ01/02 | 실제 participant union·nonparticipant·join own damage·unvisited·duplicate/retry/cancel |
| PE05 periodic | RQ01/02 | 실제 독립 두 발생 vs derived delay/projectile/area/DOT |
| PE06 lifetime | RQ02/04 | trace off/on/eviction, pre/post release owner removal, 마지막 consumer 회수·clone 독립 |
| PE07 counter | RQ03 | 기본 공격만 장비 발동·한 번 반격·dead/missing attacker, 일반 효과 우선 |
| PE08 death | RQ03 | 새 피해자 death chain·same-death dedup·actual killer·sorted ally, Corrosion optional suppression |
| PE09 resonance | RQ03 | use 동안 양수 0·음수 허용·cap discard·bank 없음 |
| PE10 wedge arithmetic | RQ01/02 | 110/120/260 fixture, cast/target/copy 분리·같은 종류 비율 합산·original source/type |
| PE11 wedge strike | RQ01/02 | HP positive만 확정·full shield/zero 제외·같은 strike segment 한번·동시 별도 strike |
| PE12 wedge lifetime | RQ02/04 | released source/cast count 보존·cancel/finish 회수·새 cast reset |
| PE13 budgets | RQ04 | actual producer one-short faults, prefix/rollback/retry, retained/clone/queue accounting |
| PE14 public/restore | RQ01/04 | 외부 RON/typed negatives·fresh Continue/attempt/retreat·DTO/capability 소비 |

RQ05는 전 요소의 독립 입력·oracle·회귀 선택, RQ06은 전 요소의 정확한 정책·scope·선행 완료 근거다. 테스트 파일 존재를 실행 성공으로 취급하지 않는다.

## 검사 범위와 재사용

독립 시험 Astra의 8개 검증 묶음은 위 수용 seam과 일치한다. 기존 World `setup_with_skill_input`, 외부 `public_amount_trial_fixture`, 실제 fragment/relic 장착, `native_action_fixture`, `runtime_transaction` 및 debug snapshot을 우선 재사용한다. 내부 ancestry helper 주입은 원자성의 보조 seam일 수 있으나 정상 producer·공개 경로를 대체하지 않는다.

직접 영향 회귀는 actual strike/segment, log eviction/source, general damage의 basic-trigger 비수집, counter/dead-attacker, death recipient order, Corrosion suppression, U05 positive gain 및 native released lifecycle다. U08 copy/저장과 U09 structural publication/generated plan은 새 공개 schema·plan 결속의 영향을 받으므로 관련 검증을 선택한다. 최종 명령·선택은 실제 변경을 기준으로 준비하고 Sol이 실행한다. 공유 hot source/queue/clone 크기 변경은 기존 대표 native workload와 resource limits를 비교할 구체 이유다. 무관한 전체 테스트는 단계 종료만으로 재실행하지 않는다.

U09 전체 독립 Complete/High/open0는 선행 계약 인수의 근거다. 새 U10 delta의 실행 성공을 증명하지 않는다. 기존 PC/R05 입력은 Employee relic 경로를 직접 소비하지 않으므로 새 유물 기능 oracle로 쓸 수 없다. hot source/queue 영향이 생기면 그 공통 경로 비용 회귀 측정에는 사용할 수 있으며 workload/한도를 바꾸지 않는다.

Released/Deployed actual 및 Unity·개별 최종보스는 이번 workspace 구현의 required source가 아니다. 외부 배포 호환을 추정하지 않는다. 현재 review는 접근 가능한 실제 Core와 정상 공개 작성/실행/저장 경계의 구현 계획을 닫는다.
