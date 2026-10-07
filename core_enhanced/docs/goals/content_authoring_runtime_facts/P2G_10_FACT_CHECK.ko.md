# R04 P2G-10 제한적 정책 사실 확인

2026-09-11 KST. 정식 P2G-20 Source Observation/Review/Binding이 아닌 Finding Triage 근거다. production·test·store는 바꾸지 않고 새 build/test/probe도 실행하지 않는다.

Evaluation Basis: Input `WR-CARF-P2G00-001`, change request N/A, unchanged intake `PIS-CARF-P2G00-001`. 현재 사용자 입력·scope 변경은 없다. 아래 Candidate ID는 P2G-10의 식별이며 아직 정식 Policy Element로 승격하거나 disposition을 결정하지 않았다.

| Candidate | 확인할 의미와 필요한 사실 | 조사 종료 조건 |
| --- | --- | --- |
| C-CARF-01 | 실제 Damage/Heal/Cost/Death와 optional dispatch 분리. signed Heal, lethal 순서와 현재 비용 writer가 무엇인지. | 실제 mutation/finalization/trigger/cost writer와 호출 구간을 식별하고 기존 의미 보존으로 닫을 수 있는지 판정. 새 HP 비용/방어 정책은 만들지 않음. |
| C-CARF-02 | source/action/strike/segment/pre-death 사실. 현재 다중 segment 경계와 원래 쐐기/냉기 요구, HP-loss gate·pre-death 시점에 선택이 필요한지. | 기존 타격 묶음과 승인 원문의 관찰 대상을 확인. 새 zero-HP 타격 인정·동시 우선순위 결정은 추정하지 않음. |
| C-CARF-03 | 현재 cast/step/projectile/area의 접수·방출·완료·취소. pending와 released 전달체의 실제 취소·실패·상태 정리 및 결과 수명. | 상태 writer와 cleanup 규칙 및 attempt 경계를 확인해 관찰 계약과 새 gameplay 수명 선택을 구분. |
| C-CARF-04 | 장비의 실제 persistent copy→draft→RuntimeItem 출처. | 실제 보유 ID와 현재 정보 유실 변환 지점, 파편 사본 부재를 확인. 기존 저장 정책을 바꿀 필요가 있는지 판단. |
| C-CARF-05 | 새 상태의 attempt 수명/rollback/저장/예산·결정론 및 최소 검증. | 현재 공개 operation·snapshot·attempt entry/result/drop 소유 구간 확인. 구현 가능성 전체와 coverage 판정은 P2G-20으로 남김. |

이미 원래 R04 카드와 K01/K04/K06/K08 및 PG02–04/08을 읽었다. 기술 권고의 exact type명이나 현재 구현을 자동으로 사용자 gameplay 결정으로 승격하지 않는다. 조사 전 parent lookahead는 재사용 가능한 원시 단서이며 완료 판정이 아니다. 이후 기록은 exact source byte identity·relevant dirty·실제 읽은 구간·방법·관찰·미확인을 포함한다.

## 관찰과 정책 판정

Exact source identity, relevant dirty와 실제 읽은 구간/원문은 [fact evidence](P2G_10_FACT_EVIDENCE_001.json)에 보존했다. 아래 제한된 확인은 종료했다. 후속 P2G-20은 자기 exact basis의 currentness와 필요한 전체 coverage를 따로 검토해야 한다.

