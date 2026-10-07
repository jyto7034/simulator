# U08 전체 독립 감사 — u08_current_012 / pass001

판정: **Not Complete / Confidence High / open findings 2**. `Complete/High/open0`를 부여하지 않는다. F01은 실제 소비 경로의 검증 공백이며 제품 오동작을 재현했다는 뜻이 아니다. F02는 감사 도중 발생한 canonical 문서 변경으로 인한 현재성 불일치이며 제품 source438의 변경이 아니다. 수정·재실행·후속 target 동결 뒤 독립 재감사가 필요하다.

감사자는 이 Goal의 계획·설계·제품 구현·시험 작성에 참여하지 않은 신규 독립 담당자다. 제품·성능 시험을 실행하지 않았으며, 기존 source/증거/target/문서를 수정하지 않았다. 이 디렉터리의 독립 hash/archive/복원/원시자료 비교 도구와 결과만 작성했다. 기존 담당자에게 받은 위치 안내는 검색 보조로만 사용하고 해당 source/raw를 직접 확인했다.

## 대상과 현재성

- Target: `execution/single_goal/target/u08_current_012/MANIFEST.json`, 336,317 bytes, SHA-256 `1f00555f55860c5701f9353606a1726bb8b93424be61011dd2c99b50fab0cf1d`.
- Product: 438 files, SHA-256 `4c7bbd317a9a12045267bd306057e771133096ccbccace6a51f484d23a46e2ac`.
- Config: `evidence/finalizer_ready012/CONFIG_READY_012.json`, 12,443,830 bytes, SHA-256 `d2072042ad80b8ed6790681d3e21ba6a0e653cf1fb2989f1ed5de2773c03ecfa`.
- Entry: `ENTRY_IDENTITY.json`의 2026-10-07T05:11:21Z 검사에서 현재 고유 파일6,902개와 target 내부4개 archive의 membership/bytes가 일치했다. source before577→after617, records6,294, references118이다.
- 독립 격리 복원: 원 before archive에 forward patch를 적용하여 after617개와 정확히 일치하고, reverse patch로 before577개에 정확히 복귀했다. 외부 U07M archive4개도 실제395/405/2,829/1,228 members와 hash가 일치했다. workspace에 reverse를 적용하지 않았다. `RESTORE_INDEPENDENT.json` 참조.
- Exit: product438와 target/archive 자체는 보존되지만 `core_enhanced/docs/README.md`의 현재 bytes가 frozen source와 달라졌다. F02 및 `EXIT_IDENTITY.json`, `ADDITIONAL_INDEPENDENT.json`, `README_CURRENTNESS.json` 참조. 따라서 target012를 현재 전체 working tree의 exact target이라고 승인할 수 없다.

## F01 — 자동 스킬과 대기 시전의 유물 수치 실제 소비 검증이 빠짐

**분류:** 중간 심각도의 완료 차단 검증 공백. **요구:** frozen PER07, PE-CRNV-07.2, IVA-CRNV-V07, P2G_10_POLICY_001.json:900/918, P2G_50_GOAL_SET.ko.md:79. frozen 정책은 기본·자동·명령 시전, queued cast와 지연 소비에서 개인 계획의 일관된 실제 사용을 요구한다.

**위치와 관측:**

- `core_enhanced/src/game/world/tests/u08_carriers.rs:1280`의 두 직원144/210 시험은 `cast():506`을 통해 BeginSkillTargeting→ActivateSkill을 실행한다. `assert_skill_hit():557`의 실제 HP·copy/effect/source 검증은 유효하다.
- 같은 파일 `basic_auto_and_authored_extra_melee_projectile_use_final_amount_once_and_keep_raw_attack():1736`은 Auto/Triggered **기본공격**이다. `assert_basic_hit():1700`은 basic profile과 fragment 없음까지 구분한다. 이것을 자동 **스킬** 시험으로 셀 수 없다.
- `core_enhanced/tests/skill_runtime_contract.rs:1004`의 `explicit_cast_targeting_separates_cast_context_from_step_execution_targets`는 실제 AutoCastStart→AbilityCast 인과를 검사하지만 abnormality fixture이며 유물 소유자/기여/수치 기대값이 없다.
- `core_enhanced/src/game/world/tests/combat.rs:9674`의 `activate_skill_uses_equipped_manual_fragment_in_live_defense`는 PauseBattle→접수→Resume→AbilityCast를 검사하지만 유물 setup/기여/피해 기대값이 없다.
- 현재 U08 projectile/fixed-area/follow-area 시험은 명령 시전으로 방출된 지연 payload와 미방출 취소를 실증한다. 이는 유효한 지연 queue 증거지만 정지 상태에 접수된 시전의 release 이후 유물 수치 소비를 직접 검증하지 않는다.
- 위 selector는 `evidence/integrated_core_default007.log`의 1962/1959/2123/1630행에 실제 PASS이며 fixtures/observation에서도 실행됐다. 문제는 미실행/가짜 PASS가 아니라 시험들이 서로 다른 속성을 각각 검증하고 있다는 점이다.
- 제품 `core_enhanced/src/game/battle/core/sim.rs:4420` AutoCastStart와 `:4670` 부근 AutoCastEnd는 별도의 접수·target·action/source 경로를 거친 뒤 공통 invocation/capture에 합류한다. 정적 검토상 공통 relic capture 사용과 모순은 찾지 않았지만, 명령 시전 성공만으로 이 경로의 실제 수치·출처 결과가 입증되지는 않는다.

