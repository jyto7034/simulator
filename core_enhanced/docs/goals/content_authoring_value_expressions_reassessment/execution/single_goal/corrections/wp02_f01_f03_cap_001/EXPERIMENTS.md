# R06 WP02 교정 실행 기록 — 첫 단계검토 경계

상태: **F01 현재 frame/evaluator host admission 교정·자체 검증 종료, 부분 target 고정 준비**. 이 기록은 독립 재검토 판정이 아니다. 적용 지침은 `docs/goal_execution_pipeline(1).md` revision `2026-09-13-gex-r7`; 이번 요청의 단계검토 opt-in을 유지한다.

## 진입과 actual bytes

HEAD `f318414fb0156a67e04c5479000c2a2683fc0c09`, branch `cleanup/simulator-dirty-worktree-20260803`. WP02 감사의 exact target manifest SHA-256은 `d3dbf3ba7c3c3be5a584db3092e7059c116d92b1ae812740f1fb68ccb05c2fc3`이며 판정은 `CORRECTION_REQUIRED`다. mutation 전 `baseline/entry_001/MANIFEST.json` SHA-256 `d0c56f7c2e453db69dc986d31b0fe83389b261cc6a41d6070245c13445994f94`에 실제 322개 파일, 전체/범위 Git status와 복원 archive를 보존했다. verifier는 archive 322개와 entry 당시 WP02 target 312/314 일치, 기존 Unix 테스트 두 파일 delta를 확인했다. 이것은 새 target의 검토 PASS가 아니다.

수정 전 현재 adapter selector를 직접 실행해 7/7 PASS를 관찰했다. 그 첫 실행의 raw stdout은 별도 파일로 저장하지 못했으므로 후속 검토의 직접 archived receipt에는 합산하지 않는다. 수정 후 같은 selector의 10/10 실행 로그와 source identity를 별도로 보존했다.

## 변경과 선택한 중단점

`game/value_expr/mod.rs`는 독립 kernel meter를 유지하며 선택적 host admission 함수를 담는다. `eval.rs`는 work·retained를 실제 단계 직전에 local limit과 host operation 양쪽에 적용하고 각 work 경계에서 취소를 다시 확인한다. `battle/core/value_expr_runtime.rs`는 frame 참조/plan 순회의 work와 Live frame 할당을 선청구하고 evaluator에 host 함수를 결속했다. 종료 후 evaluator work 일괄 청구는 제거해 중복을 막았다. Snapshot frame의 저장 공간은 capture extent에서 한 번 계산한다. 기존 한도 상수·수식 값·rounding·fingerprint·publication gate는 바꾸지 않았다.

F01의 현재 private frame/evaluator admission 경계는 빌드·관련 시험과 원본 감사 재현을 통과했다. 다음 큰 변경인 F02 action provenance/F03 lifetime/CAP storage growth 전에 중단한다. F01의 향후 production consumer가 반환 trace를 장기간 보존한다면 그 owner extent를 consumer 연결 시 추가 확인해야 한다. 현재 adapter에는 공개 expression consumer가 연결되지 않았다.

## 실행 증거

원시 명령·cwd·exit·출력 hash·도구 identity는 `evidence/EXECUTION_RECORDS.json`에 묶었다. 원본 PC probe와 집계기는 복사만 했고 입력/반복 수를 변경하지 않았다. 격리 사본의 계측용 include와 read-only native usage hook은 `evidence/pc_after_f01/`에 별도 저장했다.

| 범위 | 직접 결과 |
| --- | --- |
| F01 현재 adapter | 10/10 PASS. 새 native work·retained 및 authoring limit/cancel 테스트 3건 포함 |
| 원본 감사 F01 probe | work=1에서 전체 평가 요청 allocation 53,936 → 거부까지 342 bytes; 잔여 retained 1 byte도 frame allocation 전에 거부. 각각 1/1 PASS |
| 평가 도중 취소·다음 operation | 격리 사본의 추가 probe 1/1 PASS |
| 공유 evaluator / runtime facts / resources | 각각 36/36, 11/11, 37/37 PASS; 같은 최종 root debug binary로 직접 실행 |
| 전체 테스트 | 첫 sandbox 실행은 1,409 PASS, Unix socket 1건 `PermissionDenied`로 exit 101. 동일 명령을 권한 제약 밖에서 재실행해 lib 1,410건 + integration 52건 PASS, exit 0 |
| 빌드·정적 검사 | `cargo check -p game_core --all-targets --offline --locked` PASS; focused rustfmt PASS; Clippy는 기존 allow 2개만 사용해 exit 0, 경고 원문 보존 |
| PC 원본 36조건 | 이전 30 완료/6 거부 → F01 후 25 완료/11 거부. 새 5개 거부는 128 Read 반복의 cumulative retained admission. 단순 Read 6,000회 및 100/300명 literal step은 완료 |

Clippy는 기존 소스 전반의 warning과 새 helper 두 개에도 적용되는 `result_large_err` warning을 보고했다. allow를 늘리지 않았다. R05의 9,030,188 bytes concurrent requested-allocation guard는 이번 target에서 직접 재실행하지 않았으며 역사적 증거를 현재 PASS로 합산하지 않는다. 이 구간은 R05 storage 구조를 수정하지 않았다.

## 미완료와 재검토 입구

WP02-F02 actor/equipment copy/action 결속, WP02-F03 inactive Live resource 거부, CAP-01 ActorSummaries 및 CAP-02 EventQueue 성장·extent는 아직 수정하지 않았다. PC 측정에서 기존 `actor_row_reserve` 300명 피해 거부와 `queued_event_slots` 500명 생성 거부가 같은 fault/requested 값으로 남았다. F01 후 128 Read 126회 뒤 거부는 새 admission이 작동한 결과이며, 이전 완료 시간을 새 거부까지 시간과 비교하지 않는다.

선행 WP02 감사 finding의 F01 부분만 이 target에서 검토를 요청한다. 재검토 PASS, WP02 완료, 다음 순차 effects 또는 WP03 진입은 선언하지 않는다. 재검토 결과와 별도의 다음 구현 요청을 기다린다.
