# 최소 작성·실행 지원 계약 — P2G-50 작성·의미 검토

2026-09-09 · Codex · NEXT-0007. P2G 단계의 외부 작업·검토 기록이며 독립 Exact Artifact Ref 또는 GEX 감사가 아니다.

## 1. 요청과 인수

사용자 요청 원문:

> core_enhanced/docs/roadmaps/content_authoring/ROADMAP_STATUS.md를 읽고
> 다음 단계를 진행해줘. 첫 인수에서 문서 배치와 P2G/GEX의 적용 버전도
> 확인하고, 작업 결과와 근거를 기록한 뒤 현황 문서를 갱신해줘.

시작 시 ROADMAP_STATUS v1.7 기록 0008의 **NEXT-0007**을 선택했다. `execute / bounded_pipeline`, 대상은 기존 `WPD-CAMEC-20260909`의 같은 최소 정책 범위이며 종료점은 complete P2G-50 Goal Set과 표준 검증 또는 필수 blocker다. 필요한 같은 범위의 문서 수리·안전한 진단은 포함한다. 게임 코드·RON·store·검증기·AGENTS·P2G/GEX 규칙 수정, 새로운 정책/권한 선택, P2G-60과 GEX는 제외한다.

작업공간은 `/mnt/f/work/simulator`, branch `cleanup/simulator-dirty-worktree-20260803`, HEAD `f318414fb0156a67e04c5479000c2a2683fc0c09`, dirty=true다. 시작 Git 항목 16,518개, 비교 경로 1,699개를 인수했다. [NEXT-0006 종료 검사](NEXT_0006_VERIFICATION.json)의 원본·도구·지침·현황 해시와 이전 비교 경로에 변화가 없었다. 선택 경로의 상위·하위 적용 위치에서 AGENTS는 발견하지 않았다. 기존 dirty의 구현 소유권을 이번 P2G에 귀속하지 않는다.

문서 배치는 기존 `docs/goals/content_authoring_minimal_effect_contract`가 정책/Goal/결과 원본, `docs/roadmaps/content_authoring`가 조회·다음 카드, `history`가 오래된 요약, `docs/tools`가 구조 검사 도구다. 별도 Topic이나 GEX execution 디렉터리를 초기화하지 않았다. [운영 규칙](../../roadmaps/content_authoring/OPERATING_RULES.md)의 원본 우선·최근 기록 5건 분리를 적용한다.

적용 지침은 [P2G r2](../../policy_to_goal_pipeline_revised_v2.md) `2026-09-09-r2`이며 배포 사본과 동일하다. [배치된 GEX](../../goal_execution_pipeline%281%29.md)는 `2026-09-09-gex-r1`, 기존 감사 basis는 별도 [archive](../../archive/goal_execution_pipeline.md)다. 네 파일 모두 이전 해시와 같다. P2G 원문의 옛 GEX 링크가 가리키는 경로 부재를 새 파일의 실제 적용 완료로 해석하지 않는다. 이번 적용은 실제 읽은 문서 기준이며 runner·서버·Unity·배포 적용은 미검증이다.

Reader Contract, P2G-50, §8.0/8.3/8.5/8.8, complete-result/current-selection 계약과 Goal template를 읽었다. GEX는 Reader와 §6.4/6.7의 검증 재사용·감사 분리 경계만 확인했다. 단계 실행은 P2G-50이다.

## 2. 재사용한 entry와 코드 근거

[P2G-40 complete result](P2G_40_RESULT_002.json)가 선택한 `WR-CAMEC-P2G20-001`, `FPF-CAMEC-P2G30-001`, `PP-CAMEC-P2G40-002`와 passed `PVR-CAMEC-P2G40-002`를 인수한다. 기존 Goal Set은 없다. applicable Change/active obligation은 없으며 정상 Goal 작성이다. predecessor 수리·정책 변경·P2G-00/10/20/30/40 재실행을 만들지 않는다.

[Review §3–5](CODE_REVIEW_001.ko.md)의 31개 source는 당시 hash와 모두 일치한다. 기존 CRAR Master의 `Complete / High`와 target 해시 대조도 이전 인수 근거로 보존한다. 이력의 게임 PASS 수를 이번 실행으로 세지 않는다. 현재 정책/Plan/도구 해시도 동일하므로 전체 후반 코드 조사를 되풀이하지 않고 해당 검토와 currentness를 재사용한다.

