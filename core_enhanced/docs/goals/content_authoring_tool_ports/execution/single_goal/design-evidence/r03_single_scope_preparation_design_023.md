# R03 single-scope preparation — bounded design 023

이 보고서는 INPUTS_063의 private TrialState를 다음 구현 단위에 연결하기 위한 기술 제안이다. 코드 구현·시험 결과·공개 readiness·전체 Goal 완료 감사가 아니다. 저장소/소스/문서/Git 변경 및 build/test 실행은 하지 않았다. Root의 동시 lifecycle 검증 결과를 이 보조의 시험 결과로 인수하지 않는다.

**권고:** 초기 생성의 in-scope 본문만 추출하고, 새 private prepare가 하나의 run_with_provenance 안에서 initial → 명시 preparation prefix → 실제 최종 readiness → 마지막 mandatory ready receipt를 실행한다. 외부에 publish하기 전 accepted_work는 scope-entry base 0으로 유지하며 반환된 전체 usage와 ledger를 딱 한 번 채택한다. 새 상태기계나 중첩 scope는 필요하지 않다.

## 1. 현재 source와 caller

- trial.rs 95–171 initialize는 fresh provenance run을 소유하고 초기 receipt 이후 usage.work/ledger를 채택한다. 현재 직접 caller는 같은 파일 453–468의 test helper initial이다. initialize는 아직 준비 완료를 주장하지 않는다.
- advance(174–205)는 remaining(self.accepted_work), run_with_provenance, prepare_step(208–383), outer/inner 성공 확인, complete swap 순이다. prepare_step의 현재 caller는 advance 하나다. 실제 command/snapshot/clock/assert branch(238–290)와 binding/result slot 구조를 그대로 재사용할 수 있다.
- CoreApplication::new_for_authoring(787–791)는 GameCore::new_for_authoring(world.rs 116–120)을 통해 fixed lifecycle owner를 켠다. fork_for_authoring(803–808)은 기존 complete opaque clone이다.
- world/combat.rs 683–690은 실제 active 설치·entry 생산·zero advance 성공·timezero 완료를 순서대로 기록한다. terminal은 1396–1410에서 active 제거 전에 witness를 보존한다. application draft는 world/application.rs 186–202에서 reconcile/poll 후 sidecar도 같이 commit한다.
- TrialLifecycle::is_ready(authoring_lifecycle.rs 57–66)는 time_zero_completed이고 Running 또는 Finished일 때만 참이다. Exited/EntryPending/None은 거짓이다. Finished라면 active가 없어도 성공할 수 있다. 마지막 reconciliation(190–226)은 이미 생성된 witness만 무효화하며 snapshot에서 entry를 만들어내지 않는다.

003 원문 /tmp 경로는 현재 없다. durable design-evidence/trial_port_design_003.md 전체를 읽었고 JSON의 원문 SHA와 일치했다. 이는 003 승인 전제를 뜻하지 않는다. 003의 borrowed-input/public raw step 제안은 현재 owned input/검증된 scenario cursor에 맞춰 바꾸고, §5–6의 단일 preparation scope·실제 timezero·독립 B swap 원칙을 재사용한다.

## 2. 최소 refactor

trial.rs 안에 다음 private helpers만 있으면 된다. 명칭은 제안이다.

1. **initialize_in_scope(content, scenario, limits) -> Result<TrialState, AuthoringError>.** 현재 initialize closure 본문(109–162)을 옮긴다. active scope 전제 검사와 entry poll을 하고, 기존 owner 생성·profile snapshot·initial receipt 순서를 유지한다. run, usage 채택, ledger reset을 이 helper 안에서 하지 않는다. 기존 initialize wrapper는 테스트의 “unprepared” 경계로 남길 수 있으나 새 prepare는 이를 호출하지 않는다. 현재 production caller가 없어 cfg(test)로 제한하는 것도 작다.
2. **prepare_owned(content, scenario, limits, cancellation) -> Result<Self, AuthoringError>.** 이미 검증된 owned 입력을 받는다. limits.remaining(0, TrialPreparation)을 먼저 실행하고 fresh ledger로 run을 한 번 연다. prefix 길이를 usize로 복사해 보관한다. initialize_in_scope 다음 `for _ in 0..prefix_len { candidate = candidate.prepare_step(TrialPreparation)?; }`를 실행한다. candidate에서 빌린 slice iterator를 유지한 채 candidate를 교체하지 않는다. advance 호출은 nested scope/중복 work 채택을 만들므로 쓰지 않는다.
3. **finish_preparation_in_scope(&mut self) -> Result<(), AuthoringError>.** 실제 lifecycle을 Copy로 읽고 missing/not-ready/actual elapsed ceiling을 검사한다. 새 ready receipt owner를 먼저 만들고 모든 fixed bookkeeping을 charge/admit한 다음 최종 encode_receipt를 호출한다. 준비된 receipt와 새 receipt_bytes를 local aggregate에 move한다. 이 함수 뒤에는 계수·할당·JSON parse·fallible projection을 두지 않는다.
4. outer run의 최종 acceptance가 성공하면 inner candidate?를 확인한 뒤 returned ledger를 대입하고 accepted_work=usage.work로 설정해 반환한다. 중간 candidate.native_storage의 초기 placeholder를 TLS에 재설치하지 않는다. preparation 중 실제 ledger는 동일 TLS Attempt에 단조 누적되고 최종 반환 때만 authoritative field가 된다.

