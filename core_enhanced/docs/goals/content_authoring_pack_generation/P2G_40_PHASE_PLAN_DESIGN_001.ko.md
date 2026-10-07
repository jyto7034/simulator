# R02 최소 Phase 계획 — P2G-40 후보

2026-09-10 KST. Exact Plan `PP-CAPG-P2G40-001`, Freeze `FPF-CAPG-P2G30-001`, WR `WR-CAPG-P2G20-001`. 구조 authority는 동명의 JSON이다. 표준 Receipt 통과와 complete result 전에는 non-current다. 게임 구현/검증 완료가 아니다.

Repair Basis: applicable obligation/resolution/supersession sets explicit empty. Parallelization: none approved.

정책와 acceptance는 Freeze selector에서만 파생한다. R02 AT02의 loader/new-run/save 전환이 하나의 최소 Atomic Group이며, 실제 완료는 별도 durable validation node가 증명한다. R03 이후 기능을 전제로 이 Phase를 완료하지 않는다.

## Work Package와 순서

- `WP-CAPG-01` — 외부 input→owned snapshot→typed raw resolve→최종 lowering/validation/generated setup→opaque immutable generation 게시. 공식 pack packaging과 production embedded 경로 제거를 함께 소유. 공식 팩과 외부 A/B가 같은 public loader를 통과하고 invalid candidate는 token 없음. 모든 winning 참조/실제 acquisition 소비자에 Add/Replace/Remove 적용. production fallback 없고 지원/입력 제한 직접 거부.
- `WP-CAPG-02` — CoreApplication/GameCore generation authority·기존 StartNewGame gating 이후 briefing 이전 pin·state/seed/token/output prepared commit·최신 content-bound checkpoint capture/cache/export/continue/explicit load 동시 전환. A 유지와 성공한 B briefing/starter/전투/저장 결속, 허용되지 않은 시작/late preparation/mismatch 실패 원자성. 새 application에서 동일 content로 run identity/state/RNG 복원. 구형/unknown schema·변조·content mismatch 거부.
- `WP-CAPG-03` — R02 전체 수용 기준의 durable external-directory fixtures·공개 API/release 소비·회귀·완료 증거를 소유. 최종 core baseline과 R01 영향 검사/current 감사 복구를 이 실행 node에 배정. 8 IVA와 14 acceptance direct evidence를 닫고 core lib/tests 및 release public loader를 실제 실행/확인. R01 변경 영향은 fresh separate GEX-20으로 currentness 복구 후 R02 fresh 완료 감사에 제공.

`AG-CAPG-01` = `WP-CAPG-01` + `WP-CAPG-02`. 두 방향 producer/consumer·저장 전환 의존으로 coimplement_atomic SCC를 구성한다. 응축 결과 `AG-CAPG-01` → `WP-CAPG-03`를 유일한 `PHASE-CAPG-01`에 배정한다. 같은 Atomic Group이 모든 runtime 작업을 한 transaction으로 합치라는 뜻은 아니다.

## 유효 중간 상태

next R03 absent: official/custom external packs compile and publish; A remains pinned, eligible B new-run commits before briefing; latest storage matches content; profile/files preserved. single token authority; all reader/writer/cache paths current; supported capability and input limits enforced; order/fingerprint agree; old reader/embedded fallback absent; failed publication/preparation/restore unchanged; unreferenced generations reclaimable.

## 공유 계약

| Contract / source | Meaning owner and producer/consumer | Implementing / validating node | Availability / remaining choice |
| --- | --- | --- | --- |
| SC01 source/typed resolve — L01–04 | Freeze E01/E02/E06; snapshot and typed resolver produce winning definitions/original locators for compiler and all catalog consumers | AG-CAPG-01 / WP-CAPG-03 | New API after AG validation. Full typed definition operations; local IDs/UUID/reference-keyed extensions distinct. No file-order winner. |
| SC02 semantic generation — L04/L05 | Freeze E01/E03/E06; final validator publishes one opaque database+identity token consumed by app/new-run/save | AG-CAPG-01 / WP-CAPG-03 | Hash actual authored runtime array order and resolved contracts/selection, separate bytes receipt. Exclude derived caches/source metadata. |
| SC03 app commit/profile — L05 | Freeze E03/E05; existing gate then draft installs candidate before briefing; final commit carries state/seed/token/output | AG-CAPG-01 / WP-CAPG-03 | Existing action eligibility preserved. Failed B retains A; AbandonRun preserves profile. No live setter/ignored bool bypass. |
| SC04 latest storage — L06 | Freeze E04/E05; capture/cache/export and all readers share content/contract authority | AG-CAPG-01 / WP-CAPG-03 | Only newest format after cutover, no guessed legacy fingerprint/conversion/deletion. Check before mutation. R01 historical retention evidence stays historical. |
| SC05 R01 impact — V08/F05 | R02 owns expected successor source change and current regression; fresh auditor independently assesses preserved R01 accepted contract | WP-CAPG-03 / separate fresh GEX-20 | Existing R01 Complete/High is current at planning entry. Restore currentness after actual delta before claiming R02 completion. |