**위험:** 자동 스킬 또는 정지 중 접수된 시전의 source/provider/actor 결속이 잘못되어도 현재의 U08 manual/basic 수치시험과 generic 자동/정지 시험이 모두 통과할 수 있다. 이 상태에서 모든 실제 cast consumer를 검증했다고 판단하면 frozen 수용 범위를 축소한다.

**최소 교정·폐쇄 기준:**

1. 기존 지원되는 정상 등록/획득/장착/전투 경로로 유물 소유 직원의 자동 스킬을 실제 발생시킨다. 실제 AutoCastStart/AutoCastEnd 또는 동등한 권위 경로를 확인하고, 고정된 수작업 기대 피해량과 실제 HP 감소, 정확한 actor/copy/effect 및 원 fragment/catalog/action 출처를 함께 assert한다. 서로 다른 유물 구성의 같은 SkillId 두 직원으로 개인 격리를 확인한다. 기존144/210 또는180/210 oracle를 재사용할 수 있으나 제품 evaluator 출력으로 기대값을 계산하지 않는다.
2. 같은 지원 계약에서 PauseBattle 중 명령 시전을 접수하고 정지 중 시간/피해가 진행되지 않음을 확인한 뒤 Resume하여 실제 유물 수치 피해와 정확한 출처를 assert한다. 기존 delayed release/withdraw 시험은 그대로 재사용하며 별도로 모든 지연 조합을 다시 만들 필요는 없다.
3. 권한 있는 작성자가 시험을 보강하고 지정 실행자가 새 source identity에 결속된 실제 nonzero 결과를 만든다. 바뀐 source/receipt/coverage를 반영한 후속 target을 동결한다. 정책 또는 제품 지원 범위의 변경이 필요하다면 숨은 fallback이나 임의 정책 확대로 대신하지 말고 별도 근거로 반환한다.

## F02 — 감사 중 canonical source 현재성이 달라짐

**분류:** 중간 심각도의 완료 차단 target-currentness 불일치. **요구:** PE-CRNV-16.2와 전체 target의 entry/exit exact 현재성.

`core_enhanced/docs/README.md:55`에 시설 노드 구성 검토 문서의 색인 행이 추가되었다. frozen012는17,979 bytes / `4b9c4baaf3cd61a886d50eea108cb52cd3def521390973649628a322bbe63c36`, exit 현재 파일은18,356 bytes / `d9036609aaa919bec9991e0531b437a3a82df6fe19538bf07543d2ae73f0f407`이다. filesystem mtime은2026-10-07T05:11:37.926318Z로 entry 검사 뒤다. 누가 변경했는지는 이 감사에서 확정하지 않았다. diff는 `README_CURRENTNESS.diff`에 보존했다.

현재 확인한 변화는 다른 시설 기획 문서의 색인 추가이며 유물 제품 source438/수치/실행입력을 바꾸지 않는다. 따라서 제품 회귀나 과거 실행 무효화로 확대하지 않는다. 그러나 이 파일이 frozen source617/MEMBERSHIP에 포함되어 있으므로 현재성 성공으로 숨길 수도 없다.

