# R03 single-scope preparation — bounded source review 025

이 검토는 INPUTS_067의 private prepare/replacement 연결과 focused090의 실제 oracle만 다룬다. 완료 감사가 아니며 공개 API/전체 observation/모든 native bounds를 재조사하지 않았다. 소스·문서·Git 변경과 build/test 실행은 하지 않았고, 신규 /tmp 보고서만 작성했다.

**이 제한 source slice에서 구체적인 production 결함을 발견하지 못했다.** 023에서 제안한 단일 scope, 누적 receipt usage, 최종 work/ledger 채택과 B swap이 실제로 연결되었다. 아래 시험 한계는 코드 결함과 분리한다.

## 1. source·evidence 기준

초기 capture 2026-09-10T11:44:39.135235+00:00. INPUTS_067의 at_utc는 2026-09-10T11:41:52.293147+00:00, manifest는 249개 파일을 나열한다. 그 전체를 검토한 것이 아니라 아래 5개 관련 source entry와 읽은 파일 SHA의 일치를 확인했다. 기존 023 MD를 전체 다시 읽었고 SHA `8cea7da48fafed44f52e0be43a226bcadbb6da9b3957979afd569e4455ed6c4b`를 보존했다.

focused_090.json 원문은 root가 실행한 `cargo test -p game_core --lib -j 1 private_trial_ -- --nocapture`의 exit 0, selected/passed 4, failed/ignored 0, compile 44.19s, tests 3.29s를 기록한다. 출력의 네 test 이름은 현재 trial.rs의 네 tests와 일치한다. **helper의 독립 실행 결과가 아니다.** 진행 중인 default-release check019의 결과는 이 보고서가 검증하거나 주장하지 않는다.

## 2. source에서 확인한 경계

**Scope와 중간 candidate.** trial.rs 94–123의 prepare_owned는 remaining(0)을 검증하고 fresh provenance로 run_with_provenance를 한 번만 호출한다. 108–116에서 prefix_len을 값으로 보관해 initialize_in_scope 이후 prepare_step만 반복한다. advance나 cfg(test) initialize를 호출하지 않으므로 중첩 run 또는 단계별 fuel reset이 없다. 초기 accepted_work=0(291), 매 prepare_step candidate도 self.accepted_work 그대로(506)이며, 최종 outer/inner 성공 뒤 121–122에서 반환 ledger와 전체 usage.work를 한 번 대입한다. 준비 중 stale placeholder native_storage field는 TLS에 재설치되지 않는다. 실제 이력은 control.rs 154–289의 동일 Attempt에 남고, clone/내부 rollback과 함께 reset되지 않는다.

**완료·교체.** finish_preparation_in_scope(141–195)는 prefix 전체가 성공한 뒤 실제 Copy lifecycle의 is_ready와 elapsed ceiling을 검사한다. authoring_lifecycle.rs 57–66의 timezero+Running/Finished 조건을 재사용하므로 이전 단계가 ready였어도 최종 Exited면 실패하고 실제 Finished는 허용한다. raw Run snapshot이나 active 존재를 readiness 대체값으로 쓰지 않는다. replace_prepared(126–139)는 독립 B 생성 결과가 Ok일 때만 *self=candidate 한다. 이전 A를 중간 변경하는 문장이 없으며 inner Err는 prepare_owned의 candidate?에서 종료된다. 성공 후 반환은 저장된 ready receipt borrow뿐이다.

**최종 receipt와 work.** 초기/step/ready payload는 모두 usage_basis=scope_cumulative이고 preparation 중 prior_accepted_work=0이다(trial.rs 181/185, 266/273, 475/484). output.rs 68–72 주석은 intermediate cut과 마지막 scope usage를 명확히 구분하도록 수정되었다. fixed prefix 예약·payload·header fill charge 후 snapshot을 읽는 알고리즘(76–125)은 유지된다. 마지막 ready encoding 뒤에는 receipt String/Arc/byte scalar 이동만 있고 새 charged 작업이 없다. returned usage와 final header work를 더하는 식이나 intermediate 합산은 없다. 후속 advance는 기존 accepted_work+새 한-step usage만 더한다(324).

