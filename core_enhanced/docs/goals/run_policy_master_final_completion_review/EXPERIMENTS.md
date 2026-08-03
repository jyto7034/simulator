# Run Policy Master Final Completion Review Experiments

Status: complete

Implementation complete: true

Created: 2026-07-27

## 2026-07-27 — 활성화와 문서 상태 정합화

- 사용자 결정에 따라 과거 P06 hidden-boss 범위를 현재 master dependency와 완료 조건에서 제외했다.
- P07 prerequisite를 P00-P05 Complete/High로 변경하고 이 child Goal을 lazy 생성했다.
- README에 hidden-boss를 콘텐츠 개발 시 별도 Goal로 함께 수행할 후반 작업으로 유지했다.
- SavePoint, facility, combat-power decision record의 오래된 `미구현` header를 실제 completion review
  pointer가 있는 구현 이력 disposition으로 정정했다.
- 실제 hidden-boss production code/data/resource는 수정하지 않았다.

## Evidence Log

| Surface | Evidence | Result |
| --- | --- | --- |
| scope/dependency | master PLAN/README/decision disposition | P06 excluded, P07 active |
| checkpoint/active-run | `world/state.rs`, `world/node_flow.rs`, `world/active_run.rs`, server active-run store/handlers | P03~P05 state round-trip, monotonic envelope 분리 확인 |
| compendium | Core compendium domain, combat result commit, server compendium store/handlers | checkpoint 밖 local-install profile과 exact commit 확인 |
| facility/difficulty | facility template/generator/progression, RunPowerAuthority, live map/run RON | 같은 Floor generation commit과 저장된 node difficulty 확인 |
| monolith | `monolith.rs`, map encounter/shop/combat flow, policy RON | generic Shop requirement, remanifestation commit와 dormant key 확인 |
| Unity | NodeMap/checkpoint/compendium readers와 tests | strict semantic-slot projection 및 legacy NodeConfirm alias 제거 확인 |
| deferred content | scoped actor/encounter/VN/capstone/completion search | hidden-boss production 구현 없음, 기존 completion 유지 |

## Runner Log

| Command/Runner | Pass | Fail | Skip | Result |
| --- | ---: | ---: | ---: | --- |
| `cargo fmt --all -- --check` | n/a | 0 | 0 | 성공 |
| `cargo check -p game_core` | n/a | 0 | 0 | 성공 |
| `cargo check -p game_server` | n/a | 0 | 0 | 성공 |
| `cargo test -p game_core --test ron_loading -- --nocapture` | 23 | 0 | 0 | 성공 |
| `cargo test -p game_core live_ron --lib -- --nocapture` | 52 | 0 | 735 filtered | 성공 |
| `cargo test -p game_core --quiet` library | 787 | 0 | 0 | 성공 |
| `cargo test -p game_core --quiet` integration groups | 47 | 0 | 0 | 3/4/23/13/4 성공 |
| `cargo test -p game_server --quiet` | 52 | 0 | 0 | 성공 |
| focused Core/server seam runners | 44 | 0 | 0 | facility 9, monolith 6, generator 8, compendium 5, active-run 3, server persistence 12, admin SavePoint 1 |
| Unity focused legacy-alias test | 1 | 0 | 0 | 성공 |
| Unity EditMode job `c948fec856c24644ae75b52ace695fdf` | 259 | 0 | 0 | 성공 |
| Unity PlayMode job `0ff24f7b419c45d1a25a264171a30560` | 62 | 0 | 4 | 성공; live opt-in만 explicit skip |
| admin non-combat live WebSocket probe | 1 scenario | 0 | 0 | 성공 |
| authored SavePoint checkpoint/load WebSocket probe | 1 scenario | 0 | 0 | 성공; save 0/3, load 1/2 |
| Python probe syntax | 3 scripts | 0 | 0 | 성공 |
| scoped `git diff --check` | n/a | 0 | 0 | 성공 |

## Finding Log

| ID | Severity | Owner | Finding | Disposition |
| --- | --- | --- | --- | --- |
| F-01 | Medium | P00/P01 external probe/canonical | probe와 문서가 제거된 Support/Rest, 확률적 1층 SavePoint 탐색, 오래된 checkpoint shape를 사용 | SavePoint/Maintenance exact contract와 authored fixture/location으로 정정, 두 live probe 통과 |
| F-02 | Medium | P03 Unity reader | NodeConfirm reader가 제거된 `node_kind_id`/`node_type` alias를 허용 | exact `kind_id`/`category`만 허용하고 negative regression 추가, focused + EditMode 259 통과 |
| F-03 | High | P00/P01 Core/server admin seam | `admin_enter_save_point`가 템플릿 밖 `admin_fixture` slot을 저장해 active-run publish가 실패 | authored SavePoint slot을 사용하도록 수정, checkpoint validation test와 live publish/load 통과 |

## 2026-07-27 — 최종 독립 감사 완료

- P00~P05 runtime/data/persistence/server/Unity source를 predecessor review와 독립적으로 확인했다.
- F-01~F-03은 정책 선택이나 hidden-boss 콘텐츠 없이 승인된 exact contract로 수정 가능한 범위였다.
- Unity compile 직후 Console error는 0건이었다. PlayMode 종료 뒤 `Error` 필터에 잡힌 1건은 Unity Test
  Runner의 `TestResults.xml` 저장 경로 알림이며 compile/runtime failure가 아니다.
- 실제 hidden-boss actor, skill, encounter, 관리자 VN, capstone, FinalBoss 재분류와 completion interception은
  추가하지 않았다.
- 최종 verdict는 `Complete / High`다.
