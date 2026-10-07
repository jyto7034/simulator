# R04 사건·실제 결과·출처·ActionHandle — Working Policy

2026-09-11 KST · P2G-60 technical repair complete; eligible successor GEX reintake next. 이 문서는 mutable Working Revision view이며 정책·구현 완료 증거가 아니다.

## Metadata와 요청 경계

- Working Policy Document ID: `WPD-CARF-20260911`
- Policy Topic: R04 사건·실제 결과·출처·현재 전달체의 ActionHandle
- Target Documentation Root: `core_enhanced/docs`
- Policy Directory Name: `content_authoring_runtime_facts`
- Current Physical Path: `core_enhanced/docs/goals/content_authoring_runtime_facts/WORKING_POLICY.ko.md`
- Current Working Revision / Checkpoint: `WR-CARF-P2G20-002`; exact predecessor WR001/Freeze001 inherited with unchanged5PER/5DR/20acceptance and PFC001/COD002 basis. [Owner entry preparation](P2G_20_WORKING_PREPARATION_002.json). Current planning Freeze `FPF-CARF-P2G30-003` with all5membership rows in [complete result004](P2G_30_RESULT_004.json).
- Policy Intake Snapshot: `PIS-CARF-P2G00-001`.
- P2G-10 Completion Predicate: true. [Policy payload](P2G_10_POLICY_001.json)의 5 decided elements/5 current Decisions, 7 candidate dispositions, current open questions 0. [검증·분류·결정 view](P2G_10_RESULT_001.json).
- Requested Result / Progression: 사용자 채택 AUTORUN의 R04 최소 `bounded_pipeline`, P2G-60 Handoff까지. 다음 단계가 실제로 닫혀야 그 결과를 인수한다.
- Stage Selection Basis: Authorized Endpoint Prerequisite; R03 Single Goal actual Complete/High 인수 뒤 상위 요청이 R04를 선택했다. [Parent 확인](PARENT_PREDECESSOR_CHECK_001.json).
- Current Stage: P2G-60 repair complete. Current `WR-CARF-P2G20-002` → `RB-CARF-P2G20-003` / RSP004 / RSB003 / 5CRB003 → `FPF-CARF-P2G30-003` → `PP-CARF-P2G40-004` / passed PVR004 → `GS-CARF-P2G50-002` / GOAL-CARF-01 → `HOF-CARF-P2G60-002`. Sole declaration COD002 via DS001; O20/30/40/50/60 resolved by respective RR002 and complete owner results. [Current Handoff result](P2G_60_RESULT_002.json). Authored/Not Activated/Implementation Authorized No preserved; parent AUTORUN separately selects eligible GEX successor.
- 권한: [채택 원문](../../roadmaps/content_authoring/AUTORUN_REQUEST.ko.md) §§3/5/6/8. 문서·동결·Goal 작성은 GEX 실행 권한을 스스로 만들지 않는다. 실제 exact Goal/Handoff가 존재하고 자격을 갖춘 뒤 별도 GEX로 인수한다.
- P2G 허용 작업: 이 Topic의 필수 계획·source 검토·안전한 구조 검사. Production/test/RON/canonical contract mutation, 배포·전역 변경은 이 P2G pass에 포함하지 않는다. 새 material policy 질문과 실제 권한·안전·자원 한계는 원문대로 중단한다.

## 승인 initial scope

아래는 [initial Working Revision](P2G_00_INITIAL_001.json)의 Approved Metadata and Scope projection이다. 각 항목의 exact approval은 같은 payload의 Approval Reference와 채택 원문/R04 카드에 있다. 정책 의미의 폐쇄·code-informed 검토는 다음 owner 단계의 책임이다.

