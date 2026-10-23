# GOAL-CRSL-01 실행

<!-- U05 CURRENT STATUS BEGIN -->

## 현재 상태

P2G-00–60과 GEX-00 인수를 마쳤고, 현재 GEX-10 통합 자체 검증 중이다. Core 기본1758/fixture1758/관측1767, 호스트 기본91, 공개 기능·재시작·되감기·다섯 fault 모드가 현재 소스에서 통과했다. 호스트 관측·최종 정적/heap 검사와 성능 측정, 최종 증거 연결·정역 복원·독립 감사는 아직 남아 있다. U05 독립 판정은 없으며 Complete/High 전 U06 종속 구현을 시작하지 않는다.

<!-- U05 CURRENT STATUS END -->

## 시점별 계획·판단 기록

아래의 준비·미완료·다음 단계 표현은 해당 기록 시점의 상태다. 최신 실행 상태는 위 요약과 최종 실행 증거가 소유한다. 정책·Goal의 동결 원본은 각 P2G artifact를 참조한다.

GEX revision2026-09-13-gex-r7; P2G current r7/quality2. Core-first U05/B02,B03,B04,B09.
Mode goal-run: 원 연속 요청 ../../P2G_00_INITIAL_001.json IN-001, CF-D06/07 위임.
명시적 독립 감사·finding 교정·다음 owner 전환까지 승인됨. 중간 단계검토 opt-in 없음.
Exact member GS-CRSL-001/GOAL-CRSL-01; PHASE-CRSL-01;
AG-CRSL-RESONANCE + WP-CRSL-VERIFY. Accepted HA-CRSL-001/FPF-CRSL-001/PP-CRSL-001/PVR-CRSL-001.
Exact refs/digests는 ../../P2G_60_HANDOFF_001.json과 P2G40/50/60 result가 소유한다.

### GEX-00 intake checkpoint

GEX-00 complete; GEX-10 active (상세 설계 및 atomic 구현). U01/U04 independent CompleteHigh를 인수했다.
현재 member에 기존 PLAN/EXPERIMENT/AUDIT가 없는 신규 실행임을 좁힌 경로 검색과 생성 이력으로 확인.
이 디렉터리 하나가 실행 문서 owner다. PLAN=current scope/basis/design/progress,
EXPERIMENTS=실제 시도·검증, EXPERIMENT_NOTES=선택·실패/후속 위치.
독립 감사는 구현 후 별도 post_audit 디렉터리에서 작성한다. U05 감사 verdict는 아직 없다.

baseline SOURCE_BEFORE archive/index와 ARCHIVE는 실제 dirty before bytes를 보존하고 추출 hash까지 확인했다.
기존 사용자 변경·선행 구현·과거 감사는 reset/stash/일괄정리하지 않는다. GEX00 인수 시 150 reviewed source unchanged를 확인했다.
초기 검증은 ../../evidence/PREDECESSOR_BASELINE_001.json의 U04 final367 product unchanged 근거를 재사용:
Core1734/1734/1743,host89/97+heap1,fmt/check/event-log/clippy/release,16public consumer/PC576와
actual continuation/rewind/fault 증거. U05 변경 후 검증으로 주장하지 않는다.

## Implementation and readiness

정책 원본은 Freeze14요소, 구조/actual/검증 seam은 ../../CODE_REVIEW_002.ko.md RQ01–07 TD01–11 V01–14.
DEFINITION→ADMISSION→LIFETIME→PROJECTION→VERIFY를 root가 순차 구현한다.
네 구현 WP는 최소 atomic 전환이며 중간 코드를 독립 완료로 게시하지 않는다.
직원 base20%와 유효 InitialSkillCharge/배치 효과, 파편별max와 적 autonomous 별도 owner를 유지한다.
입력 최신검증→소비/자격→기존 start/corrosion timing→actual-use 종료→all-positive gain→query/restore를
한 owner에 연결한다. effects settlement·resource scope와 use lock을 합치지 않는다.

첫 product mutation 전에 구체 descriptor/compiled future-control 조건, credential state key와 consume fact,
finite buff/channel 실제 lease, pulse scheduling/CC overlap, cancellation/released payload와 extent/clone hooks,
strict authoring/public projection의 타입·함수·순서·시험 기대를 이 PLAN에 추가하고 준비 점검한다.
공개 source/host/local fixture와 Rust toolchain/cache가 제공되며 operational store/배포 mutation은 필요 없다.
필수 policy/source gap 없음. exact 내부 실현 선택은 TD01–11 안의 GEX 책임이다.

## Validation and completion

