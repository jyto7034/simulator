# Checkpoint-Owned Floor Anchor Correction Experiments

Status: complete

Implementation complete: true

Created: 2026-07-28

## 2026-07-28 — Child Goal 초기화

- Master P01 correction completion review: `Complete / High`.
- P02 child 경로는 생성 전에 존재하지 않았다.
- Master P-02가 payload/envelope ownership과 no-migration 경계를 승인해 policy question은 없다.
- 관련 Core/server/canonical 경로는 기존 implementation campaign 변경과 겹친다.
- Production code/schema/canonical 변경과 P02 test 실행은 아직 없다.
- 다음 작업: child 문서 완독, current capture/load/advance/envelope/backup call graph와 scoped diff 조사.

## 2026-07-28 — Current-state와 anchor defect 재현

### Current evidence

- `RunCheckpointPayload`는 map/progression과 run-local 상태를 소유하지만 power model version과 current Floor
  anchor가 없다.
- `restore_run_checkpoint_payload()`는 map을 복원하지만 `RunPowerAuthorityState`를 건드리지 않는다.
- Active-run Core V1 payload는 checkpoint만 담고 model/version, run peak, anchor를 모두 별도 export field로
  전달한다.
- Host envelope version 2의 monotonic metadata가 loads, model version, run peak, anchor를 담고 backup 생성 때
  peak와 anchor를 max-merge한다.
- 따라서 Floor 1 checkpoint에서 Floor 2로 진행한 뒤 load하면 Floor 1 map/node generation과 Floor 2 anchor가
  함께 남는다.

### 수정 전 regression

- 새 `checkpoint_load_restores_floor_anchor_and_gate_replay_is_deterministic`는 Floor 1 anchor 100을 저장하고
  Gate로 Floor 2 anchor 125를 만든 뒤 checkpoint를 load했다.
- 수정 전 결과: 0 passed, 1 failed, 789 filtered.
- 직접 mismatch: 복원 기대 anchor 100, 실제 anchor 125.
- 이 결과는 P-02의 generation split을 재현한다. Production 변경은 checkpoint payload ownership, Core payload
  V2와 host envelope V3, anchor max-merge 제거로 제한한다.

## 2026-07-28 — Checkpoint ownership과 strict schema 구현

### Core

- `RunCheckpointPayload`를 strict `deny_unknown_fields` schema로 만들고 `power_model_version`과
  `current_floor_power_anchor`를 추가했다.
- SavePoint capture가 runtime authority의 두 값을 payload에 저장하고 load/continue가 map보다 먼저 exact
  복원한다. `run_peak_certified_power`는 복원하지 않아 high-water를 유지한다.
- Checkpoint validation은 live model version, non-zero anchor, 모든 manifested node의 model/anchor/profile
  일치를 검사한다.
- Active-run Core payload는 explicit `version: 2`를 가진 V2로 교체했다. Export top-level에서는 model/anchor를
  제거하고 loads/run peak만 남겼다.
- Active-run transaction rollback merge는 같은 model generation의 run peak만 max-merge하며 anchor/model은
  rollback snapshot이 소유한다.

### game_server

- Active-run envelope를 version 3으로 올리고 monotonic metadata에서 model version과 Floor anchor를 제거했다.
- Rolling backup은 이전 opaque Core payload를 그대로 보존하고 loads/run peak만 current와 max-merge한다.
- Actor continue call도 payload/loads/peak만 Core에 전달한다.
- Old envelope version 2, 제거된 monotonic anchor field, checksum mismatch와 unsupported version을 migration
  없이 거부한다.

## 2026-07-28 — Focused/broad 검증과 종료 판정

### Focused

| 검증 | Pass | Fail | 핵심 증거 |
| --- | ---: | ---: | --- |
| cross-Floor load/Gate replay | 1 | 0 | anchor 100 복원, 재진행 anchor 125 고정, map 동일 |
| active-run Core round-trip/schema | 1 | 0 | V2, checkpoint anchor/model, peak/load 보존, legacy/mismatch reject |
| server active-run group | 13 | 0 | real files, backup payload 보존, loads/peak high-water, failure/invalid lifecycle |
| old envelope/removed field exact | 1 | 0 | envelope V3 shape와 V2/anchor-field reject |

### Broad와 hygiene

- Core lib: 790 passed, 0 failed.
- game_server: 54 passed, 0 failed.
- Workspace all targets: Core 790, integration 3/4/23/13/4, server 54, 실패 0.
- `cargo check --workspace --all-targets`, `cargo fmt --all -- --check`, scoped `git diff --check`: 통과.
- Runtime/canonical legacy search에서 Core V1, envelope V2, anchor max-merge와 stale canonical 문구가 남지
  않았다. Test의 removed-field injection만 의도적으로 남는다.
- Unity source/canonical 검색에서 internal model/anchor/peak 필드 consumer는 0건이고 public
  `run_checkpoint.loads_used`/ContinueRun 계약만 존재했다. Wire DTO가 바뀌지 않아 Unity 변경과 Unity MCP
  test 실행은 적용 대상이 아니었다.
- 독립 completion review: `Complete / High`.