현재 prepare_step이 단계마다 opaque fork와 immutable prefix 복사를 유지해도 된다. 이미 사용 중인 per-step staging/control을 제거하는 최적화는 이 연결에 필요하지 않다. 실행 중인 scope의 fuel은 gameplay clone/내부 rollback과 독립이므로 각각의 성공 단계 뒤에도 reset되지 않는다.

## 3. receipt와 work 의미

현재 output.rs 68–69의 “모든 encode_receipt 뒤 추가 charged work 금지” 주석과 PLAN 274, 489–490의 “각 header는 final attempt usage”는 intermediate encoding에 그대로 적용할 수 없다. **최소 수정은 encoder 알고리즘이 아니라 scope 기준을 명시하는 것**이다.

- 각 header의 usage는 **같은 outer attempt 시작부터 해당 receipt encoding/header fill까지 누적된 counters**다. 마지막 ready receipt의 usage만 준비 scope 반환 usage와 완전히 같다.
- prior_accepted_work는 **그 outer attempt 전에 외부에 확정된 session work**다. 준비 중 초기/각 prefix/ready receipt 모두 0이다. local provisional step을 실행했다는 이유로 값을 올리지 않는다.
- 준비 완료 후 accepted_work = ready.header.usage.work. 이후 interactive advance에서는 기존 방식처럼 old accepted_work + 새 한-step scope usage.work를 한 번만 더한다.
- initial/preparation/final header work를 합하지 않는다. usage peak들도 합이나 차로 재구성하지 않는다. 필요하다면 scope 내 구간 work만 연속 누적 work의 차로 계산할 수 있지만 이번 연결에 별도 delta field는 필요 없다.
- 이미 완료된 content validation usage는 initial payload의 별도 provenance다. 이번 already-validated 입력 경로가 validation 비용을 같은 scope에 포함했다고 표시하거나 다시 합산하지 않는다.

예: initial header=10, prefix headers=30/70, ready header=90이라면 B accepted_work는 **90**이다. 200이 아니다. 다음 step의 prior_accepted_work=90, 그 scope header=25이면 accepted_work=115다. 10/30/70/90은 설명용 숫자이고 실제 측정값이 아니다.

기존 fixed-width header reservation은 그대로 작동한다. 중간 receipt 뒤 추가 work가 있어도 이전 header는 그 cut 시점의 불변 snapshot이다. payload에 고정 `usage_basis: "scope_cumulative"`를 붙이고 ready `kind: "preparation_ready"`로 최종 경계를 명시하는 방식이 작다. 공개되지 않은 private schema지만 의미 변경을 PLAN에 기록해야 한다. 마지막 ready receipt의 동일성 검사로 기존 자기참조 방지 계약을 유지한다.

## 4. ready receipt의 소유권·index

ready receipt는 scenario step이 아니므로 records에 넣지 않는다. records.len()을 binding/step cursor로 쓰는 현재 구조에서 별도 None record를 넣어도 다음 execution step이 밀린다.

최소 필드 후보는 `preparation_receipt: Option<Arc<PreparedReceipt>>`다. initialize_in_scope는 None, 준비 마무리만 Some을 만든다. 기존 initialize test wrapper를 유지할 때도 “unprepared”가 명확하다. 후속 prepare_step candidate를 만들 때 이 Option/Arc handle만 controlled_clone으로 계수·복사하고 full receipt를 재직렬화하지 않는다. 이 clone은 기존 final step encoding보다 앞에 둔다. 별도 public receipt getter/추상 builder는 지금 필요 없다.

