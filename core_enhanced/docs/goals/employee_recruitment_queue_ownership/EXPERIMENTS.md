# Employee RecruitmentQueue Ownership Experiments

실제로 수행한 조사, 구현과 검증을 시간순으로 기록한다.

## 2026-07-18 — Child Goal 초기화

### 수행한 확인

- Phase 2 경로 존재 여부를 `phase2_missing` 명시 출력으로 확인했다.
- Phase 1 `complete`와 completion review를 확인했다.
- 관련 Core paths가 기존 modified 상태이고 Master/Phase 1 docs가 untracked임을 확인했다.

### 변경한 내용

- Phase 2 필수 문서 3개를 생성했다.
- production 파일은 변경하지 않았다.

### 실제 실행된 test 수

해당 없음. 신규 Child 문서 초기화다.

### 다음 판단

직접 의존 정책과 current production paths를 조사한다.

## 2026-07-18 — Runtime discovery와 production baseline

### 확인한 사실

- 기존 HQ 후보는 `recruitment_candidates_for_node`가 non-starter definitions를 node seed로 shuffle/truncate해 만들었다.
- `RunState`, `RunCheckpointPayload`와 Floor carry-forward에는 recruitment state가 없었다.
- HQ active content가 full candidate definitions를 보유했고 채용은 roster/UUID를 먼저 변경한 뒤 node completion을 실행했다.
- 응급지원은 자원을 먼저 지급한 뒤 completion, HQ shop은 candidate 반환 없이 active content를 교체했다.
- public candidate/result/command와 Unity reader는 Phase 1의 `employee_definition_id + EmployeeDefinition profile` shape를 이미 사용했다.
- SavePoint checkpoint는 Core in-memory clone이며 public save는 planning return에서만 가능하다.

### production 변경 전 baseline

직전 Phase 1 완료 검증을 그대로 production baseline으로 사용했다. Phase 2 discovery 전 production 변경은 없었다.

- Core: 759 pass, 0 fail
- game_server: 30 pass, 0 fail
- Unity EditMode: 240 pass, 0 fail
- Unity PlayMode: 57 pass, 0 fail, server 의존 explicit skip 4

## 2026-07-18 — RecruitmentState와 HQ transaction 구현

### 변경한 내용

- `src/game/recruitment.rs`에 run-seeded finite queue와 active session aggregate를 추가했다.
- unified catalog ID를 정렬한 뒤 고정 namespace로 한 번 shuffle하고 selected starter IDs를 제외했다.
- queue/session에는 `employee_definition_id`만 저장하고 full profile은 catalog에서 투영한다.
- `RunState`와 checkpoint payload가 aggregate 전체를 소유하고 Floor/run-complete가 carry-forward한다.
- HQ 진입은 queue head를 active session으로 이동하고 같은 node의 반복 진입은 기존 session을 반환한다.
- 채용은 선택 ID를 staged roster로 이동하고 미선택 ID를 session order로 queue tail에 반환한다.
- 응급지원과 HQ shop은 모든 active 후보를 session order로 tail에 반환한다.
- HQ action은 game state, run, roster, UUID, resources, fragments와 checkpoint를 rollback snapshot으로 감싼다.
- HQ active content의 candidate definition 사본을 제거해 run active session만 ownership authority가 되게 했다.
- combat recovery capability는 static non-starter 존재 여부가 아니라 실제 queued candidate 존재 여부를 사용한다.
- checkpoint load 전에 Queue/Session/Roster exact ownership을 검증해 손상 상태를 부분 복원하지 않는다.
- node-seeded recruitment reroll 함수와 seed namespace를 제거했다.

### focused 검증

```bash
CARGO_TARGET_DIR=/tmp/core_enhanced_target cargo test game::recruitment::tests -- --nocapture
```

- 3 pass, 0 fail

```bash
CARGO_TARGET_DIR=/tmp/core_enhanced_target cargo test game::world::tests::node_sessions -- --nocapture
```

- 16 pass, 0 fail
- 포함 증거: fixed session, invalid recruit rollback, exact-one roster 이동, tail 순서, 응급지원/상점 반환, checkpoint exact restore, invalid checkpoint rejection

```bash
CARGO_TARGET_DIR=/tmp/core_enhanced_target cargo test game::world::state::tests -- --nocapture
```

- 2 pass, 0 fail
- Floor와 terminal run carry-forward 확인

```bash
CARGO_TARGET_DIR=/tmp/core_enhanced_target cargo test no_deployable_defeat_allows_only_immediate_typed_recruitment_then_rechecks -- --nocapture
```

- 1 pass, 0 fail
- staged recruit가 recovery recheck 전에 roster에 반영됨을 확인

### broad 검증

```bash
CARGO_TARGET_DIR=/tmp/core_enhanced_target cargo test --all-targets
```

- Core 총 766 pass, 0 fail
- lib 722, live item 3, live catalog 4, RON 20, skill runtime 13, skill suite 4

```bash
CARGO_TARGET_DIR=/tmp/game_server_target cargo test --all-targets
```

- game_server 30 pass, 0 fail

Unity MCP:

- EditMode job `7633fb1e6dfd43d38acf73fd346d6caa`: 240 pass, 0 fail, 0 skip
- PlayMode job `ee9d88bebc254a50b2e8b91f5572e3eb`: 57 pass, 0 fail, 4 explicit skip
- 4 skip은 모두 실행 중 server/admin server가 필요한 기존 live tests다.

### 정적 검증

```bash
git diff --check
rg -n "recruitment_candidates_for_node|4851_5F52_4543_5255" src ../game_server '/mnt/f/unity projects/ark/Assets' -g '*.rs' -g '*.cs'
```

- whitespace error 없음
- legacy node-seeded recruitment function/namespace 검색 결과 없음

### clippy 결과

```bash
CARGO_TARGET_DIR=/tmp/core_enhanced_target cargo clippy --all-targets -- -D warnings
```

- 실패. 현재 dirty baseline의 battle/world/test 파일에 unusual byte grouping, too-many-arguments, large-enum-variant 등 117개 기존 lint 진단이 있다.
- 첫 진단과 대부분은 Phase 2 범위 밖이며 Phase 2 production 파일의 compile warning은 없었다.
- 기능·계약 gate는 compile, focused와 broad runtime tests로 통과했다. 범위 밖 lint debt를 수정하지 않았다.

### 결론

Phase 2 사용자-visible 완료 조건과 완료 gate를 충족해 `complete`로 전환한다.
