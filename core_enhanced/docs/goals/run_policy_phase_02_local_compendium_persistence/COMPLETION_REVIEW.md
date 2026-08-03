# P02 Local Encounter Compendium Persistence Completion Review

Reviewed: 2026-07-27

Review basis: Goal 완료 표시에 의존하지 않고 현재 Core profile/producer/rollback, game_server 실제 파일
store와 actor lifecycle, 원본 Unity Editor consumer, canonical 및 broad validation을 다시 확인했다.

## Audit Table

| Item | Verdict | Long-term fit | 핵심 증거 |
| --- | --- | --- | --- |
| Core profile과 dormant boundary | Complete | High | strict discovered/defeated subset, persisted/current-catalog 검증 분리, idempotent union |
| 실제 encounter producer와 projection | Complete | High | primary victory, 실제 non-primary death, authored 전체 placeholder projection |
| Durable local-install store | Complete | High | checksum/current+backup/lock/atomic publish/bootstrap/reset 실제 파일 test |
| Active-run/profile transaction 분리 | Complete | High | 양방향 write failure와 rollback 독립성, stale profile copy 없음 |
| Server·Unity·canonical 수렴 | Complete | High | singleton actor injection, dirty/fatal lifecycle, strict Main Menu UX와 broad Editor 통과 |

## Core Profile, Producer And Projection

Expected behavior:
- abnormality/corrosion의 발견·격파가 run 밖 singleton profile에 단조 누적되고 defeated는 discovered의
  부분집합이다. 과거 dormant ID는 보존하고 신규 commit만 현재 catalog에서 검증한다.
- 실제 encounter에서 primary 승리와 승리 attempt의 실제 non-primary `UnitDied`만 격파한다.
- 현재 catalog 전체 authored 순서를 투영하되 미발견 metadata는 숨긴다.

Runtime evidence:
- `src/game/encounter_compendium.rs`: strict persisted state, current-catalog commit, battle-result extractor와
  idempotent merge
- `src/game/world/combat.rs`: consumed battle result 뒤 profile commit
- `src/game/behavior.rs`, `src/game/world/snapshot.rs`: authored full projection과 placeholder
- `src/game/world/active_run.rs`, `state.rs`: active-run restore/rollback에서 current profile 보존

Test evidence:
- malformed/subset/dormant/current commit/idempotence, primary/non-primary death/survivor/defeat,
  authored placeholder와 active-run rollback independence tests

Verdict: Complete / High

## Durable Store And Host Lifecycle

Expected behavior:
- profile은 transport UUID와 무관한 local-install singleton이다. 별도 checksum envelope/current/backup/
  lock/bootstrap을 사용하고 손상을 빈 상태로 조용히 대체하지 않는다.
- profile write 실패는 gameplay나 active-run file을 rollback하지 않으며 최신 전체 union을 dirty로
  유지해 retry한다.

Runtime evidence:
- `game_server/src/persistence/encounter_compendium_store.rs`: v1 store, actual publish/recovery/reset와
  Clean/Recovered/Dirty/Fatal lifecycle
- `game_server/src/persistence/mod.rs`: platform replace/directory sync primitive
- `game_server/src/env.rs`: OS application-data root resolution
- load-balance/player actors: one shared profile runtime, bootstrap gating, retry/reset/notification와 bounded
  stop flush

Test evidence:
- real-filesystem missing/current/backup/dual corrupt/unsupported/checksum/lock/publish-failure/reset/dirty retry
- different player UUID singleton, 양방향 store failure independence, fatal gate와 recovery notification once

Legacy/fallback audit:
- per-player `HashMap` durable authority, silent reset, compatibility alias/dual reader와 active-run payload copy가
  official runtime에 없다.

Verdict: Complete / High

## Unity And Canonical Convergence

Expected behavior:
- Unity는 exact Core projection/status만 읽고 Main Menu에서 placeholder와 발견/격파 수를 표시한다.
- dirty/recovered/fatal 상태를 노출하고 fatal에서는 retry 또는 확인된 전체 reset만 허용한다.

External evidence:
- Unity strict model, Main Menu composer/view, retry/reset command와 confirmation modal
- generated contract fixture와 EditMode/PlayMode contract/render/dispatch tests
- Core `game_rulebook.md`, `core_runtime_contract.md`, `data_loading_contract.md`, server bridge canonical과
  Unity `unity_core_contract.md`, `unity_client_implementation_goal.md`가 같은 boundary를 기록한다.

Final validation:
- `CARGO_INCREMENTAL=0 cargo test --workspace --all-targets --quiet`: Core 765,
  integrations 3/4/23/13/4, server 51, 실패 0
- Unity full EditMode: 251 passed, 0 failed
- Unity full PlayMode: 62 passed, 0 failed, explicit live-server opt-in 4 skipped
- MCP restart 뒤 asset refresh/domain reload 완료, Editor ready, Console error 0

Verdict: Complete / High

## Cross-Component Findings

- 새 run, RunFailed, checkpoint load, active-run continue/rollback은 profile set을 rewind하지 않는다.
- active-run/profile write 실패는 서로의 gameplay/file commit을 rollback하지 않는다.
- 현재 catalog metadata authority와 persisted dormant set authority가 분리돼 있다.
- P05는 같은 generic commit seam에 신규 identity만 연결할 수 있고 storage/profile 의미를 재작성할
  필요가 없다.
- material alias, fallback, player-keyed profile authority 또는 competing copy를 발견하지 않았다.

## Remaining Risk

- 배포 OS 전원 차단 crash 시험과 Windows Rust cross-target compile은 별도 플랫폼 validation 영역이다.
  실제 filesystem failure test, fsync/verify/replace ordering과 명시적 Windows API 경계에는 완료를 막는
  material gap이 없다.

## Policy Questions

- 없음.

## Final Verdict

- Verdict: Complete
- Long-term fit: High
- Improvement class: none
- P03 activation recommendation: 승인. P02 profile/storage transaction을 facility run-state와 결합하지
  않고 P01 active-run/checkpoint 확장 seam만 사용한다.
