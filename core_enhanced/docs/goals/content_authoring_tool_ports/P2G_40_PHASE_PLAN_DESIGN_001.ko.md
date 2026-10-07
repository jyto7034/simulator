# R03 최소 Phase 계획 — P2G-40 후보

2026-09-10 KST. Exact Plan `PP-CATP-P2G40-001`, Freeze `FPF-CATP-P2G30-001`, WR `WR-CATP-P2G20-001`. 구조 authority는 동명의 JSON이다. 표준 Receipt와 complete result 전에는 non-current다. 게임 구현/실행 완료가 아니다.

Repair Basis: applicable obligation/resolution/supersession sets explicit empty. Parallelization: none approved.

정책와 acceptance는 Freeze selector에서 파생한다. [CRA](CODE_REVIEW_001.ko.md)의 P01–07 경계를 같은 source/control/session surface의 최소 Atomic Group으로 묶고, 실제 완료 증거는 별도 validation node가 소유한다.

## Work Package와 코드 기반 순서

- `WP-CATP-01` — 공통 authoring validate/describe/explain의 source/diagnostic contract. 실제 owned snapshot/parser/resolver/final validator/compiler/generated setup에서 code/phase/known pack-kind-location-related data를 보존하고 immutable source/resolution 설명을 publication에 결속. 모든 producer phase의 실제 잘못된 external input을 정확한 source와 API/CLI parity로 거부; current descriptor/definition/effect/origin/compiled graph 조회, readonly/identity/metadata lifetime 보존. 선택 진단/source metadata 한계 실제 검증.
- `WP-CATP-02` — 공통 authoring simulate/session과 thin default-release CLI. Private opaque CoreApplication fork/staging, generic finite observed-result bindings, 실제 public battle 준비/command/clock, producer-level fuel/cancel/queue control, terminal 전 source observation, step/complete B aggregate commit과 receipt/lifetime 동시 전환. 독립 external effect A/B의 실제 nonzero battle delta·replay/on-off invariance. Valid-load failed-prep/low-fuel/cancel/mandatory-output-limit 반복 A 보존과 이후 회복. Trace overflow gap은 commit을 바꾸지 않음. 초기 load/prep 및 단일 runtime call 내부의 budget 중단, drop와 public privacy 검증.
- `WP-CATP-03` — R03의 13 acceptance/9 validation seams를 실제 durable tests·isolated external fixtures·public API/CLI release consumers·raw receipts·replay/rollback/reclamation/cost 관찰과 선행 currentness로 입증. 13 acceptance와 IVA01–09 실제 nonzero 근거를 모두 닫고 적용 가능한 R01/R02 currentness를 복구. Root 구현 종료 후 별도 fresh R03 GEX-20에 exact source/evidence/known limits 제공. Source 문제는 AG01 owner로 교정 반환.

`AG-CATP-01` = `WP-CATP-01` + `WP-CATP-02`. 공통 loader가 실행하는 generated setup/start부터 operation control이 필요하고, 그 결과/source/diagnostic을 실제 trial/CLI가 소비하므로 두 방향 dependency를 함께 닫는다. 응축 결과 `AG-CATP-01` → `WP-CATP-03`를 단일 `PHASE-CATP-01`에 배정한다. 각 gameplay command의 기존 transaction 경계는 유지한다.

## 유효 중간 상태

R04 and later absent: common production authoring library/CLI validates/describes/explains and actually simulates independent external content; failed B preserves A and success prepares actual B before swap. one public core authority; load-time/command/clock control and required result prepare before publication; current support and exact known sources; optional trace isolated from gameplay admission/RNG; terminal observation; finite lifetime/reclamation; V6/profile/pin/privacy preserved; no embedded/content-specific fallback.

일반 core의 기존 behavior는 작업 중 유지한다. 완료라고 게시하는 새 authoring surface는 source/control/observer/session/consumer가 함께 유효해야 한다. 다음 R에서 안전장치나 저장/취소를 처음 붙이는 중간 상태는 허용하지 않는다.

## 공유 계약

| Contract / source | Meaning owner and producer/consumer | Implementing / validating node | Availability / remaining choice |
| --- | --- | --- | --- |
| SC01 typed diagnosis/source — CRA P02/P03 | Freeze E01/E02/E04; acquisition/parser/writer/final/compiler/generated producer → common facade/CLI/explain | AG-CATP-01 / WP-CATP-03 | Existing source data are partial; new common envelope only usable after coordinated implementation. Preserve actual known precision/R02 contributors and existing semantic hash inputs. |
| SC02 operation control — CRA B06/P06 | Freeze E02/E03/E05/E06; authoring operation creates candidate-local limits/cancel → loader generated setup, startup, command/effect/target/deferred/clock execute → typed precommit failure | AG-CATP-01 / WP-CATP-03 | New opt-in route, no current bounded-call claim. Enforce before expansion/construction, no shared mutable A/B quota, no new ordinary gameplay outcome. Exact limits measured in GEX. |
| SC03 authority/session — CRA P04 | Freeze E03/E05/E06; private opaque CoreApplication fork and public commands → mandatory bounded result → tool step commit or complete B reload swap | AG-CATP-01 / WP-CATP-03 | CoreApplication clone service does not exist yet. Old A untouched by candidate failure/cancel; prior accepted steps retained. No raw inner public access/checkpoint substitution. |
| SC04 observation/receipt — CRA P05/P06 | Freeze E03/E04/E06; actual execution-source/occurrence/terminal producer → staged readonly observation and owned receipt → explain/CLI | AG-CATP-01 / WP-CATP-03 | Capture before active removal. Optional trace cap/work independent of mandatory receipt/admission/gameplay fuel/bindings; trace overflow is explicit gap with same commit/RNG. No log-seq semantic ID. |
| SC05 lifetime/store — CRA P03/P04/P06/P07 | Freeze E04/E05/E06; opaque generation and trial aggregate → bounded session/DTO/drop; existing V6/profile owners remain authority | AG-CATP-01 / WP-CATP-03 | Process-local disposable state; no new midbattle save/personal-file behavior. Receipt DTOs must not pin whole historical generations. |
| SC06 current predecessor/release — CRA P07/V08/V09 | Freeze E01–E06; current R01/R02 source/actual evidence → changed input impact → current regression/fresh reviewer → R03 completion | WP-CATP-03 / separate applicable fresh GEX-20 | R01/R02 current now; expected R03 delta requires explicit restoration or positive non-impact reuse, never inherited stale PASS. New performance path requires new measurement. |