| Candidate | 관찰한 사실 | 판정과 미확인 경계 |
| --- | --- | --- |
| C-CARF-01 | `execute_committed_damage`는 basic에만 OnAttack/OnHit plan을 모으고 actual HP-loss를 계산한다. HP 적용에서 사망 finalize까지 호출한 뒤 남은 formula/activation batches를 실행한다. `ApplyHeal`은 flat+maxHP×percent의 signed delta이며 음수도 HP를 줄이고 기존 actual_damage_target_count를 증가시킨다. Auto/ManualCastEnd에 현재 공명 값을 0으로 만드는 실제 자원 소비가 있다. 현재 authoring observation의 Resonance는 command mutation만 다루고 cast reset은 제외한다. | 기존 formula·gate·반응 순서를 보존하면서 실제 mutation receipt를 더하는 요구다. 음수 Heal은 Heal 요청과 음수 actual delta를 함께 보존하며 HP 비용으로 바꾸지 않는다. 현재 공명 소비는 Cost receipt의 실제 producer이고, 새 HP 비용/흡수/HP1/패링 기능은 R04 달성에 필요하지 않다. 향후 HP 비용을 Damage로 가장하는 승인은 없다. |
| C-CARF-02 | 현재 damage breakdown은 한 계산의 기본/bonus component들을 합산한다. 하나의 committed basic occurrence에서 이를 별도 기본공격으로 실행하지 않는다. cast_seq는 별도지만 pending start_seq와 원인 연결 일부는 log seq다. death finalize는 movement/dead 처리→wave death epoch→buff cleanup→graveyard/log→optional death/kill/allydeath 순서다. cleanup 전 buff key에는 caster와 target가 있다. 원래 요구 기록은 쐐기 n을 cast·target·strike별로 구분하고 zero-damage 인정은 미정, 눈여왕은 자기 원천의 유효 냉기 중 다른 직원 kill을 구분한다. | source/occurrence 사실과 정리 전 상태를 보존하는 것은 승인된 요구다. 각 실제 strike와 component/segment를 구별하되 breakdown을 새로운 타격으로 세지 않는다. buff 정리 전 상태 관찰을 추가할 수 있으나 새 동시각 우선순위·HP0 타격 자격·냉기 콘텐츠 proc·쐐기 수식 자체를 정하지 않는다. R04에서 producer와 관찰 검증을 닫고 R06/R14/R16 등 실제 rule consumer는 자기 정책으로 닫는다. 이 분리는 현행 HP-loss gate를 보존하며 R04 producer 요구를 제거하지 않는다. |
| C-CARF-03 | 수동 요청 검증은 lock/pending write 전이다. focus=0도 release를 다음 1ms에 예약한다. 현재 invoke 함수는 일부 실패에서 void return한다. cast step terminal은 started+pending delivery=0이고 cast cleanup은 모든 step/area/deferred가 끝나야 한다. hard CC는 pending와 basic epoch를 취소한다. withdrawal은 active cast를 제거하지만 projectile와 GroundFixed area의 독립 수명을 모두 제거하지 않는다. | fallible request/milestone 관찰은 현재 경계를 드러내는 R04 승인 기능이다. 취소된 제어 흐름과 남은 전달체의 settlement를 분리해야 한다. 새 cancel이 방출된 payload를 되감거나 방출 전 실패가 완료로 보이면 요구 위반이다. 정확한 내부 자료구조·terminal 보관/조회 구현은 P2G-20/GEX 기술 설계이며 silent eviction·무제한 수명·새 retry 정책으로 의미를 메우지 않는다. |
| C-CARF-04 | player_spawns의 실제 inventory lookup은 instance_uuid를 쓰지만 양쪽 draft 경로가 base_uuid만 수집한다. BattleUnitDraft.equipped_items는 Vec<Uuid>이고 양쪽 runtime build가 base UUID+unit/index에서 transient item instance를 만든다. | 실제 보유 ID를 source binding으로 전달하는 AT04 요구다. persistent copy ID와 transient runtime ID는 별개이므로 RNG/cooldown 등의 기존 runtime identity를 무단 교체할 이유가 없다. 기존 inventory가 이미 ID를 보유하므로 새 사본을 만들어 저장하거나 파편 ID를 위조할 필요가 없다. 전체 constructor/consumer coverage와 검증은 P2G-20 책임이다. |
| C-CARF-05 | BattleCore는 Clone이며 explicit CloneShape가 runtime fields를 열거한다. reset_runtime_state_for_battle이 active cast/projectile/area를 지운다. BattleEntrySnapshot은 inventory/roster/run 등을 소유하고 restore에서 active_battle=None으로 만든다. retreat는 active를 take한 뒤 entry를 복원한다. 기존 제작 trial은 candidate와 유한 work/native provenance를 성공 때 채택한다. | R04의 새 runtime 상태는 승인된 attempt-local 수명으로 같은 rollback/reset/drop 경계를 따라야 한다. live reconnect 관찰과 durable checkpoint는 구별한다. 이 producer 기능만을 위해 새로운 Run 저장 상태·손실/마이그레이션 정책을 도입할 필요는 확인되지 않았다. 새 state가 실제 clone/entry/result/공개 operation에서 빠짐없이 처리되는지는 P2G-20/GEX에서 검증할 의무이며 현재 완료 주장 아님. |

## 남는 작업과 질문 여부

현재 선택된 R04 producer 범위에서 새 material policy를 결정할 필요는 확인되지 않았다. 이미 승인된 관찰/출처/현재 전달체/attempt-local 의미를 결정 기록에 연결한다. 이는 미정 gameplay를 해소했다는 뜻이 아니다. PG02의 true join·정기 root, PG03의 새 zero-HP 타격 인정·콘텐츠별 pre-death 반응/동시 순서·침식 cast-start, PG04의 방어·HP비용·HP1·특수kill은 필요한 후속 consumer owner에서 그대로 남는다. P2G-20이 R04 in-scope 구현 자체에 새 observable 선택이 필요함을 발견하면 해당 요소를 P2G-10으로 반환한다. 의무 구현·시험을 후속으로 미루는 조건으로 현재 정책 폐쇄를 주장하지 않는다.