## Integration Validation Ownership

- `IVA-CAPG-01` → `WP-CAPG-03` / `PHASE-CAPG-01`: 동일 바이너리·실제 A/B directory, 공식 pack packaging과 외부 원문 수정 → 실제 서로 다른 정의/실행 및 source path, embedded fallback 부재. 격리 공식 source·별도 output·기존 custom live_data fixture를 준비해 packaging 전후 source/custom bytes 보존을 대조. E01/E05/E06
- `IVA-CAPG-02` → `WP-CAPG-03` / `PHASE-CAPG-01`: 독립 namespace의 동일 local 이름, global override, recipe extension, explicit winner/chain, shuffled pack/file 목록 → exact typed key·안정된 실행 순서/지문. 참조를 함께 정리한 유효 Remove 뒤 최종 catalog·조회 index·등록/획득 소비자에서 제거 결과를 관찰. 모호 writer·중복/누락 dependency·cycle·없는 target는 오류. E02/E06
- `IVA-CAPG-03` → `WP-CAPG-03` / `PHASE-CAPG-01`: cross-pack skill/corroded preset, loser의 폐기된 dangling ref, winner가 남긴 dangling buff/reward/registration → 최종 목록만 검증, late compile/generated setup 실패는 미게시. E01/E02/E03
- `IVA-CAPG-04` → `WP-CAPG-03` / `PHASE-CAPG-01`: A 런/briefing 유지 중 B 게시, 새 B 런, invalid B와 B briefing 준비 실패·허용되지 않은 StartNewGame → A state/profile/seed/generation/출력 무변경, 성공 B는 briefing/starter/전투 pin. A/B export fingerprint가 각 pinned 세대와 일치함을 직접 대조. E03/E05
- `IVA-CAPG-05` → `WP-CAPG-03` / `PHASE-CAPG-01`: matching 최신 콘텐츠의 새 application restart/continue/explicit checkpoint load → 복원 전후 run identity·상태·난수 진행 또는 이어지는 결정론적 결과를 직접 대조. 같은 ID 다른 내용, contract mismatch·old V4/V5·unknown fields·checksum·load-limit는 기존 상태/파일 무변경 오류. E04/E05
- `IVA-CAPG-06` → `WP-CAPG-03` / `PHASE-CAPG-01`: 실제 외부 manifest/catalog의 구형 schema·미지원 authoring/execution capability·실행되지 않는 unsupported opcode, 누락/빈 selection·duplicate ID/path·case collision·절대/탈출/symlink·동시변경·file/byte/depth/dependency 한계 경계 → source 진단·token 미반환·embedded fallback 부재. 같은 입력 재시도/주석 변경·명시적 순서 변동의 지문, strong reference 회수 관찰. E01/E06
- `IVA-CAPG-07` → `WP-CAPG-03` / `PHASE-CAPG-01`: 기존 catalog/item/skill/fragment/preview/map/VN/new-run/profile/checkpoint 회귀와 release lib/실제 pack 소비 binary. 전체 core baseline을 최종 owning node가 수행. E01–E06
- `IVA-CAPG-08` → `WP-CAPG-03` / `PHASE-CAPG-01`: R01 IVA02/04/05/06에 대한 source/parser/publication/store 변경 영향 map, 필요한 nonzero 회귀와 current target의 fresh R01 GEX-20. 과거 V4 보존 delta와 현재 R02 전환을 구분. R01 accepted outputs 보존

모든 신규 fixture·실제 public consumer·최종 core baseline은 WP03 소유다. R01 감사는 source 변경 후 fresh 별도 읽기 전용 감사자가 수행한다. 과거 감사·시험을 새 R02 PASS로 쓰지 않는다.

## Known Unknown

KU30-01 exact types/manifest/adapters/budgets remain AG owner technical detail inside L01–06. KU30-02 test selectors/performance observations belong to WP03. KU30-03 predecessor currentness restoration required before R02 completion. KU30-04 external host deployment excluded, API adaptation documented. Material meaning/eligibility/persistence-loss changes return to P2G-10; no silent new policy.
