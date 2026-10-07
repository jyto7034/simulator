# R06·R07 최종 통합 구현 인계

R06·R07 구현·자체 검증 종료 / 최종 통합 target 동결 / 독립 감사 요청 대기.

독립 감사 GEX-20/30은 수행하지 않았다. 최종 Complete/High 또는 독립 PASS를 선언하지 않는다. 이전 C07의 독립 미검토도 최종 감사 범위다. 기존 dirty/untracked 작업을 reset/stash/clean하지 않았고 전체 변경을 이번 작업으로 귀속하지 않는다.

## Target과 검증

최종 source digest `714195327ec0df7facc60d1aafe721e6e26c1296494b39cff593232c45365e6f`. [최종 manifest](target/r06_r07_integrated_001/MANIFEST.json), [수용표](COVERAGE.json), [R06 31 acceptance/21 IVA 영향](R06_IMPACT_COVERAGE.json), [명령별 currentness](evidence/VALIDATION_CURRENTNESS.json). Actual source/data/tests/config/tools/raw V7 membership, before/after archives, forward/reverse patch 및 격리 복원 근거는 target에 있다. 동결 후 manifest SHA는 FINAL_TARGET.json에 기록한다.

R06 경계 manifest SHA256 `0b6938d77cc6cded598b2b01e51d41bf7e651dd66e8acf2f695a1cf3b5fd44f1`: [별도 경계](../../../content_authoring_value_expressions_reassessment/execution/single_goal/continuations/r06_completion_001/target/r06_boundary_001/MANIFEST.json). 이 상대 locator의 실제 root 경로는 `core_enhanced/docs/goals/content_authoring_value_expressions_reassessment/execution/single_goal/continuations/r06_completion_001/target/r06_boundary_001/MANIFEST.json`이다. 경계 원자료는 덮어쓰지 않았다.

기본 전체 1581 / test-fixtures 전체 1581 passed, failed0, ignored0, 기본 병렬. check/event_log_checks/release bins/clippy/fmt 성공. Clippy의 기존 never_loop/eq_op allow는 R06 동일 명령을 유지했다. Release CLI에서 R06 정확 i128 수치, signed MIN/범위 밖 거부,4종 resolved-owner 집계 및 R07 Snapshot/Live 정상 지급·Maintenance·개화/선택·전투의 trace off/on4문서가 일치했다. 공개 API 새 process 저장 복원은 inventory와 effective-input fingerprint를 비교했다.

PC36:576rows,36conditions,31success/5typed refusals, gameplay/completion/error code 차이0. R05 concurrent peak `9214469` bytes(cap9030188), 초과 `184281`만 합의된 경고다. 나머지 allocation operations/retained/event peak/repeatability/parity hard guard 실패0. latency는 non-comparable 환경의 관찰값으로 보존한다. 실패 원자료와 교정은 EXPERIMENTS.md에 연결했고 덮어쓰지 않았다.

## 구현과 R06→R07 변경 분류

R06: C07-OBS01 delivery 만료 settlement, C03-OBS01 i32::MIN RON decode, 공개 지원 설명 교정, typed expression Snapshot/Live·유한 집계·stat/fingerprint·읽기 전용 amount/trace·정확 숫자·한도/취소/rollback의 구현을 보존하고 최종 소스에서 다시 실행했다.

R07: 단일 OwnedFragment map과 checked UUID grant; 양방향 copy binding; exact equip/unequip/transfer; recipe 개화 해금과 별도 형태 선택; 확인 분쇄의 해제/제거/산출/Run owner 정리; 사망 회수; 공개 DTO·form eligibility와 command 일치. 실제 profile→actor→cast/delivery→amount/trace에 FragmentSourceSnapshot을 연결하고 R05 FragmentCopy/SourceFragment를 기존6 lifetime에 통합했다. Runtime retained/CloneShape accounting, bounded map/set decode, exact/short/overflow 및 중간 취소 재시도도 검증했다.

저장 전환: live schema4 / canonical strict V8. V7 자동 로드는 거부하고 explicit converter가 원본 checksum·caller 원래 identity를 검사한 뒤 새 사본들을 결정적으로 만든다. old awakened는 해당 모든 사본으로 승계, rank/progress/0보유 history와 dust는 보존 정보다. 원본 raw V7 SHA256 `47af09db635ab1691dc61d7094d6b2414237562c27cd6e1d1b6ce9a4e49024c9` 유지. V8 전체 graph/compatibility/content mismatch를 commit 전에 거부한다.

## 위임 정책과 경계

사용자 과거 확정 정책으로 위장하지 않은 결정은 [P2G 검토](../../P2G_POLICY_AND_REVIEW.json), [현재 기술 판단](EXPERIMENT_NOTES.md)에 기록했다. 주요 결정: copy cap1024/forms16/recipe64; 기존 UUID stream을 보존하는 FRAGCOPY 전용 checked stream; 개화는 unlock-only/선택 별도/occupied 이전 거부; definition+form compatibility 교집합; recipe 없는 정의는 unavailable; 기존 dust reward/잔액 보존과 새 개화 소비 분리; shared rank gameplay 제거와 explicit V7 history; FragmentCopy owner 및 기존 lifetime 재사용; 원본을 쓰지 않는 deterministic V8 변환.

공식 active 정의의 미작성 recipe와 BasicAttackModifier placeholder는 명시 unavailable다. 검증용 외부 공개 콘텐츠에 통화+환상체 부위 recipe를 작성해 전체 경로를 증명했으며 공식 밸런스를 임의로 발명하지 않았다. Host/GUI의 V8 저장 파일 게시 UX는 이번 Core 범위가 아니다. 구현 hard blocker 없음. 남은 것은 별도 요청의 독립 감사이며 R08 이후/20분 soak/스킬 자동 취소는 수행하지 않았다.
