# U17 독립 완료 감사

**판정: Complete / High. 열린 완료 차단 0건, 확인된 범위 내 제품 finding 0건.**

2026-10-16 사용자의 U17 독립 감사 요청에 따른 GEX-20 `stage-run`이다. GS-CMER-001 / GOAL-CMER-01 / PHASE-CMER-01의 원래 기능과 승인된 저장·복원 후속을 현재 조합 입력에서 함께 감사했다. 이 실행자는 해당 구현·교정에 참여하지 않았으며, 이번 pass에서 제품 코드·기존 시험·정책·동결 입력을 수정하지 않았다. 서브에이전트·GEX-10 교정·U18 착수는 수행하지 않았다.

## 대상과 현재성

정확한 대상은 [AUDIT_TARGET_RECORD](AUDIT_TARGET_RECORD.json)이다. 기존 `u17_current_001`의 source979/evidence688과 후속 `u17_storage_001`의 source626/evidence1103을 상속하고 후속 source를 우선하는 **source1090/evidence1791**의 조합이다. 제품 source 네 루트의 실제516파일과 목록도 일치한다. 원래 U17 당시 target에 최신 후속을 소급 삽입하지 않았다.

[입구 대조](ENTRY_CURRENTNESS.json)에서 후속 source/증거와 기존 evidence가 모두 일치했다. 기존 source의 변경12개는 후속에 귀속하며 신규 `continuation_contract.rs`도 후속에 포함된다. 나머지 차이는 로드맵의 승인된 후속 등록/완료 기준 추가다. [문서 delta](ROADMAP_DELTA.diff)를 직접 검토하여 기존 A01–A22 의미가 유지됨을 확인하고, 추가된 후속 요구도 아래에서 감사했다. [당시 현재 로드맵](ROADMAP_REVIEWED.ko.md)을 별도 보존했다.

[출구 대조](EXIT_CURRENTNESS.json)는 위 조합 기준 `current / unchanged`다. 감사 중 제품·동결 target 변경0, 누락0이며 HEAD는 `2f725172edbffd27c5078d29000a1a2ba6ddc4d9`다. 무관 dirty 작업과 과거 실패/원본을 보존했다. mutable HANDOFF/EXECUTION_HISTORY/ROADMAP_STATUS와 이 감사의 출력은 target 외부다.

[보관 검산](FREEZE_VERIFICATION.json)은 두 target의 선언된 archive/manifest/diff/delta/복원 receipt 해시, SOURCE member/bytes, 실제 preimage517/12개와 delta420/123개를 확인한다. archive map에 guarded 정방향/역방향 연산을 독립 재생하여 전체 member/bytes 일치를 확인했다. 기존 격리 filesystem 복원 원본도 보존·검증했고, 작업 트리에 patch를 적용하지 않았다.

## 기준·선행과 검증의 재사용

[Goal 원문](../../P2G_50_GOAL_SET.ko.md)의 수용22·차원10·IVA6·실행 node3, [원본 계획](../single_goal/PLAN.md), [후속 계획](../storage_refactor/PLAN.md), 현행 GEX/연속 지시서 및 룰북의 최초 보상·부화·저장 수명을 기준으로 판단했다. 사용자 §1.9 범위에 따라 외부 임의 변조만으로 가능한 사례를 새 필수 시험/finding으로 삼지 않았다.

U16의 현재 [Complete / High 처분](../../../../roadmaps/core_first/execution/u16_final_audit_20261015_001/CURRENT_DISPOSITION.ko.md)과 사용자가 보류한 극단 XP 개선을 인수했다. 이를 새로운 U17 필수 교정으로 되살리지 않았다. 선행 U04/U12/U13/U14/U15/U16의 변경 영향은 결과/노드 완료/자산/codec/Continue·SavePoint 경계다. 현재 Core 전체 및 관련 native, 실제 공개 writer/fresh reader, server persistence를 다시 실행한 구현 근거를 검산해 사용했다. 변경 없는 loader/API/서버 나머지 baseline은 원본 BASELINE_REUSE의 실패 교정과 source 불변성을 대조해 재사용했다. 과거 판정을 현재 코드에 무조건 승계한 것은 아니다.