| ID | Disposition | Initial scope |
| --- | --- | --- |
| I01 | in | R04 current damage/heal/cost/death fact generation and actual mutation/optional reaction consumer boundaries; preserve mandatory death/wave bookkeeping, current basic OnAttack/OnHit and existing accepted gameplay meanings. |
| I02 | in | R04 source and distinct current action/strike/segment facts, pre-death observation and actual result receipts independent of debug log retention; K01/K06/K08 ports within the R04 card. |
| I03 | in | R04 fallible request and bounded ActionHandle/epoch/milestone/terminal contracts for current cast/step/projectile/area; Accepted/Released/FlowFinished/EffectsSettled distinction and current cancellation/released-delivery behavior, without requiring later Flow or boss adapter implementation. |
| I04 | in | R04 actual persistent equipment copy identity through inventory/player-spawn draft/RuntimeItem source binding (AT04); no fabricated fragment copy identity before its R07 owner. |
| I05 | in | R04 introduced attempt-local state ownership/lifetime, rollback/entry/result/retreat/drop and applicable save/reconnect semantics, bounded resource/determinism/cost checks and actual T07/T23/T26/T28 acceptance; required current predecessor/direct consumer/core contract validation. |
| I06 | in | Minimum necessary P2G-00 through P2G-60 planning and structural/semantic validation for the selected R04 scope. P2G performs no production mutation; subsequent eligible exact GEX has separate parent authorization. |
| O01 | out | Server/Unity/client, external services deployment, RON editor UI, remote publication and global installation/configuration. |
| O02 | out | Unapproved new material gameplay/balance/UX/lifetime/data-loss/compatibility choices; ambiguous shield/HP-cost/HP1/parry/special-kill, join/root or timing meanings are not invented. Apply AUTORUN policy-question boundaries. |
| O03 | out | Later R05–R24 implementations beyond the required R04 producer/consumer/lifetime boundary, including full Flow/boss task adaptation, fragment copy binding and new relic composition. Their parent requirements remain uncompleted, not removed. |
| O04 | out | Personal store deletion/automatic conversion/overwrite, content-name executors, fixture-only production success, unsupported placeholder/fallback, external arbitrary script/OS/network authority. |
| O05 | out | P2G/GEX/validator/AGENTS/runner/global Codex modifications or new approval authority created by artifacts. |

## Bootstrap intake와 다음 owner

[Initial artifact container](P2G_00_INITIAL_001.json)는 user raw request와12개 exact repository content refs를 의미 판정 없이 보존한다. Type/ID envelope와 payload를 분리하고 Content Digest는 N/A다. Bootstrap 접근 확인은 P2G-20 Source Observation이 아니다. Parent의 이전 read-only lookahead도 이 initial policy basis에 code-reviewed 상태를 만들지 않는다.

[P2G-00 result](P2G_00_RESULT_001.json)는 실제 path 미점유·identity 검색·empty initial basis·승인 scope/reference 확인을 소유한다. P2G-00 당시 Candidate/Disposition·Policy Element/Decision·Review/Freeze/Phase/Goal/Handoff는 없었다. 이후 [P2G-10](P2G_10_RESULT_001.json)이 정책 요소와 기존 승인의 의미를 결속했다. 이후 [P2G-20](P2G_20_RESULT_001.json)이 같은 승인 scope의 source/consumer 검토를 완료했다. [P2G-30](P2G_30_RESULT_001.json)이 같은 5 members의 binding/acceptance/transition disposition을 검증해 final policy closure를 완료했다. [P2G-40](P2G_40_RESULT_001.json)은 최소 유효 단일 Phase 설계와 표준 receipt·별도 의미 검토를 완료했다. [P2G-50](P2G_50_RESULT_001.json)은 quality contract2 구조 검사와 별도 root 의미 검토를 완료했다. [P2G-60](P2G_60_RESULT_001.json)은 durable Handoff를 완료했다. 상위 AUTORUN이 별도로 exact eligible Goal의 GEX00을 선택한다. 새로운 사용자 메시지를 기다릴 단계 전환 사유는 없다.

## Current Policy Element Registry

