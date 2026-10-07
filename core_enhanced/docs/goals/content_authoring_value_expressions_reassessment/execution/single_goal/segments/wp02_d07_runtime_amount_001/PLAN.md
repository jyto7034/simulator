# R06 WP02 D07-B 실제 평가 기록·조회 PLAN

- 시작점은 D07-A manifest SHA-256 `ae6609c1e75b711a27cacf8467ac0c42eaa99635a0fc19c5ae486ecd949bb1ac`, 확장 338개 digest `5279b8333c6be8f90992ac94a47f09beb083bf82bfcdf02ebbe3e0d179c8f530`다. 구현 전 currentness와 격리 양방향 복원을 다시 확인하고 실제 바이트를 before로 보존한다.
- 신뢰된 `TrialLimits` opt-in에서 실제 executor가 소비한 compiled amount 평가의 최근 1건만 active observer에 보관하고, 종료 시 last-terminal capsule로 이동한다. 관측 off의 기존 gameplay와 저장 형식은 유지한다.
- runtime identity/frame 선택 뒤 공통 numerical helper를 사용한다. runtime 결과와 recorded preview/explain 재평가는 같은 plan/frame/evaluator/narrowing을 사용하며 현재 전투 상태로 과거 입력을 재구성하지 않는다.
- legacy Damage/Heal도 opt-in authoring 전투에서는 catalog plan의 공통 evaluator 결과를 실제 command가 소비한다. catalog 없는 programmatic effect와 일반 gameplay literal 경로는 유지한다.
- 새 조회는 Latest(active/last-terminal)와 Exact(battle key/evaluation ordinal)만 허용한다. evaluated와 unavailable을 명시적으로 나누고 amount/trace 0 또는 빈 성공값을 발명하지 않는다.
- record commit은 effect command·projection·prefix settlement 성공 뒤에만 수행하고 candidate transaction rollback 안에 둔다. ordinal과 최근 record도 gameplay와 함께 복원한다.
- frame/trace/source/plan record extent, 교체 peak, clone/native retained와 query work/metadata/result/cancellation을 계량한다. 전체 이력·map·ring buffer를 만들지 않는다.
- 기존 정적 explain, 공간 preview, optional NDJSON trace, runtime facts, target usefulness eligibility, authored publication 차단을 유지한다. 미래 수치는 Indeterminate이며 과거 record를 다음 시전 판단에 재사용하지 않는다.
- B01–B20, 실제 CLI `simulate --amount-observation`, PC36/R05를 최종 source에서 검증하고 `target/d07_runtime_amount_001`에 currentness와 신규 파일 부재↔존재를 포함한 양방향 복원을 동결한다.
- D07-C/WP03/04, 20분 시험, production authored expression publication 허용, 전체 완료 판정은 수행하지 않는다.

## 실행 결과

최종 확장 source는 340개, digest `ad8021359785e365c3f9560268bcef2a95205495f9e33a22314c3f97743480c4`이며 제품 336개 digest는 `6b5e0291b1b6f936c277e2138bc2c2adbbeaaa2b3df5c1b4d5bf31b5acff6807`이다. B01–B20 대응, Snapshot/Live V17 원자료, PC36, R05 raw와 모든 실패를 기록했다. 최종 R05는 concurrent peak 단일 warning이며 다른 hard guard는 통과했다. 동결 후 독립 검토 전 상태에서 중단한다.
