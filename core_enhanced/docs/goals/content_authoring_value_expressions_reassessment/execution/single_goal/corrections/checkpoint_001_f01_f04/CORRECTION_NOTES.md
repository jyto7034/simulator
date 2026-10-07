# R06 checkpoint 001 F01–F04 교정 기록

State: `교정본 재검토 준비 완료 / 재검토 요청 대기`

## 고정 경계

- 수정 전 실제 311개 source bytes는 `baseline/pre_fix_001/SOURCE_BEFORE_FIX_001.tar.gz`에 보존했다.
- parent target은 `R06_STAGE_REVIEW_TARGET_001`, manifest SHA-256은 `e62085bbcbdb8cfe9108f640c7d61859a0e12c29aae8171fa62d64b98474bfe1`이다.
- 원래 `checkpoint_001_review_001` report/probe/log/currentness는 수정하지 않았다.
- typed AST → compiled immutable plan → evaluation, 단위/반올림/checked overflow/literal lowering/정상 연산 순서라는 핵심 설계는 변경하지 않았다.

## F01 — 입력 준비와 동적 payload admission

컴파일은 AST 복제 전에 source site/file/path, resource/result/unit reference, authored member identity/owner/value-unit payload를 checked scan한다. 기존 evaluation `max_work` 의미를 바꾸지 않도록 `max_compile_work`를 별도 한도로 추가했고, compiler task/segment/op/value/reference/scratch와 compiled plan 보유량을 allocation 전에 승인한다. canonical fingerprint는 같은 바이트열을 임시 `Vec<u8>`에 만들지 않고 SHA-256 state에 직접 기록한다.

평가는 기존 AST/plan/frame이 이미 소유한 메모리를 execution retained에서 제외한다. 대신 frame 값과 raw aggregate member의 순회·문자열/참조/값 payload, 정렬·중복 비교·owner key 비교를 work로 부과한다. raw member 수는 `ExprUsage.raw_members`, 중복 제거 후 실제 평가 수는 `dynamic_members`로 분리했다. 입력 member를 deep clone하지 않고 검증된 참조 index만 정렬하며, index buffer는 allocation 전에 승인한다.

실행 중 새로 확보하는 task/value-stack capacity, group BTree 저장소와 값 buffer, reduction 임시 결과, trace entry의 identity/unit payload를 retained에 포함한다. `retained_bytes`는 즉시 환급하지 않는 보수적 누적 계측이다. 가변 문자열은 32-byte work granule을 사용하고, 모든 합·곱은 checked arithmetic이다. 긴 반복은 `charge_work`를 거치므로 기존 cancellation check도 유지한다. 오류 원문에는 member identity를 넣지 않고 EffectSite와 resource unit은 bounded copy로 제한한다.

## F02 — compiler queue와 입력 폭 분리

Add 진입 시 직접 자식 수가 남은 node budget보다 크면 자식 task를 만들기 전에 거부한다. 통과한 Add는 `ContinueAdd { next_child }` frame 하나로 source order의 자식을 한 개씩 방문한다. Mul/Ratio/Clamp/Aggregate의 고정 폭 frame도 allocation 전에 compiler storage로 승인한다. operand 순서와 constant-fold overflow 위치는 유지했다.

## F03 — 동시 value stack과 구조 요구량

각 compiled segment의 개별 최대치 대신, 부모가 보유한 값과 nested aggregate member segment가 동시에 보유한 값을 합산한다. plan cost는 `max_depth`, `max_stack`, `stack_storage_slots`, `max_task_frames`를 기록한다. 평가 입구에서 compiled nodes/depth/stack이 더 작은 meter 한도를 넘으면 거부하며, 각 value push와 task push에서도 실제 동시 사용량을 확인한다. 실행 후 `peak_stack_values`와 `peak_task_frames`를 evidence로 남긴다.

`max_nodes`와 `max_depth`는 실행 중 AST를 다시 세는 한도가 아니다. immutable plan에 기록한 실제 구조 요구량과 더 작은 meter를 비교하는 한도다. task/group buffer는 `max_stack`에 섞지 않고 retained 및 `max_groups`로 보호한다.

## F04 — resource와 target capability 누적

모든 Resource read는 `resources`를 요구한다. 그중 `TargetEmployee`만 추가로 Snapshot의 `target_at_release` 또는 Live의 `target_at_consume`를 요구한다. SourceEmployee/SourceEquipment에는 target 조건을 붙이지 않았다. 정적 capability를 통과한 뒤에도 EvalFrame 실제 값과 unit 검사는 그대로 수행한다.

## 보수적 모델과 남은 경계

이 계측은 allocator metadata나 process RSS의 정확한 바이트 계약이 아니다. 커널이 새로 요청·보유할 container capacity와 동적 payload를 실제 요청보다 작지 않게 계산하기 위한 결정적 모델이다. `Vec::new()` 성장 buffer에는 최대 4배 slot, exact-capacity 임시 buffer와 동적 문자열에는 2배 여유를 적용하고 BTree node 여유도 별도 포함한다. 수정 전 reviewer allocator probe는 모델 테스트와 함께 다시 실행한다.

resolved frame의 실제 문자열/참조 payload는 compile 시 알 수 없으므로 plan의 `worst_case_work`는 고정 구조와 최소 preprocessing을 기록하고, 추가 가변 payload work는 runtime meter가 거부한다. frame의 기존 메모리는 retained에 합산하지 않지만 그 내용을 확인하는 work는 무료가 아니다. 실제 resource lease, authored RON/catalog, battle Snapshot/Live 연결과 저장 구조는 이번 교정 범위 밖이며 후속 통합 책임으로 남는다.

## 검증 결론

- 수정 전 원 unit 11건은 통과했고 원 review probe는 예상된 2 pass / 5 fail로 F01–F04를 재현했다.
- 수정 후 value_expr unit 18건과 correction probe 7건이 전부 통과했다. 경계 테스트는 compile work/retained, raw/canonical member, execution work/retained, node/depth/stack/task frame/group을 각각 limit−1/limit/limit+1에서 검사한다.
- 전체 `game_core --all-targets`는 lib 1,378건과 integration target 52건이 통과했다. 샌드박스 첫 실행은 Unix socket 생성이 EPERM으로 막혀 1건만 실패했으며, 동일 명령을 승인된 환경에서 재실행해 전부 통과했다.
- 수정 전 archive를 별도 빌드한 fingerprint와 교정본 fingerprint의 세 표본 바이트가 일치한다. 최초 verifier는 격리 workspace에서 부적합한 `--locked` 사용으로 실패했고, verifier 자체를 고친 뒤 재실행이 통과했다. 두 receipt를 모두 보존한다.
- parent source scope 311개 중 변경은 `compile.rs`, `eval.rs`, `mod.rs`, `tests.rs` 네 파일뿐이다. 원 checkpoint와 review 증거는 덮어쓰지 않았으며 baseline에 기록된 review 증거 7개가 현재도 모두 일치한다.
