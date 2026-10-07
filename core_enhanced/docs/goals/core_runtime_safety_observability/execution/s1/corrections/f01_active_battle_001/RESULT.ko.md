# S1 F01 교정·자체 검증 결과

S1 최초 독립 감사는 Not Complete / High confidence로 닫혔다. F01은 전투 직전 발급한 non-battle persistence token이 전투 진입 뒤에도 복원되어 진행 중 전투를 제거하던 결함이다. 원 감사와 실패 재현은 ../../../../post_audit/s1_20261004_001에 그대로 남겼다.

public `CoreApplication::restore_persistence_rollback`은 owner/next-attempt 검증 뒤 현재 active battle이면 `active_battle_persistence_rollback` InvalidAction을 반환한다. 이 검사는 revision 증가·private restore 이전에 수행된다. 기존 non-battle 복원, monotonic profile/peak, stale/foreign/duplicate/overflow 검사는 그대로다. private Core undo/test API를 새로 공개하지 않았다. 동결 정책 TD03과 canonical 계약을 구현하는 교정이므로 P2G 의미 변경은 없다.

제품 delta는 application.rs의 guard7줄과 server handlers.rs의 기존 first-entry regression 확장뿐이다. COVERAGE.json의 기계 검사는 그 외 application bytes와 해당 test 밖 host bytes의 동일성을 확인한다. 교정 전 baseline은 감사된 s1_001 target의 exact source/document bytes다.

기존 회귀 테스트는 pre-entry token 캡처 → 실제 successful entry → restore 거부를 검사하고, live battle 존재 및 run/reconnect projection 불변을 확인한다. 새로운 공개 production-profile probe는 최초 감사의 external R06 seed77 경로를 재사용하여 같은 거부와 상태 보존을 검증한다. entry 전후 exported durable checkpoint도 같다. 실제 host는 이 checkpoint의 변화가 있을 때만 store.publish하므로 entry에서 persistence failure rollback을 호출하지 않는다. 기존 non-battle storage failure와 독립 profile publication은 현재 host 전체 suite에서 다시 검사했다.

최종 Core 기본/test-fixtures 각각1700 tests와 host76 tests가 통과했다. check/event_log_checks/Clippy/fmt/release 및 공개 F01 probe도 current exact source로 통과했다. Clippy 경고는 보존한다. 최초 fmt check1은 새 assertion의 줄바꿈을 요청했고, 그 한 곳을 정리한 fmt check2와 최종 suite를 사용했다. 실패 원본은 삭제하지 않았다. 공개 probe1은 감사 당시의 cached rlib 경로를 재사용하여 교정 전 라이브러리를 연결했다. hash가 이전 라이브러리와 같음을 확인했고, probe2는 동일 release Cargo 명령의 compiler-artifact JSON에서 현재 game_core/serde_json/uuid 경로를 가져와 다시 compile·실행했다. 원래 실패 로그와 양쪽 library hashes는 보존하며 probe1을 current-source 성공으로 세지 않는다.

전체7acceptance, V01–08/V12,10dimensions, inherited88conditions 및 U01 F01 selectors를 successor COVERAGE/PREDECESSOR_IMPACT에 연결했다. native/authoring/expression/history 알고리즘과 cost 입력, default numbers, CLI, public API traits/signatures, host production bytes는 이번 교정에서 바뀌지 않았다. 해당 기존 public/PC/R05 증거는 원래 source/time/result로 보존하고, guard-only delta에 근거해 제한적으로 재사용한다. 최종 suites/checks/release는 새 receipt가 대체한다. R05 raw101/concurrent-peak-only warning과 이전의80 PC requested-detail differences는 그대로 남으며 새로 실행한 비용 검사로 주장하지 않는다.

다음은 `target/s1_f01_001/MANIFEST.json`에 대한 fresh independent GEX-20다. 감사 output은 ../../../../post_audit/s1_20261004_002에 분리한다. 기존 reviewer는 구현에 참여하지 않았으므로 fresh pass에 재사용할 수 있다. 관련 predecessor currentness/impact 복원과 S1 Complete/High 판정 전까지 S2를 시작하지 않는다. 이 문서는 교정·자체 검증 결과이며 독립 완료 verdict가 아니다.