[VERIFIED_RESULTS](VERIFIED_RESULTS.json)가 raw 검산 결과다. 원본 workspace의38실패는 실제 이름별 교정 통과를 확인했다. 후속 Core 원시 결과는2159pass/1fail/6ignore이며 유일 socket EPERM을 별도 권한 실행1pass로 닫았다. 따라서 기본 Core 유효2160pass/6ignore다. native25·6·45와 persistence29는 서로 별도 묶음이고 중복 합산하지 않는다. 단일 full-green workspace 실행으로 표현하지 않는다.

현재 host의 binary/config SHA, process exit, 요청/응답 trace, 실제 선택/실행 수를 확인했다. 정상3후보/확장64후보는 각각333공개 연산, fresh14분기(각15프로세스), IO17거절/동일재시도다. 이전 V17 writer→새 reader도14분기다. 귀환/Abandon은 writer/reader4프로세스24연산과 실제IO2거절/재시도다. host source는 `os error 21` 도달, Core roots와 active/profile 파일 bytes 불변을 assert한다. persistence29에는 실제16개 crash 지점의 수락 전/후 journal 복구가 포함된다. Abandon을 전투 패배의 대체 증거로 쓰지 않았다.

비용12프로세스의 원시 결과·BINDINGS·전후 중앙값을 검산했다. gameplay requested bytes 정확 회수, 기본 한도 유지, 정상/확장 Continue heap904258→804211bytes와 진단 시간 개선은 해당 기록의 범위에서 유효하다. 내부 누적 예약, 실제 heap, RSS를 구분했다. loader128bytes와 allocator RSS retention, dev 진단이라는 한계는 그대로다.

## 이번 감사에서 새로 실행한 검증

- [집중 검사](focused003.log): runtime-observation의 U17, Continue owner, allocation, runtime profile 복원 **31pass/0fail/0ignore**, exit0. 현재 source에서 빌드하여 실행했다.
- [공개 Continue](restore_probe/SUMMARY.json): **새 프로세스4개 모두 통과**. 정상 SavePoint alias, 정상3단계 선택 완료, 확장64후보 선택 대기, 이전 V17 writer의2단계 부화 저장본을 원시 bytes로 복원했다. inventory/map/roster/알/부화 상태 일치와 requested heap 정확 회수를 확인했다. 선택권 전체 snapshot과 다음 명령의 동등성은 집중 검사의 `assert_fresh_application_next` 및 검산한 host fresh 결과로 확인했다.
- 작성 오류1건을 정상 package loader로 확인했다. 일반 activity에 `elite`가 들어간 kind alias를 연결하는 의심 사례는 [loader 거절](kind-package001.stderr)로 종료됐다. `live_ron.rs`의 allocation role 검증이 runtime의 이름 제약과 일치한다. 따라서 지원 경로의 부화 결함은 재현되지 않았고 finding을 만들지 않았다. calibration/admission/gameplay는 실행하지 않았다.

첫 집중 실행 두 건은 사라진 `/tmp/wp02-cargo-home` 의존성 캐시로 `fs2`를 찾지 못해 빌드 전에 종료했다. sandbox 네트워크 DNS 실패도 보존하고 승인된 fetch 후 성공한 focused003과 분리했다. 이는 제품 실패·0시험 PASS가 아니다. archive 검산기의 첫 실행은 예전 baseline manifest의 dict 형식을 목록으로 취급한 감사 도구 오류였으며, 두 형식을 처리한 freeze002에서 통과했다. 모든 실패 로그를 보존했다.

## 완료 조건별 판단

아래는 원본 [COVERAGE](../single_goal/COVERAGE.json)의 locator를 독립 검토한 결과다. 모든 A가 PASS이며, 후속으로 영향받은 저장/비용 근거는 현재 후속 결과와 새 감사 실행을 우선한다.