V01–14와10 required dimensions를 actual public RON→normal command/query 및 kernel boundary로 검증한다.
공식25개+Flow/sustained 공개 fixture 전종, slow projectile/combo/timedbuff/channel/independent 생성,
Pause input/0잔량start/중복/취소무환불/stale preview/empty tile/accepted anchor/corrosion actual timing,
양수gate/기본gain제거/RestHP/초기충전효과/적주기, clone rollback·same actor reconnect·U04 Continue가 포함된다.
선행127조건+별도31R06의 changed/reuse/focused map, full Core/host/native/observer/strict/public release
검증을 완료한다. 실패/zero/skip은 보존하고 필요한 수정·재실행만 수행한다. R05 approved peak exception과
20분 제외를 유지하되 새 실패를 숨기지 않는다. 실행 후 source 변경은 해당 evidence를 invalid 처리한다.

완료 후 exact source/data/docs/evidence target과 before/after fuzz0 forward/reverse 복원을 고정한다.
변경없는 선행 증거는 exact external archive/member/digest로 인수해 중복 아카이브를 피한다.
제품 mutation을 멈춘 뒤 구현 비참여 GPT-6 Astra/High가 독립 GEX20을 수행한다.
finding을 교정·재감사하여 CompleteHigh 전에는 U06 종속 구현을 시작하지 않는다.
Single Phase이므로 Master/GEX30 없음. 사용자 stop/승인 로드맵 완료/현 권한·환경으로 해결불가 실제 장애에서만 멈춘다.

## GEX-10 detailed design / mutation readiness (2026-10-04)

Scope/authority/upstream은 위 인수와 동일하다. 보존한 374 before files와 reviewed150을 기준으로
현재 타입·함수 call chain을 재확인했다. 아래는 구현 전 결정이며 시험 통과 주장과 구분한다.

1. DEFINITION: `SkillFragmentForm.resonance_max: u32`를 필수 양수로 작성하고 실제 selected form을
   `SkillFragmentLoadout::apply_to_profile`에서 해석한다. base floor(max/5), gain_lock0을 만든 뒤
   기존 consumable ceil bonus/cap을 적용한다. 직원 persistent `EmployeeCombatProfile.battle_profile`에는
   별도 strict serde authoring projection을 두어 옛 resonance(start/max/gain_lock)를 거절한다.
   공통 runtime `UnitCombatProfile`/`ResonanceDef`는 적 autonomous와 이미 합성된 값의 운반용으로
   유지한다. 직원 base의 비기본 공명 주입도 validate해 조용히 무시하지 않는다. persistent reader
   version/legacy 원본보호를 이 schema 변경과 함께 맞춘다. Rest의 공명 field/call은 제거하고 HP는 유지한다.
2. `SkillDef.use_lifetime`/RawSkillDef에 필수 explicit descriptor를 둔다: `Action` 또는
   `ActionAndEffects { effects: Vec<SkillUseEffectRef> }`; ref는 stable step/effect ID 쌍이다.
   목록은 유일·존재·finite buff/sustained 의무인지를 catalog validation에서 확인한다. DOT/permanent
   및 독립 area/summon은 Action이다. 혼합은 해당 finite 의무만 열거한다. 공식 데이터와 공개 fixtures,
   catalog/clone/retained shape를 동시에 전환한다. Rust builder도 같은 validator를 통과한다.
3. ADMISSION: player Employee source만 새 domain이다. `ResonanceUse`를 RuntimeUnit에 보관해
   action handle/skill/consumed amount/control pending을 묶는다. 기존 transaction 안의 최신검증 후
   accept→전량0+resource fact/log→use credential→pending/event 순서다. pending0은 credential로
   start를 통과하며 그 뒤 Silence/full 재검사는 하지 않는다. HardCC/actor unavailable/필수 target 상실은
   취소한다. coroutine가 아닌 기존 SkillStartGate 시간 경계를 유지한다. corrosion은 접수 당시 비용을
   재소비하지 않고 자격과 현재 source/trauma를 확인해 기존 start payload를 실행한다. 적 automatic과
   resource-free boss explicit은 기존 비용 owner를 유지한다.
4. LIFETIME: use는 pending cast control과 현재 cast의 미래 미방출 action, selected finite effect 의무를
   합친다. FlowState에 recursive remaining-control 판정을 둔다: Action은 dispatched 후 false;
   Sequence/Parallel은 남은 children, 미선택 Branch는 미래 obligation, Repeat는 남은 iteration,
   Wait는 아직 끝나지 않은 실제 동작 대기를 반영한다. 마지막 leaf의 EffectsSettled 대기만으로 잠그지
   않는다. legacy step/ExtraAttack의 예약·release까지 포함한다. descriptor로 선택한 아직 미도달
   finite 효과는 delivery가 끝날 때까지 유지하고 실제 적용 후 buff/sustained record의 source action으로
   lease를 추적한다. refresh/replacement/early removal/no-op/CC channel end에 실제 record 상태를 사용한다.
   취소 시 미래 control만 닫고 released payload와 적용된 timed buffs는 각각 자기 수명을 유지한다.
   필요 없는 use는 즉시 제거해 과거 action을 무한 보유하지 않는다.
