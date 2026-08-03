# Employee Psychology State Contract Experiments

실제 조사, 구현과 검증을 시간순으로 기록한다.

## 2026-07-18 — Child Goal 초기화

- Phase 1~3 completion review와 Master Phase 4 gate를 확인했다.
- production 조사 전에 필수 문서 3개를 생성했다.
- 이 단계에서 production 파일은 변경하지 않았다.

다음 단계는 current `employee_trust`, Employee/Profile, battle/support/checkpoint DTO와 Unity consumer의 read-only 조사다.

## 2026-07-18 — Production 조사와 Psychology complete cutover

- 기존 `EmployeeTrustState`는 append-only memory, reaction summary와 비활성 high-risk/cue/combat/trauma modifier policy를 한 모듈에 노출했다.
- 실제 production mutation은 Rest support의 `RestedAtSupportRest`뿐이었고, 전투 trauma modifier는 기본 policy에서 비활성이었다.
- `employee_trust.rs`와 world-level policy를 제거하고 `EmployeePsychology`로 교체했다.
- trust score -100~100, exact derived band, typed `employee_self` subject, `node_completion` source identity, semantic memory merge, source idempotence, optional expiry와 bounded recent event receipt를 구현했다.
- Rest completion은 node UUID source로 살아있는 직원에게 +8을 정확히 한 번 적용한다. 완료 노드 lifecycle은 기존 expiring memory를 감소시킨 뒤 새 Rest event를 적용한다.
- 전투불능 trauma는 승인된 condition policy 값을 직접 적용하며 dormant trust modifier를 거치지 않는다.
- checkpoint와 battle-entry retreat는 roster clone으로 Psychology를 exact restore하고 사망 asset recovery는 roster record/Psychology를 보존한다.
- Core DTO를 `psychology`로 complete cutover하고 Unity `RosterModel`에 score/band/collection/kind/subject/source strict reader와 legacy `trust` rejection을 추가했다.
- Rust fixture generator, Unity fixture/mock/test payload와 canonical 문서를 같은 shape로 갱신했다.

## 2026-07-18 — Focused 및 broad 검증

- Core focused: Psychology 5 tests, Rest exact JSON, checkpoint restore, death record preservation와 explicit wire label tests 통과.
- Core broad `cargo test --all-targets`: lib 732 + integration 44 = 776 pass.
- game_server `cargo test`: 30 pass.
- Rust employee profile fixture generator exact comparison: pass.
- Unity focused jobs `49bd22e43d4f44df861a3cfd7b98aedd`, `6a785e1649164cbf8ad30e2d9e79f4d8`, `479d0e4b38164d9d9dff4819f625c588`: all selected tests pass.
- Unity final EditMode job `478f3b049bdf4d91b7072b9c0e0b647b`: 241/241 pass.
- Unity ordinary PlayMode job `d649a2be39de45ab93f1206faf33a962`: 57/57 pass.
- relevant Core/Server/Unity `git diff --check`: pass.
- `cargo clippy --all-targets -- -D warnings`: 기존 전투/runtime lint debt 때문에 실패했다. 새 Psychology의 유일한 reported warning인 derivable Default는 수정했으며 전체 tests를 다시 통과시켰다.
- production search에서 `employee_trust`, `EmployeeTrustPolicy`, reaction/cue/modifier public surface와 구형 Core `trust` projection은 남지 않았다. Unity의 `trust` 문자열은 strict rejection negative test와 reader guard뿐이다.