| 조건 | 현재 source·소비자 판단 | 직접 검증 근거 |
| --- | --- | --- |
| A01 최초 원장 | `EncounterProgressionState`가 종류/원정별 최초를 보유하고 인벤토리와 분리한다. 결과 transaction에서 commit하며 새 원정에서 초기화한다. | 새 first-kind/public dismantle-repeat-reset 검사, 정상 host |
| A02 자연 엘리트 | `map_encounters::select_endless_elite`는 완료 원장을 후보 제한으로 읽지 않으며 기존 배정/예약을 보존한다. | 새 natural-elite 검사, 정상 반복 trace |
| A03 최초 알·예외 | 일반보스 최초만 Stage2 알을 만들고 loader가 엘리트/무보상 story의 revival을 거절한다. | 새 class/story·public boss 검사 |
| A04 부화 원자성 | 자격/recipe/source 지불은 application candidate에서 처리하고 실패 시 원본과 UUID를 보존한다. | 새 explicit-payment·native exact/short/retry, host hatch IO |
| A05 노드 보호 | 실제 예약/omen/normal class/미해결 자격을 확인하며 Gate 우회로를 요구하지 않는다. | 새 reserved-normal·only-route-to-gate 검사 |
| A06 촉매 출처 | U15 `pay_materials`와 run/profile debit, `revival_snapshot`의 소지/창고 quote를 연결한다. | 새 혼합출처/부족 검사, 상점 구매→부화 host |
| A07 강화·수량 | node stage policy와 실제 difficulty를 결속하고 무억제 수량 배율만 기존 grant 경로에 적용한다. | 새 stage roundtrip·unrestrained zero-choice, host query |
| A08 단계·패배 | Stage2 승리만 다음 알, Stage3/패배는 알0. 패배 node는 소비하며 지급/환급하지 않는다. | 새 egg sequence 및 실제 battle tick defeat |
| A09 후보 고정 | `equipment_reward_options`와 retained source/stage가 결과/reader의 같은 후보를 만든다. 후보0은 선택만 생략한다. | 새 alternate/zero-choice·fresh helper, 확장64후보 fresh |
| A10 한 번 지급 | 선택/claim은 저장된 reward owner를 소비하고 중복 claim을 거절한다. | 새 public boss/fresh next-command, 실제IO 동일재시도 |
| A11 부활 보상 제한 | revival result는 원래 봉인/omen 보상 경로와 구분하며 고정 stage 보상만 생산한다. | 새 public boss/zero-choice, 정상 host의 파츠/장비/알 oracle |
| A12 후퇴·실패 | 후퇴는 독립 entry snapshot+횟수 overlay, 두 번 후 마지막 도전 유지. 패배와 후퇴를 구분한다. | 새 retreat/actual defeat, host retreat1/2/3rd rejection |
| A13 Continue | unfinished root는 입장 전 상태, 입장 전 확정 부화는 retained root에 남는다. | 새 before-entry/unfinished hatch, host fresh/reconnect |
| A14 SavePoint | 공통 copy/install 뒤 해당 원정 debit 차이만 reconcile하고 host load 횟수를 유지한다. | 새 before/after hatch·alias·runtime profile, 새 SavePoint process |
| A15 재방문/하강 | 같은 층 node owner 유지, `leave_floor`는 node만 제거하고 알 유지. 입장 receipt 별도 보존. | 새 reserved/revisit·retreat, 실제 same-floor SavePoint branch |
| A16 durable IO | 준비 candidate와 paired journal 수락 경계가 분리되고 거절/동일재시도는 원자적이다. | normal/expanded 각17 IO, terminal2, persistence29/16 crash 지점 |
| A17 legacy 제거 | world dispatch에서 typed retired rejection, 활성 제안/런 owner와 새 codec에서 구 진행 제외. | 새 retired command 검사, source/state/query 검토 |
| A18 저장 호환 | V17 및 content identity를 검증하고 이전 개발 저장을 보존·거절한다. 새 cache는 검증된 bytes다. | 새 실제V16 거절, 새 old-writer process, old-writer fresh14 |
| A19 선행 회귀 | 변경 공유 경계의 기본 Core/native/host/persistence가 현재 근거다. 튜토리얼·profile 격리도 유지한다. | 현재 Core2160/ignore6, 관련45·persistence29, 검산한 baseline 실패38교정 |
| A20 지원 제작 | RON→validation→공개 수동 전투/부화/선택을 연결하고 잘못된 recipe/class/reward를 거절한다. | 새 authoring/public 검사, 정상/확장 host, 추가 작성 오류 거절 |
| A21 공개 소비 | CoreApplication→host writer→fresh process→후속 명령의 실제 경계를 확인했다. 완료 preview만 회수한다. | 새 completed-preview/fresh helper, 현재 host/fresh/IO, 새4process |
| A22 비용/한도 | typed decode·Value·normalization·buffer·Arc·clone을 구별하고 지원 상태를 기본 한도에서 처리한다. | 새 allocation/one-short/retry, 현재3500자원 회귀, 비용12원본 검산·새4회수 |

