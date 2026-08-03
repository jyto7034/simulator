# Run Policy Correction Master Final Completion Review Experiments

Status: complete

Implementation complete: true

Created: 2026-07-30

## 2026-07-30 — P04 활성화

- P00~P03 child completion review가 모두 `Complete / High`임을 gate로 확인했다.
- P03 current broad evidence는 Rust workspace all-targets 전부 통과, Unity EditMode 260/260,
  PlayMode 62 passed/4 explicit live-server skips다.
- P04는 child 완료 표시를 증거로 재사용하지 않고 finding/runtime/data/save/server/Unity/canonical과
  legacy search를 독립 대조한다.
- D-01 보정은 P00 current header와 정정 record만 대상으로 하며 historical experiment 본문은 보존한다.
- 다음 작업: P04 문서 완독 후 finding evidence matrix, legacy/status/diff audit와 final review 수행.

## 2026-07-30 — finding별 current evidence 재대조

- F-01: common transaction과 active-run rollback은 rollback 전/도중의 compendium을 monotonic union하며,
  `AttachSession`은 durable profile sync 뒤 snapshot을 만든다.
- P-01: certified formula는 `Employee::is_combat_eligible()`만 합산하고 live RunPolicy model은 2다.
  Canonical은 Critical/Breakdown/Dead 제외와 zero eligible power `VeryHigh`를 일치하게 규정한다.
- P-02: Core payload V2 checkpoint가 model/anchor를 소유하고 host envelope V3 monotonic metadata는
  loads/run peak만 소유한다. Old/missing version과 removed monotonic anchor는 strict reject한다.
- F-02/P-03: typed stage/ledger와 단일 resolver, exact Stage2/3 stats, unchanged wave, one-pass reward 및
  qualitative Unity tier를 source/DTO/reader/presenter/canonical에서 재대조했다.
- D-01: original P00 experiments/notes header를 complete/true로 보정하고 초기화 당시 본문은 historical
  statement라는 current-status correction만 추가했다. 과거 실험 결과는 재작성하지 않았다.

## 2026-07-30 — focused/legacy/diff audit

- Core compendium filter: 5 passed, 0 failed.
- Server compendium filter: 10 passed, 0 failed.
- Core certified filters: `certified` 2/2, `certification_` 2/2.
- Core checkpoint filter: 24 passed, 0 failed.
- Server active-run-store filter: 9 passed, 0 failed.
- P03 current evidence: Core monolith 17/17, live RON 23/23, Unity focused EditMode 3/3.
- Old Core V1 type/version과 typed stage fallback search는 0건이다.
- Host monotonic anchor, old envelope V2, public exact reward percent hit는 negative rejection/absence tests 또는
  internal Core tuning 변수뿐이며 production compatibility reader/writer는 없다.
- `cargo fmt --all -- --check`: passed.
- Workspace `git diff --check`: passed.
- Unity changed reader/test scoped `git diff --check`: passed. Unity repo 전체 검사는 범위 밖 기존 CRLF
  `.gitignore`/IDE files를 trailing whitespace로 보고해 해당 사용자 파일은 보존했다.

## 2026-07-30 — broad 및 live opt-in 결과

- `cargo check -p game_core -p game_server`: passed.
- `cargo test --workspace --all-targets -q`: nonzero suites 798/3/4/23/13/4/54, 합계 899 passed,
  0 failed/ignored.
- Unity EditMode: 260 passed, 0 failed, 0 skipped.
- Unity PlayMode broad: 62 passed, 0 failed, 4 explicit live-server skips.
- Local game_server를 `ENABLE_ADMIN_COMMANDS=true`, token `dev`, port 8081로 기동해 explicit 4건을 지정
  실행했다. Run-start, combat controls, admin event 3건은 통과했고
  `LiveAdmin_MaintenanceFixtureExposesOperationOptions`는 executable operation 0개로 실패했다.
- Maintenance 실패는 단독 재실행에서도 동일했다. 이는 correction finding의 rollback/power/anchor/
  monolith boundary와 독립적인 기존 opt-in fixture/maintenance operability 문제이므로 P04에서 production
  범위를 확장하지 않고 completion review의 out-of-scope risk로 남겼다. Server는 정상 종료했다.
- `COMPLETION_REVIEW.md`: correction scope `Complete / High`.
