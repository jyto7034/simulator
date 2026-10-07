# F02 experiments

- Entry verified predecessor target and closed audit before mutations.

- f02_core_fault_tests_001 raw101: 15 PASS/2 FAIL. 실제 Goodbye/clock 및 observer는 PASS. unsupported authored fixture의 미사용 Goodbye RemainingTime capability 불일치를 교정했다.
- f02_swap_diagnostic_001 raw101: 실제 public swap은 leaf 조회보다 앞선 capability projection에서 보유 참조 손상으로 panic. backtrace 원본 보존. leaf-only 수정 대신 기존 application 문맥 검사에서 charge된 참조 invariant를 검사한다. 일반 missing caller와 구별한다.
- f02_core_fault_tests_002: 위 교정의 집중 회귀 실행 중.

- f02_core_fault_tests_002 raw0: 17/17 PASS. Both admitted focus branches, missing present Goodbye, either retained deployment reference, five ordinary refusal cases, unchanged dynamic expression first cause, observer end/no repeat pass. Full matrix now running on source 0e8194aca0984ec11c5a5a3068cffe32f16ab187912249a3083e0ae0c0271979.

F02 최종 matrix 001의 12개 명령 모두 raw0. Core1717/1717/1726, host81/88 및 별도 heap1 PASS. fmt/check/event_log/clippy/release PASS. 각 receipt의 source_before/after와 log hash를 검증했다. MATRIX_RESULT.json 참조.

PC 실행 중 다른 Python/worker 계산의 동시 실행을 process name 수준으로 관찰했다(evidence/CONCURRENT_ENVIRONMENT.json). 해당 작업은 변경하지 않았다. 기존 계약상 latency는 합격 gate가 아니며 결과/할당량/retained/원래 hard guard를 그대로 검증한다. 새로운 성능 한도나 제외를 도입하지 않는다.

호환성 16개 명령의 source_before/after 및 log 해시 검증 완료. PC576 exact non-timing U02 parity PASS. R05 raw101은 기존 승인 concurrent peak 단독 경고(+232950 bytes), 다른 hard guard 실패 없음. 모든 실패 원본과 별도 분류를 보존했다. COMPATIBILITY_RESULT.json 참조.

최종 결과: GEX_10_RESULT.json. 모든 final_* matrix 및 compat_* receipt의 raw log/source identity를 검증했다.
final_fault_f02_001_*은 최종 release source에서 native failure와 다섯 관측 모드 exact parity, 실제 stock websocket
예정 tick 오류→reconnect/replacement/save 보호→Quit/new owner/valid Continue를 검증했다.
R05 raw exit 101 / warning_concurrent_peak_only는 원본과 별도 분류를 유지한다.
새 독립 Complete/High 판정은 이 구현 결과에 포함하지 않는다.