## Integration Validation Ownership

- `IVA-CATP-01` → `WP-CATP-03` / `PHASE-CATP-01`: 같은 독립 external pack을 default-release public library와 CLI로 validate/describe/explain. 실제 token/fingerprint/source/지원 의미 일치; current R01 editable 조합 positive/unsupported negative; 다른 kind/compiled graph 설명
- `IVA-CATP-02` → `WP-CATP-03` / `PHASE-CATP-01`: syntax/unknown field/malformed manifest·contract/capability/ambiguous winner/final dangling reference/summoned-boss compile/generated policy-wave·custom boss source/trial invalid authority의 같은 입력. API/CLI code/phase/source/related/available expected-actual parity, unknown precision 부재, zero candidate publication
- `IVA-CATP-03` → `WP-CATP-03` / `PHASE-CATP-01`: source 사본의 actual effect 수치만 바꾼 A/B를 일반 public scenario로 준비하고 nonzero advance/실제 effect execution까지 호출. 관찰 HP/resource/result delta와 actual source/fingerprint 결속, actual commands/nonzero counts/exit와 raw receipt 저장
- `IVA-CATP-04` → `WP-CATP-03` / `PHASE-CATP-01`: 같은 initial/player/profile provenance/content/seed/template+resolved input/clock 재실행; trace on/off와 repeated/delayed/area, log eviction/terminal source, semantic identity가 log correlation에 의존하지 않음. 동일 gameplay/RNG/next decision 및 정확한 gap 표시
- `IVA-CATP-05` → `WP-CATP-03` / `PHASE-CATP-01`: exact definition/effect/origin/compiled graph 설명과 readonly Preview 반복 전후 공개 snapshot/reconnect/revision, 이어지는 같은 command/clock 결과 일치. 누락 정의·unsupported query는 명시 거절, 가짜 forecast 없음
- `IVA-CATP-06` → `WP-CATP-03` / `PHASE-CATP-01`: 실제 A→B 성공; bad syntax/compile/generated B 및 valid token+실제 trial prep 실패 B를 각각 반복; time-zero/출력 한계/취소 실패 A token/state/seed/input/receipt 불변; 이후 valid B 회복. 실패 단계 앞의 committed 성공 보존; 소유 참조 drop 확인
- `IVA-CATP-07` → `WP-CATP-03` / `PHASE-CATP-01`: 빈/최소/정확 cap/cap+1/잘못된 nested request/binding/clock overflow, load-generated setup과 valid-token trial startup에서 각각 낮은 fuel/취소, same-time 반복·command/deferred fan-out·queue growth의 작은 fuel 한계와 취소. 한 호출 내부에서 실제 중단·명시 오류·rollback, trace-on만 optional cap을 넘는 입력에서도 trace-off와 commit/fuel/continuation/RNG 일치 및 gap, mandatory receipt 오류는 양쪽 후보 미게시
- `IVA-CATP-08` → `WP-CATP-03` / `PHASE-CATP-01`: 기존 meaningful core suite, 관련 R01 effect/provenance/delayed·R02 loader/pin/V6/profile/failed-candidate invariants, all-target/default release와 실제 독립 positive API/negative privacy consumers. Source 변화로 침범한 선행 currentness·fresh audit 복구
- `IVA-CATP-09` → `WP-CATP-03` / `PHASE-CATP-01`: 새 load/source inventory/trace/candidate clone/fuel의 실제 시간·할당·peak/retained/generation count와 비용을 source/toolchain/workload별 기록. Plain latency와 allocator timing을 분리. 기존 baseline/guards는 actual impact를 대조하여 재실행하거나 exact non-impact reuse 근거를 제시

새 durable fixture/consumer/actual performance 및 core regression은 WP03 소유다. Source 수정은 AG owner로, fresh 감사는 구현 종료 후 별도 reviewer로 분리한다. 기존 R01/R02의 실제 currentness를 재검토하며 과거 audit/실패/receipt는 보존한다.

## Known Unknown

KU30-01 exact types/code/fork/observer/control and engineering bounds remain AG owner inside CRA P01–07. KU30-02 actual test selectors and new cost results belong to WP03. KU30-03 affected predecessor currentness/audit must be restored before R03 completion. KU30-04 external host/deployment/personal-store compatibility N/A. Policy/acceptance/runtime meaning, stored lifetime/loss or scope changes return to owning P2G stage; no unbounded placeholders or future R acceptance declared complete.