10개 차원(success, invalid_input, failure_rollback, retry_idempotency, ordering_concurrency, boundary_empty_max, disconnect_restart_persistence, live_data, producer_consumer, legacy_fallback_absence)은 위 조건과 raw로 모두 충족하며 N/A는 없다. IVA-CMER-G27/G28/G29/RESTORE/LEGACY/AUTHOR 및 AG-CMER-RUN/WP-CMER-PUBLIC/WP-CMER-VERIFY도 해당 조건의 직접 근거로 충족한다. 상세 ID 연결은 [CONDITION_REVIEW](CONDITION_REVIEW.json)에 있다.

## 저장·복원 후속과 구조 검토

후속 delta의13개 source/test 파일을 직접 검토했다. `RunCheckpointPayload::{preflight_copy,copy_owned,from_shared}`는 independent owner admission을 통일하며 SavePoint alias와 실제 gameplay root를 독립 유지한다. cached SavePoint가 존재하면 shared copy가 필요하므로 단독 Arc 이동만을 최종 절감이라고 판단하지 않았다.

Continue는 typed decode 전에 shape admission을 수행하고 검증된 immutable payload/encoding을 보관한다. 공개 application의 직원/relic/resource reconcile, loadout/profile 검증은 남아 있다. 현재 정상 writer가 만든 동일 content/profile 상태에서 이 검증은 새 진행을 만들지 않으므로 재capture/재encode 생략이 정당하다. fresh 전체 snapshot/다음 명령, old writer와 확장 복원 검증이 이 경계를 직접 뒷받침한다. 매 저장마다 전체 restore를 호출하거나 정상화로 오류를 숨기지 않는다.

canonical writer는 지원 dependency graph의 기본 BTreeMap 순서를 사용하고 SHA-256은 streaming한다. Shape는 객체별 node 상한과 depth128, 닫힌 RunResourceEntry의 실제 typed 구조를 사용한다. 문자열/array/scratch·encoded Vec·Arc는 별도로 예약한다. borrowed sorted resource index는 authored catalog 순서를 강제하지 않고 중복/owner/lifetime/cap 검증을 보존한다. 기존3500자원 입력과 기본16MiB/작업한도를 줄이거나 올리지 않았다.

완료 node의 preview/attempt/일반 Event session은 한 경계에서 회수하지만 relic Event receipt, 미확정 node, 확정 result와 미수령 권리는 별도 owner로 남는다. Continue/SavePoint/거래 rollback/전투 후퇴의 정책 차이는 후속 PLAN 표와 실제 capture/restore/lifecycle을 대조했다. 귀환·Abandon 및 실제 battle defeat는 각각 다른 경로로 확인했다.

테스트 전용 제품 분기, 신규 fallback/정책 변경, 새 중복 authority, 불필요한 타입 계층 확장이나 근거 없는 예산 완화는 확인되지 않았다. canonical 문서의 V17/명령/query/부화 계약과 구현도 일치한다.

## 한계와 다음 경계

완료를 막는 미검증/접근 불가 입력은 남지 않았다. 재사용한 과거 private issuer의 `/tmp` 디렉터리는 현재 없지만 key는 원래 보존 대상 밖이며, 당시 host binary/config/승인 결과/trace와 저장본은 남아 검산했다. 이를 현재 실행 가능한 issuer라고 주장하지 않는다. 새 공개 Continue는 issuer가 필요 없는 Core 포트에서 수행했다.

정상3/확장64후보의 유한 입력, dev 비용 측정, 기존6 Core/22 server ignore, 공식 밸런스·Unity·개별 최종보스·U18 이후 미인수 범위는 유지한다. 이번 결론은 현재 U17 기능과 저장 후속의 완료이며 전체 로드맵/최종 SLO 완료가 아니다. 이전 target의 `audit_performed:false` 등은 당시의 고정 사실로 보존한다.

재현은 `AUDIT_PLAN.ko.md`, 새 검사 command가 기록된 `RESULT.json`, `restore_probe/*/RECEIPT.json`, 각 검산 스크립트와 [기존 재현 안내](../storage_refactor/REPRODUCE.md)를 따른다. write-once 출력과 기존 실패를 덮어쓰지 말고 새 sibling 감사 디렉터리/출력명을 사용한다. **이 stage-run은 감사 완료에서 종료하며 다음 구현은 새 사용자 요청 대상이다.**