| Element | Current revision | 의미 |
| --- | --- | --- |
| CARF-E01 | `PER-CARF-E01-REVIEWED-001` | 현재 Damage/Heal/Cost/Death 요청과 실제 확정 결과를 구별하고 실제 HP 변화·유효 회복·자원 소비·사망 사실을 제공한다. 기존 기본공격 OnAttack/OnHit·HP-loss gate·계산/반응 의미는 보존한다. 필수 사망/웨이브 처리는 optional 전투 반응의 억제와 분리한다. |
| CARF-E02 | `PER-CARF-E02-REVIEWED-001` | 정의·효과·제공자와 actor·action/cast·직접 strike·damage segment·target occurrence를 구별해 실제 결과의 출처와 인과를 보존한다. 사망에 의해 지워질 관련 상태를 사망 처리 전에 관찰할 수 있고 core 사실은 narration/log retention에 의존하지 않는다. |
| CARF-E03 | `PER-CARF-E03-REVIEWED-001` | 현재 cast/step/projectile/area 요청은 거부/실패를 명시하고 bounded ActionHandle과 epoch로 접수, 방출, 제어 흐름 종료, 효과 정산 및 취소/실패를 구별한다. 현재 미방출 중단과 이미 방출된 전달체의 독립 수명을 보존하며 stale 완료가 새 실행을 전이시키지 않는다. |
| CARF-E04 | `PER-CARF-E04-REVIEWED-001` | 장비의 실제 영구 사본 ID를 inventory/player draft에서 RuntimeItem와 효과 source binding까지 보존한다. 정의 ID·runtime item ID·actor와 영구 사본을 구별하고 아직 없는 파편 사본을 만들지 않는다. |
| CARF-E05 | `PER-CARF-E05-REVIEWED-001` | R04가 도입하는 runtime 상태는 attempt-local 소유·수명과 실제 공개 operation의 rollback/실패, entry/result/retreat/drop 및 해당 저장·재접속 경계를 함께 처리한다. 기능 도입부터 유한 예산·결정론과 직접 검증을 갖추고 기존 유효한 선행/consumer 계약을 보존한다. |

모든 current element는 기존 `PER-CARF-E01..05-REVIEWED-001`이다. [Current immutable review003](P2G_20_REVIEW_003.json)은 successor WR002에5exactbindings와9freshobservations(7required/2N/A)를 결속한다. [CRA003](CODE_REVIEW_003.ko.md)의 source correction은 정책·Decision·20acceptance를 변경하지 않는다. [Root exact checks](REPAIR_REFERENCE_CHECK_001.json), [별도 의미 검토와 교정](REPAIR_SEMANTIC_REVIEW_001.json)을 확인했다. 새 production/runtime tests 없음.

Original P2G00–60와 최초 GEX00/10 인수 기록은 historical이다. COD001→COD002의 [DS001](DECLARATION_SUCCESSOR_001.json)이 유일한 declaration leaf를 고정한다. [철회된 same-WR repair candidates](REPAIR_CANDIDATE_DISPOSITION_001.json), membership 누락 RESULT003의 [complete replacement RESULT004](P2G_30_RESULT_004.json)를 current로 혼용하지 않는다. 원본 bytes는 보존했다.


## Current source-premise repair · 2026-09-11

[COD002/DS001](DECLARATION_SUCCESSOR_001.json)은 PFC001의 동일12original artifact invalidation과 empty policy closure/O20→30→40→50→60을 보존한다. WR002가 owner entry에서 predecessor WR001/Freeze001과 새 Change basis를 exact 상속한다. O20/O30/O40/O50/O60은 current complete results와 각 RR002로 해결했다. [GF001/002](GEX_FINDING_001.json)와 EXP003은 GEX10의 owner 반환을 보존한다. Old GS001/HOF001은 stale 이력이며 current GS002/HOF002의 explicit GEX 인수 전 구현하지 않는다.