final payload에는 기존 source identity/input hash, prep step count, 실제 fixed lifecycle, prior_accepted_work와 usage_basis를 담으면 된다. 초기/profile/각 요청/result는 이미 각 receipt가 소유하므로 다시 전체 prefix를 embed하지 않는다. 최종 ready owner는 encode 전에 할당하고 Arc unique owner에 prepared String을 move한다. max_receipt_bytes는 initial+모든 preparation receipt+ready의 실제 누적값에 적용한다. encoding 중 사용한 committed_bytes는 scope 안에서 이미 준비된 prefix byte count여도 계산상 유효하다; 이름/문서에서 외부 publish와 혼동하지 않는다.

## 5. readiness와 B 교체

명시 prefix를 **끝까지** 실행한 뒤 is_ready를 검사한다. 중간에 ready였다는 이유로 조기 종료하지 않는다. 예를 들어 battle에 들어갔다가 명시 retreat/result acknowledgement로 최종 Exited가 되면 실패해야 한다. 반대로 timezero에서 실제 Finished가 된 경우는 현 is_ready가 허용하므로 InBattle snapshot/active 존재를 추가 조건으로 요구하지 않는다. 빈 prefix나 StartNewGame까지만 한 prefix는 실제 entry가 없으므로 실패한다. scenario.steps에 battle 시작이 있어도 preparation_steps를 자동 연장하지 않는다.

현재 witness는 terminal verdict/time/ordinal 등 fixed 사실만 보존한다. 이를 사용한 private readiness 연결은 가능하지만 full terminal source capsule/semantic occurrence/optional trace가 완료되었다는 뜻은 아니다. 공개 surface는 그 남은 의무와 분리해 계속 보류한다.

B replacement는 private `replace_prepared(&mut self, owned_B_inputs, cancellation)`에서 `let b = Self::prepare_owned(...)?; *self = b;` 순이면 충분하다. B는 새 default-memory app/fresh ledger/새 seed·input·receipt·work family이며 A의 잔여 work나 mutable counters를 쓰지 않는다. 실패한 B의 intermediate receipt/ledger/quota는 B와 함께 drop하고 A에 붙이지 않는다. inner Err를 outer Ok로 성공 처리하지 않는다. final poll 뒤 swap 이후에는 fallible 재직렬화나 두 번째 cancellation 판단을 두지 않는다. 반환 receipt 참조가 필요하면 이미 저장된 ready owner를 빌린다. A의 더 이상 소유되지 않은 Arc/native graph는 평소 drop 규칙을 따른다.

## 6. 필요한 좁은 검증과 남은 한계

아직 실행하지 않은 시험 후보다.

- 실제 public preparation prefix의 Running 성공 및 실제 timezero terminal Finished 성공. prefix 후반이 실패하거나 Exited인 경우에는 earlier-ready와 무관하게 B 실패.
- final ready header 전체 usage == outer 반환 usage; preparation prior_accepted_work 전부 0; 다음 step의 base는 준비 전체 work; cumulative headers를 합하지 않는 oracle.
- 전체 preparation의 exact work/one-under. 개별 단계에는 충분하지만 합에는 부족한 fuel로 실패해 reset/nested 호출을 잡는다. 양수 session remainder clip도 후속 advance에서 확인한다.
- final ready 문서만 한 바이트 부족한 cumulative receipt limit 및 ready 이후 acceptance cancellation: A app/token/input/seed/ledger/records/copy/receipt/work 유지. 별도 실패 후 동일 valid B 재시도.
- prep records의 None slot과 양수 binding copy quota 유지; ready를 records에 넣지 않아 첫 execution index가 prep_len인지 직접 확인.
- 변경된 provenance를 실제 만드는 prep command와 late output 실패를 포함한다. 기존 zero-copy/변경 없는 ledger fixture만으로 aggregate 보존을 완료했다고 표시하지 않는다.

PT020에서 지적한 기존 public projection의 최초 owned JSON allocation과 전체 observer 미완료는 이 refactor가 닫지 않는다. 반대로 그 전체 과제를 이번 단일-scope 설계의 새 코드 결함으로 세지 않는다. 현재 실제 caller/owner에 붙일 수 있는 조건부 기술안이며 새로운 gameplay·콘텐츠 자동 선택 정책은 제안하지 않았다.

## 7. provenance