추가 읽기는 Goal의 검증 명령·문서 책임을 구체화하는 범위다. `game_core` package 이름, 기존 `run_battle`/`hit_gated_self_heal_and_buff_stack_repeat_attack_work_together`, `bundle_validation_returns_typed_skill_contract_errors`, `ValidatedGameData` 및 checkpoint capture/restore symbol 위치를 확인했다. `data/mod.rs` 805–819의 최종 검증 뒤 token 공개, `live_ron.rs` 1915–1940의 `repeated_valid_and_invalid_publication_attempts_are_idempotent` 본문과 `world/state.rs` 399–435의 capture/restore를 읽어 실패/재호출/저장 비변경 recipe가 새 정책 의미를 요구하지 않음을 확인했다. canonical `data_loading_contract.md` 1–133, `core_runtime_contract.md` 1–27/1104–1141, `skill_target_contract.md` 1–18/418–453의 책임·소비 경계 절도 확인했다. 새 API/fixture가 실제 존재하거나 시험이 통과했다고 간주하지 않는다.

인접 문서의 기존 불일치도 구분했다. `data_loading_contract.md`의 Durable runtime documents 단락은 core payload V4를 서술하지만, current `world/active_run.rs`의 V5 writer/V4·V5 reader는 기존 Review §3 및 Freeze TR30-02가 이미 인수한 actual이다. 해당 문서와 source는 이전 기준선 그대로이며 이번에 저장 정책을 새로 선택하지 않았다. 현재 Goal은 저장 포맷 변경·reader 제거를 제외하고 정의 index의 비영구성만 검증하므로 이 인접 문구를 새 필수 선행으로 확장하지 않는다. 저장 버전 문서/구현 정합성은 해당 저장 owner의 남은 문제다. 이후 작업이 그 경계를 바꿔야 한다면 Goal의 required-source/정책 반환 조건을 적용한다. 이번에는 그 문서를 교정하거나 V4/V5 호환을 인증하지 않는다.

## 3. Goal 의미 검토

검토 대상은 [Goal Set](P2G_50_GOAL_SET_001.ko.md)의 전체 member와 내부 매핑이다. 기준은 exact Freeze/Plan 및 Review이며 이 검토가 별도 정책 authority가 되지 않는다.