**소유·index·할당 순서.** preparation_receipt는 별도 Option<Arc<PreparedReceipt>>(80)이고 records에 append되지 않는다. ready owner는 최종 encode 전에 생성된다(161–166); unique Arc에 prepared String을 넣고 Some으로 이동하므로 get_mut expect의 전제가 깨지는 alias 경로는 없다. 후속 prepare_step은 448–449에서 해당 Option/Arc를 controlled_clone한 뒤 최종 step receipt를 encode한다. Option visitor는 outer record와 Some child를, Arc visitor는 handle만 계수한다(clone_shape.rs 107–114, 152–157). 기존 ready 문서 전체를 복제하지 않는다. 결과 slot Vec는 filter 없이 구성되고 host tick None/assert None도 index를 유지한다. records Vec는 admission 후 next_len capacity를 확보한 상태에서 최종 record를 push한다(426–444, 494).

**실패와 한계 계산.** remaining(35–64)은 원 execution 설정을 먼저 검증하고 positive residue로만 clip한다. max_total_work≤2^38, usage는 effective.max_work 이하(control.rs 473–485)이므로 후속 accepted_work 덧셈은 범위 안이다. cumulative receipt 계산은 이미 준비된 prefix 길이를 사용하며 max_receipt_bytes 검사 후 append하므로 중간 provisional receipt도 같은 상한에 포함된다. 제어 실패는 enclosing final acceptance 검사에서 sticky diagnostic으로 돌아오고, B의 모든 provisional state/receipt/quota는 local drop된다.

## 3. 신규 두 test의 실제 oracle

trial.rs 572–637의 preparation_document는 **test 안에서만** 일반 public API로 유한 index를 정해 명시 JSON을 작성한다. runtime runner에는 후보 검색·자동 VN/노드 선택 분기가 추가되지 않았다. 실제 TrialState는 해당 JSON을 해석해 start/VN/snapshot/starter binding/map binding/confirm/pause/HostTick/assert를 실행한다.

**첫 test(655–766):**

- 실제 preparation이 ready이고 records 9개, paused HostTick인 slot7과 assert slot8이 None임을 검사한다. copied_bytes>0으로 이번에는 실제 양수 pointer binding 복사를 통과한다.
- 초기→9개 step→ready의 prior work 0, scope_cumulative, strictly increasing cumulative work를 검사하고 accepted_work가 마지막 값과 같음을 확인한다.
- 준비 후 snapshot을 advance해 첫 execution record의 index9와 prior work/1회 합산을 검사한다. ready Arc pointer가 유지된다.
- 동일 content의 seed78 B를 독립적으로 준비해 측정한 전체 receipt bytes−1 및 work−1로 각각 replacement가 실패함을 검사한다. A lifecycle, 양수 copy quota, receipt bytes, work, record 수10, ready Arc를 비교한다.
- B 전체 work의 exact cap에서 재시도하고 독립 B와 ready 문서/lifecycle이 같으며 새 scenario seed78, records9임을 확인한다.

**둘째 test(768–856):** max_total_clock_ms=1인 상태를 준비한다. execution NextEventBucket의 실제 elapsed 초과는 TrialExecution phase, prep prefix에 같은 clock을 추가한 실패는 TrialPreparation phase임을 직접 검사한다. 실제 retreat를 prefix 끝에 추가하면 not-prepared, 빈 prefix도 not-prepared다. 앞서 만든 ready 상태에서 출발하므로 “중간 ready이면 조기 성공” 구현을 잡는 oracle다. 실패 이후 A lifecycle/work/records/ready Arc 보존을 검사한다.

## 4. oracle 한계 — production defect로 판정하지 않음