**최소 교정·폐쇄 기준:** 다른 작업을 되돌리지 않고 현재 index 변경의 범위를 인수한다. 후속 target에 올바른 현재 source/canonical membership와 restore archive를 포함하고 entry/exit를 다시 검증한다. target012와 본 감사 원본은 보존한다. 이 문서 변경만을 이유로 제품·성능을 재실행할 필요는 없으며 F01에 따른 source/test 변경 영향은 별도 판단한다.

## 전체 검토 범위와 나머지 근거

단순 최종화 PASS나 마지막 compile.rs 수정만 검토하지 않았다. frozen16군/32문장,10차원,3노드,16seam과191 선행 조건(187 affected/4 reviewed_nonimpact), 별도 R06 31행의 현재 claim→selector/raw/assertion 결속을 확인했다. `CONDITION_REVIEW.json`은 해당 전체 집합과 F01/F02의 파급을 기록한다. 광범위한 기존 시험의 모든 줄을 새 시험처럼 재검증했다는 주장은 하지 않으며, 변경된 authority/수치/소유/저장 경계와 주요 독립 oracle를 중심으로 정적 검토했다.

| 범위 | 직접 확인한 핵심 근거·판정 |
| --- | --- |
| 01–03 사본/슬롯/이전 | UUID별 사본과 immutable 정의 분리,3슬롯·선검증 이전·variant와 무관한 대칭 비호환, 잘못된 슬롯 graph의 checksum-valid Continue 거절. 실제 lethal hit→terminal settlement→중복 acknowledgment에서 슬롯만 회수하고 Normal/Corrupt copy 보존. 추가 finding 없음. |
| 04 획득/분쇄/판매 | 정상 reward/trial grant, 판매 특정 사유 거절, 취소 무변경, dust overflow 이전 refusal, 분쇄 후 UUID stream owner 유지. 추가 finding 없음. |
| 05–06 조건/합성 | same-effect tag AND와 정확 equipment/fragment/form, immutable provider, checked i128 flat합→percent합→한 번 반올림/narrow.6개 slot permutation 실제 피해/패널티, Physical/Magic mitigation와 base attack 불변. 수작업180/210과 별도 실제 HP 사례 확인. |
| 07–08 실제 소비/출처 | manual literal/Expr, 관측off 두 직원, basic Auto/Triggered, released Snapshot projectile/fixed area, follow 취소, Live missing-context atomic failure, catalog/generation/action/provider guards와 private binding 보존 확인. 자동스킬/대기시전은 F01. |
| 09–11 침식/원자성 | authored Corrupt 효과·negative penalty 실제 적용,1촉매 유료/0촉매 Event 분리, exact scene/node/history authority, 한번만 변환·재시도 idempotence, late output admission 실패의 전체 roots rollback. |
| 12–13 복원/수명 | current/checkpoint/entry/Continue root 동시 전환, generation mismatch·forged graph typed failure, 실제 detach/attach same actor pause와 resume, fresh process/retreat/reentry/redeploy.3회 전투의 Weak owner/capture 회수는 해당 owner 수명의 근거이며 전체 프로세스 heap가0이라는 주장은 아님. |
| 14 공개 작성 | 새 ID RON 정상 loader→grant→Event/decline→실제 Normal180/Corrupt210 HP 감소→save/fresh process. public prospective query는 authority를 발행하지 않으며 같은 효과/provider와 수치 evaluator를 사용한다. |
| 15 예산/호환 | 실제 cap one-short/exact admission, cumulative root seed와 rollback, empty legacy owner 무추가 계량, 관측 세 모드 suite와 별도 allocator/PC 자료. 아래 R05 warning 유지. |
| 16 전체 증거/동결 |58개 실제 실행과 독립 복원은 확인. F01로 전체 consumer coverage가 닫히지 않았고 F02로 exit currentness가 달라져 전체 독립 완료 불가. |

선행 조건의 source delta102개와 compiler 교정 영향,4개 비영향 문서의 exact source 근거를 검토했다. R06의 단순 PASS 목록에만 의존하지 않고 mixed Resource A/B의 static/dynamic Sum raw diagnostic이 publication 전에 `value_expr_unit_mismatch`/compile/expected ResourceA/actual ResourceB를 반환하며 같은-resource positive2건이 실제 validated됨을 확인했다. probe003의 고정 oracle·입력·실행 코드를 읽고 실제 stdout/stderr와 summary를 독립 비교했다. 이전 probe009 runner argument 실패는 성공으로 세지 않았다.

## 실행·성능 증거 판독