| 검토 항목 | 판단과 근거 |
| --- | --- |
| Goal 분해 | Plan의 유일한 `PHASE-CAMEC-01`에 Single member `GOAL-CAMEC-01` 하나. assigned node는 `AG-CAMEC-01`과 `WP-CAMEC-03` 그대로다. AG의 WP 01/02를 독립 완료 node로 펼치지 않는다. |
| 실제 선행과 순서 | 현재 opaque publication·SkillId·executor는 재사용한다. 새 작성 metadata/parser/catalog/runtime가 같은 AG에서 전환된 뒤 WP 03의 durable oracle이 이를 검증한다. 전체 VM·사본화·외부 팩·보스 graph는 이 최소 작업의 필수 선행으로 추가하지 않았다. |
| 정책와 수용 기준 | 6개 exact reviewed PER, 6개 DR와 12개 zero-based acceptance member를 Freeze에서 인계한다. Goal의 completion row는 시험상 관찰을 연결하며 원래 acceptance를 재정의하지 않는다. E04는 원형/추가 효과 metadata와 대상 판별의 최소 표현으로 검증하며 영구 유물 사본·부여 runtime 완료로 확대하지 않는다. |
| source/상태 owner | 작성 EffectSite/ActionSite/SourceMap과 runtime caster/cast/proc origin을 구분한다. 기존 token 공개, cast 종료·철수, HP-loss gate, signed Heal, ModifyDamage와 signal 의미를 유지한다. metadata index는 저장 사본 key가 아니다. |
| 실제 소비 경계 | compiler→validated token→application/battle와 fragment/equipment/preview/projectile/area를 required workspace 소비자로 연결한다. 외부 wire consumer와 배포는 별도 분류하며 이 Goal의 성공으로 보고하지 않는다. |
| 상세 설계 인계 | 정확한 Rust 타입·locator·schema·변경 symbol과 지원 조합별 기대값은 AG 구현자가 현재 호출자 조사를 통해 구체화한다. 책임/수명/전환 자체는 이미 Review/Plan이 정했다. 구현 전 준비 점검과 첫 패치 이후 관찰·교정·관련 재검증도 기존 Goal 필드에 포함했다. |
| 검증 소유 | Plan의 IVA 01–06 모두 WP 03 / 같은 Single Goal에 배정했다. API가 없는 현재는 recipe와 command 후보이며 구현 뒤 실제 selector·non-zero 결과가 필요하다. production 교정은 같은 Goal 내 AG 소유다. |
| 10개 dimension | 모두 현재 publication/실행/저장 비변경 경계에 적용된다. retry는 같은 입력의 재검증과 기존 cast 결과 보존으로 한정하고 새 네트워크 exactly-once를 만들지 않는다. persistence는 기존 capture/restore·개인 파일 비접근을 검증하며 새 포맷/연결 프로토콜을 만들지 않는다. |
| 구조·문서 완료 | 단일 publication authority, 실제 지원과 descriptor 일치, old/new authoring grammar 이중 경로 부재, 기존 다른 op와 spatial consumer 보존을 완료 조건으로 연결했다. 미래 전용 VM/adapter와 fixture 전용 production 분기를 요구하지 않는다. 변경 책임은 기존 canonical 문서의 가까운 owner에 반영하도록 했다. |
| 위험·검토 | schema/catalog/runtime 공동 전환의 출처 단절·부분 공개·기존 게임 결과 변경 위험에 대비한 준비 점검과 fresh GEX-20 review를 인계한다. 이 단계에서 reviewer 지정·Goal activation·구현 승인을 하지 않는다. 현재 범위 밖 의미/Phase/필수 consumer 문제가 생기면 원래 P2G owner로 반환한다. |

Shared Validation Baseline Reuse Directive는 이 Topic에 applicable한 사용자 승인 원본이 없다. 다른 Topic의 directive를 가져오지 않았고 initial/final 횟수 계약을 새로 만들지 않았다. 대신 변경 input→검증 경계 map과 GEX §6.4의 evidence currentness·focused 재검증 원칙을 Goal에 연결했다.

위 대조에서 새 material policy, Phase 변경 또는 필수 source/consumer blocker는 발견하지 않았다. 구조 검증의 pass와 이 문서의 의미 검토는 서로 다른 근거다. 전자는 필수 표준 도구 실행 결과로 기록한다.

## 4. 실행 결과와 종료

표준 `validate_p2g50_goal_set.mjs --quality-contract 2`를 실제 실행해 exit 0 / `passed` / `failures: []`를 얻었다. validator는 `p2g50-goal-set-validator/1.3.0`이며 Freeze 6 / Phase 1 / final node 2 / Goal 1 / trace 6 / integration 6 / validation dimension 10을 검사했다. exact PER별 Decision과 12개 acceptance member를 별도로 대조한 근거는 [실제 검증 기록](P2G_50_VALIDATION_001.json)에 둔다. 검증기 회귀 37개는 NEXT-0006의 unchanged 도구 근거로 인수했으며 이번에 다시 실행했다고 기록하지 않는다.

complete [P2G-50 결과](P2G_50_RESULT_001.json)에서 필수 구조·의미·참조 검증을 결속한 뒤 `GS-CAMEC-P2G50-001`을 current로 선택한다. member는 `GOAL-CAMEC-01`, 상태는 `Authored / Not Activated / Implementation Authorized: No`다. 현 요청의 종료점은 여기까지이며 다음 NEXT-0008은 같은 basis의 P2G-60 Handoff 작성 요청용 카드다. 그 단계는 실행하지 않는다.

게임 테스트·production/RON/store 변경·P2G-60 Handoff·GEX 감사는 수행하지 않는다. 종료 시 보존·참조·링크 검사와 실제 다음 카드 미실행 여부를 [NEXT-0007 종료 검사](NEXT_0007_VERIFICATION.json)에 기록한다.