5. GAIN: 배치된 player Employee에 다음 정수 battle-second pulse를 기존 queue로 예약한다.
   매 pulse +1을 공통 양수 gate로 넣고 다음 pulse만 예약한다. Pause에는 clock이 멈추고 CC/use/cap에서
   버린 양을 적립하지 않는다. spawn/withdraw/death stale event는 실제 instance lifecycle로 거절한다.
   기본공격 release/dealt/received gain은 employee만 제외한다. `add_resonance`와 readiness는 같은
   hardCC/use/legacy autonomous lock 판정 owner를 공유하며 중첩 사유를 각각 투영한다.
6. TARGET: final PlayerTile는 tile geometry/reach만 요구하며 unit/auto usefulness는 유지한다.
   acceptance의 tile anchor를 계속 전달하고 직접 필수 unit은 cast/step 직전에 최신 관계·활성·air 검증한다.
   area anchor는 원래 unit의 이동/사망을 이유로 바꾸거나 취소하지 않는다. preview는 immutable이고 실패
   transaction은 focus/revision/resource를 함께 보존한다. manual retarget는 validation에서 거절한다.
7. PROJECTION/RESTORE: authoritative phase와 gain-lock reasons를 readiness DTO에 추가하고 host
   기존 serialize/query를 통과시킨다. use owner의 controlled clone/extent/canonical을 보완한다.
   private transaction clone·paused same actor reconnect에서 동일 credential을 검증하고 durable Continue는
   기존 pre-entry/confirmed 경계를 검증한다. 새 live-battle disk codec은 만들지 않는다.

준비 판정: 정책14/TD01–11/V01–14와10 dimensions에 owner가 있고 unresolved policy gap 없음.
필수 initial baseline은 U04 exact product367 evidence로 충족; before374 archive는 추출 hash 검증 완료.
실행 도구 run.py가 각 명령의 source before/after와 raw stdout/exit를 보존한다. first edit 후 old baseline은
새 의미의 통과 근거가 아니며 focused lifecycle matrix→전체 Core/host/native/observer/strict/release public
검증으로 갱신한다. 변경중 WP는 미완료이며 publication/audit 전 전체 atomic 경로를 완성한다.


### Early implementation checkpoint

First definition/admission/lifetime/projection pass is implemented but the atomic Goal is incomplete.
`runtime_focus_004` passes8 direct cases at its exact source; `core_lib_interim_001` is the first broad regression
attempt. It already reveals inherited tests/fixtures requiring semantic or schema correction; no completion claim.
Strict descriptor completeness, saved version transition/original protection, the full lifecycle matrix, current
canonical docs and required full Core/host/public checks remain before target freeze or independent audit.

Strict descriptor realization: all finite beneficial Sustained effects (Modifiers/ChannelAura/Overlay/Shield/
Survival/Parry) and ShieldExpr require an explicit unique reference. Hostile Control and legacy ApplyBuff
(CC/DoT), permanent stats and independent children cannot be listed as finite beneficial use obligations.
Persistent TileArea producing a finite beneficial use effect is explicitly unsupported at publication: it could
reapply a lease after the creator use has ended. This is TD06's permitted precise rejection of unsupported
combinations, not a fallback to settlement locking. Mixed ordinary independent areas and separately authored
finite effects remain supported. Current fixtures are migrated through the same public validation.

### Integrated preparation checkpoint

Strict descriptors, employee base/schema separation, explicit v7/v8 conversion and version11/current-save protection are implemented. Focused actual-use20 tests pass at runtime_focus_015, while final-source coverage remains open. The complete Core intermediate run passed1693/1694; the lone descriptor-negative fixture is corrected and requires revalidation. The awakened butterfly form now uses explicit fragment_funeral_awakened_eulogy so the autonomous original retains RetargetOnStep. Current catalog has26 skills; the frozen policy's25 denotes the intake set, all of which are still covered.

Canonical target/runtime/rulebook/authoring/data and host protocol now describe employee admission consumption, actual finite leases, positive gain gates, same-actor pending use and current durable version11 versus checkpoint-conversion version12. These are implementation checkpoints, not a final validation or audit claim.

CF-D08 문서 권위 교정 인수: DOCUMENT_AUTHORITY_INTAKE.json. 외부 사용자 문서 교정은 보존하며 game_rulebook.md를 새 정책/구현의 동기화 대상으로 쓰지 않는다. 최신 next·combat 정책은 그대로이고 runtime/data/host 계약 및 현재 실행으로 구현을 검증한다. 기존 P2G의 과거 룰북 갱신 locator는 이 인수 기록으로 현재 적용 범위만 교정하고 동결 원문을 소급 변경하지 않는다. 독립 감사는 해당 delta의 실제 비영향을 별도로 판단한다.

GEX-10 자체 검증 종료: GEX_10_RESULT.json, COVERAGE.json, PREDECESSOR_IMPACT.json에 정확한 source와 실행 증거를 결속했다. 독립 감사 판정은 아직 없다. target/u05_001의 실제 before/after와 정역 복원을 고정해 별도 GPT-6 Astra / High GEX-20에 인계한다. U06는 독립 Complete/High 이후에만 실행한다.