`RECEIPTS_INDEPENDENT.json`은58역할의 exact command/raw hash/동일 source-before-after/종료값,628 claim-selector 결속,249 assertion과426 artifact pointer를 독립 검사했다. Core default/fixtures1967·1967, observation1986; Host94·105·124; heap1·1, PC1의 실제 결과를 확인했다.57역할 exit0, R05만101이다. 문서상의 observed 텍스트가 test oracle의 독립성을 대신하지 않으므로 앞 절의 source assertion도 함께 검토했다.

Composite007/008/focus005/009/010의 역할17/7/1/24/9와1958 union은 단순 최초 관측으로 취급하지 않는다. 원 before/after와 role별 시간창·dependency·flat archive·gap의 유지된 bytes가 근거다. 해당 validator의 legacy/composite 구분, 각 role 자체의 timely coverage와 공통 파일 불변 조건을 읽었다. PC/R05 projection의50 receipt는 원 nested receipt와 정확히 같고 모든 기능 종료 뒤 원 관측 시각, 이어서 측정 시작이라는 순서를 독립 계산했다.

PC576행을 immediate U07M 원raw와 별도 재귀 비교했다. 비시간 변화는 native retained/owner extent와 refusal requested의 **정확히200 bytes**뿐이며 work/clone/allocator count/bytes/결과/오류 code/site/cap은 같다. 이는 정적152-byte independent owner와48-byte placement option의 계량 변화이며 비용 증가 없음이라는 의미가 아니다. 기존17 negative controls와31성공5typed-refusal을 확인했다. `ADDITIONAL_INDEPENDENT.json` 참조.

R05는 동시 peak **9,683,689 > 원 cap9,030,188**으로653,501 bytes 초과하고 raw101이다. 직전 U07M9,669,160보다14,529 bytes 증가했다. 다른 hard guard 실패는 없고 retained4,196,928은 cap와 같으며 operations370,928은389,240 이하이다. frozen 범위가 이 기존 peak-only warning의 보존을 명시하므로 별도 신규 completion blocker로 중복 생성하지 않는다. 전체 성능 PASS, cap 인상 승인, 증가 없음으로 해석하지 않는다. shared-host latency는 gate 불가이고 native admitted extent/allocator live/요청량/RSS는 서로 다른 관측이다.

Native fault의 역사 observation-on 실행파일 SHA 미보존(null)은 그대로 한계다. 새로운 off mode와5mode aggregate는 source/input/build/raw 범위에서 검토했으며 존재하지 않는 옛 binary hash를 추정하지 않았다. WebSocket007 no-fault 실패는 보존됐고 successor005는 정상 RON의 delayed single wave3actors,8800ms 진행 뒤 실제 HostTick work1148>불변cap1000, sticky fault·거절·reconnect·replacement·quit/Continue를 검사한다. driver는 fault를 강제하는 admin/API 우회나 cap 완화를 추가하지 않았다.20분 stress는 frozen 범위 밖이다.

최신6개 정책 파일의 dealer-first VN/첫게임 뒤 gambit/무제한 양수 반복/무이자 부분 담보/나가기 완료, stress 기본XP1회·무억제 공통 보상은 현재 roadmap U16 및 U13/U14/U26 후속 책임과 일치한다. frozen U08 사본3슬롯/수치/once corruption/atomic 계약을 바꾸지 않는다. 과거4개 bytes가 archive+recorded diff로 hash 검증 재구성된 것과2개 current-only review인 한계도 유지한다. 이 정책 delta만으로 U08 P2G 전체를 임의 재개방하거나 후속 활동 구현 완료를 인정하지 않는다.

## 후속 인수

F01/F02를 작성·실행·동결 담당자에게 반환한다. 감사자가 제품이나 시험을 직접 고치지 않았다. F01은 새 실제 자동·대기시전 증거와 source 영향 검토가 필요하고, F02는 현재 canonical index를 인수한 successor target이 필요하다. 기존 요구/한도/경고를 보존한 후속 전체 target에 대해 다시 독립 판단하며 U09 진입을 승인하지 않는다.

감사 보조 도구의 초기 복원 `-p1` 대신 올바른 `-p2` 적용 및 multiline test PASS parser 조정은 감사 도구 자체의 준비 오류였다. 그 실패는 제품/원시 실행 실패로 세지 않으며 기존 evidence를 수정하지 않았다. 최종 독립 검사 결과와 F02의 실제 불일치를 분리하여 기록했다.