주 capture 2026-09-10T11:28:17.647378+00:00, direct producer 추가 capture 2026-09-10T11:29:56.233028+00:00. 초기 discovery(2026-09-10T11:27:47.653987+00:00)의 기존 9개 파일 hash는 주 capture와 같았다. 아래 source는 읽은 범위만 사용했다. INPUTS_063에 있는 읽은 source는 주/추가 capture 시 모두 일치했다. 마지막 hash/drift는 JSON과 아래 종료 기록에 보존한다.

- core_enhanced/src/authoring/trial.rs: `a636a0269b568beed306542d77d19ac5d23ab3bf2578ba1731e494e8a40db4d5`; /files/14; 1–402, 423–476.
- core_enhanced/src/authoring/control.rs: `1a15e8bdd7618572950ddc69ad25b5cda674eebfecd0f544010abb3477b2fed8`; /files/6; 150–253, 410–423.
- core_enhanced/src/authoring/output.rs: `d56c244833316a3049409cdd85bc8a203748f79d57b15788ebe484ed2b588e5e`; /files/9; 65–128.
- core_enhanced/src/authoring/scenario.rs: `0152fd9a1f33ad5d061102da34b5bfc13f8e282b697f4e0030dcb741320da859`; /files/12; 230–268.
- core_enhanced/src/application.rs: `c37e28bfccbed675dc268a59b477ddc3041759d02ba9e6e94019c8a9506a84e8`; /files/3; 783–824.
- core_enhanced/src/game/world/authoring_lifecycle.rs: `69e06f862a961fe0ff1d9b1b35fbdb869d1db4e0cb4db69e7f71efdda552aa8c`; /files/175; 1–227.
- core_enhanced/docs/goals/content_authoring_tool_ports/P2G_50_GOAL_SET_001.ko.md: `bb0b69cfebceb17fd13374826e1dbb6ba6e38e711cb574acc29ca61ca2745f98`; document/manifest; 54–72, 94–104.
- core_enhanced/docs/goals/content_authoring_tool_ports/execution/single_goal/PLAN.md: `a61c4b0de51ec77913643737366029487a17b810ac548dda86652e7187ea5cb9`; document/manifest; 45–87, 265–275, 475–507.
- core_enhanced/docs/goals/content_authoring_tool_ports/execution/single_goal/evidence/INPUTS_063.json: `c2f2ffd3450443a362529ae784e066f9881db19db288d0bef43fcf98027b4235`; document/manifest; JSON metadata/selected entries.
- core_enhanced/docs/goals/content_authoring_tool_ports/execution/single_goal/design-evidence/trial_port_design_003.md: `3cc58bce2565c6e14e03ac7d4987caffcacc9ebec5224b4eb44cae6505308162`; document/manifest; 1–163.
- core_enhanced/docs/goals/content_authoring_tool_ports/execution/single_goal/design-evidence/trial_port_design_003.json: `d8e60be01396bfb3cea17e0873f66ee6ebdbc4698d2e7a494bbd4eb1df1dc333`; document/manifest; JSON metadata/selected entries.
- core_enhanced/src/game/world.rs: `57d47fb5fdb7a728d9b7e1ba08b6efb0a7b56de0c28380f59b413999a662bd7f`; /files/165; 104–132.
- core_enhanced/src/game/world/application.rs: `5b1a67a4550f31f03e7baeb1db45d8293147978e90d0b4b53df99f934481b2f0`; /files/174; 163–202.
- core_enhanced/src/game/world/combat.rs: `b78e0de0ab027b3f99ffbd8c078df6ea8fff5fd1559992ed600a4a3579ce0062`; /files/177; 664–697, 1382–1420.

종료 rehash: 2026-09-10T11:33:56.186157+00:00; drift 2개. Drift source의 successor 내용은 이 보고서의 설계 근거로 인수하지 않았다.
- core_enhanced/docs/goals/content_authoring_tool_ports/execution/single_goal/PLAN.md: a61c4b0de51ec77913643737366029487a17b810ac548dda86652e7187ea5cb9 → 5d1170a476ddf5d9436c56e245d07406f0548b9a2c8b376cf947962d9834f360
- core_enhanced/src/game/world.rs: 57d47fb5fdb7a728d9b7e1ba08b6efb0a7b56de0c28380f59b413999a662bd7f → 6dc90b38c0d4aa09a692b757665d45ff47fce274aa47df955eb23e8d8fff2de7