- **A 전체 graph 비교는 아니다.** replacement 실패 비교는 lifecycle/scalar quotas/record len/ready pointer 중심이다. A의 모든 public snapshot, token/content identity, initial input/seed, 각 기존 record 원문, 다음 command/RNG continuation을 직접 대조하지 않는다. code상 complete-swap 구조와 test가 관찰한 subset을 구분해야 한다.
- **바뀐 native provenance가 검증되지 않는다.** 이번 prefix에는 Monolith insertion/import가 없고 native_storage 자체를 비교하지 않는다. 실제 이력을 바꾸는 command 후 late B 실패/재시도에서 clone bound 또는 test-only ledger snapshot을 관찰하는 별도 oracle가 필요하다. 현재 소스에서 field/TLS reset 결함이 확인된 것은 아니다.
- **Late cancellation/즉시 terminal은 이 selector에 없다.** 신규 tests는 cancellation token을 취소하지 않는다. 기존 네 번째 test는 advance 전 이미 취소된 token만 다룬다. final ready encoding 후 acceptance cancellation, 즉시 Finished preparation 허용 branch는 현재 trial tests의 직접 검증 범위 밖이다.
- **Header 전체 counter 일치와 실패 site는 직접 비교하지 않는다.** 준비 test는 work의 단조성·최종 일치와 usage_basis를 검사하지만 final ExecutionUsage의 모든 peak를 외부 관찰값과 대조하지 않는다. work−1의 error_code helper(889–897)는 phase/site를 버린다. 최종 receipt byte/work cap 경계에 도달한다는 현재 source 순서는 확인했지만, 미래 회귀에서도 정확히 ready/header-fill 지점에서 실패했는지 test assertion만으로 구별하지 못한다.
- **Content generation 교체 시험은 아니다.** seed77→78은 같은 embedded validated token이다. 외부 A/B generation 변경과 full terminal/source/trace 증명으로 확대 해석하지 않는다. 해당 전체 미완료를 이번 좁은 구현의 신규 defect 목록으로 되풀이하지 않는다.

작은 보강 후보는 기존 실패 assertion에 phase/site와 initial/scenario identity·앞선 record pointer 비교를 추가하고, 실제 ledger mutation 및 ready 이후 cancellation/Finished 분기를 별도 bounded fixture로 검증하는 것이다. 새 공개 surface나 정책을 요구하는 결론이 아니다.

## 5. 정확한 source hash와 읽은 범위

- core_enhanced/src/authoring/trial.rs: `7518eb3a95979d2c8ff941fff4b60fb44abbd10704270cc2b753225d62b895e1`; /files/14; 1–1012.
- core_enhanced/src/authoring/output.rs: `2951433ffb08b52568028b70485cc12992c3c172a064e44df4795b1a22c71ddb`; /files/9; 31–130.
- core_enhanced/src/authoring/control.rs: `1a15e8bdd7618572950ddc69ad25b5cda674eebfecd0f544010abb3477b2fed8`; /files/6; 154–289, 472–487.
- core_enhanced/src/authoring/clone_shape.rs: `6e56d3990b14357c98663ace941fc44b6cde7c41667829b4dddec78e71adbc6b`; /files/4; 22–34, 105–115, 150–157.
- core_enhanced/src/game/world/authoring_lifecycle.rs: `69e06f862a961fe0ff1d9b1b35fbdb869d1db4e0cb4db69e7f71efdda552aa8c`; /files/175; 53–80.
- core_enhanced/docs/goals/content_authoring_tool_ports/execution/single_goal/evidence/INPUTS_067.json: `dc39d2badecef9fe3eaee2791fba46fff213bd64e4cca06968e1e9bcc49797c9`; evidence/prior design; JSON metadata/raw evidence; hash provenance.
- core_enhanced/docs/goals/content_authoring_tool_ports/execution/single_goal/evidence/focused_090.json: `5d9cb7e352f761460dbcfcb46e85bac31d6d5d32080d9767587523e1d9b225e4`; evidence/prior design; JSON metadata/raw evidence; hash provenance.
- /tmp/r03_single_scope_preparation_design_023.md: `8cea7da48fafed44f52e0be43a226bcadbb6da9b3957979afd569e4455ed6c4b`; evidence/prior design; 1–92.
- /tmp/r03_single_scope_preparation_design_023.json: `44f0a0b936c38a56c0f2a7724d0f60f739cf3b3c6522d3eb00e4c002d9f2cdfb`; evidence/prior design; JSON metadata/raw evidence; hash provenance.

이 보고서는 current private 연결에 대한 제한 검토이며, 공개 readiness나 전체 Goal 승인 결론을 내리지 않는다.

최종 rehash: 2026-09-10T11:48:57.631867+00:00; drift 0개. 후속 source 변경은 이 검토에 포함되지 않는다.
