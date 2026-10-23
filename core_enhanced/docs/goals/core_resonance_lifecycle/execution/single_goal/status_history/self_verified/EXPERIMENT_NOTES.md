# U05 기술 판단과 실패

<!-- U05 CURRENT STATUS BEGIN -->

## 현재 상태

P2G-00–60과 GEX-00 인수를 마쳤고, 현재 GEX-10 통합 자체 검증 중이다. Core 기본1758/fixture1758/관측1767, 호스트 기본91, 공개 기능·재시작·되감기·다섯 fault 모드가 현재 소스에서 통과했다. 호스트 관측·최종 정적/heap 검사와 성능 측정, 최종 증거 연결·정역 복원·독립 감사는 아직 남아 있다. U05 독립 판정은 없으며 Complete/High 전 U06 종속 구현을 시작하지 않는다.

<!-- U05 CURRENT STATUS END -->

## 시점별 계획·판단 기록

아래의 준비·미완료·다음 단계 표현은 해당 기록 시점의 상태다. 최신 실행 상태는 위 요약과 최종 실행 증거가 소유한다. 정책·Goal의 동결 원본은 각 P2G artifact를 참조한다.

P2G20 initial candidate001은 base 초기량+InitialSkillCharge 및 직원/적 autonomous 적용 범위 발견으로 abort했다. P2G10 successor에서3요소 보완/11exact 보존 후 complete review002를 고정했다. 원본 보존.
현재 지원 snapshot은 인메모리 candidate/같은 actor reconnect 및 U04 confirmed disk state다. 새 live battle disk 저장을 추가하지 않는다.
구체 구현 선택은 PLAN에 준비 점검과 함께 추가한다.


Current implementation is an incomplete atomic GEX-10 pass. Eight focused runtime cases pass; no final audit target.
RuntimeUnit holds one bounded ResonanceUse (action identity/skill/consumed, pending applications/extra attacks,
control cancellation); FlowState computes remaining authored actions separately from delivery settlement.
Actual buff/sustained source leases decide selected finite-effect obligations. Step delivery accounting subtracts
actual sustained children before deciding whether an unarrived selected application remains. Employee gain
and DTO reasons share ResonanceGainBlocks. Ended use state can be reclaimed at the next gain/pulse/admission;
projection and admission use current obligations immediately. This bounded reclamation refines PLAN's immediate
storage cleanup without postponing observable unlock. No timestamp-only post-use employee lock is installed.

Still required: complete strict descriptor semantics/typed fixture migration, base/max/consumable and removed-field
acceptance, employee disk schema version and explicit legacy conversion/original protection, lifecycle edge matrix
(Repeat/Branch/Parallel, early lease removal/refresh/channel, accepted area anchor, target/actor loss, overflow/budgets),
full query/host/clone/reconnect/Continue tests and docs. Then predecessor127+R06 impact map, full required checks,
source-bound exact target/restore archives, independent Astra High audit and corrections. U06 is not eligible yet.

Saved schema transition implemented: current continuation11, explicit historical checkpoint conversion output12
(distinct shape, not automatically continued). Host envelope remains5 and now requires Core11. Explicit V7/V8
converters verify original checksum/content identity first, accept only the historical employee default0/100/1000,
remove that field and produce employee base20/100/0. Unknown/custom values reject rather than lose meaning.
Original historical bytes remain untouched. Current writer/reader excludes the employee field and validates base.
Pulse deadline and use obligation counter overflows now latch typed CounterOverflow; selected-effect reference
scans charge both authoring/native work. Cancellation prunes unstarted step progress, preserving released leases.
Full native rollback tests for these new boundaries remain pending. Strict descriptor migration remains pending.

Descriptor strictness now implemented and103 regression cases passed. Hostile legacy ApplyBuff/Control are
independent; only beneficial Sustained/ShieldExpr can own use leases. Therefore removed the first-pass unused
pending ApplyBuff application counter/hooks and legacy buff use scan, rather than retain unreachable ownership.
Selected unarrived finite deliveries remain tracked by step progress; actual Sustained records own applied life.
Descriptor scans and nested sustained-child scans charge native work. Employee death/withdraw clears own
credential/next pulse and employee pending death cancels immediately; nonemployee pending death keeps its old
cast-end cost/failure path. Existing target-movement area test now uses an Employee source to exercise the new gate.
Current canonical runtime/data/skill-target/rulebook/authoring docs have an initial U05 update; remaining stale
wording and complete validation/projection/API examples must be reconciled before freeze.

Current U05 additions: selected finite effect descriptors are required; explicit unsupported persistent-area finite beneficial production is rejected. Independent areas/DoT/permanent/hostile Control do not extend employee use. Official awakened butterfly skill was split to preserve its opponent original. Twenty focused cases passed015; the host reconnect test is still being corrected at its public test seam (host-tick rather than direct simulation advance; aggregate resource_spent includes deployment cost, so the final oracle must measure the admission delta). Final full suites/public consumers/coverage/audit remain pending.


Independent-area correction: retained action attribution is not enough to claim use ownership. reserve_use_attack now verifies the exact authored skill/step and excludes persistent TileArea producers, with both authoring and native scan fuel. This preserves area attacks and their own carrier lifetimes while preventing the initial or later tick from reopening the creator use. The counterexample was found by root source review and reproduced through current public release CLI; it was not inferred from a green unit suite.

The skill-authoring canonical document was absent from the initial before manifest. AUTHORING_MODEL_BEFORE supplement explicitly reconstructs its exact pre-U05 v0.4 from archived v0.3+historical fuzz0 patch, and verifies that only the two U05 edits differ. The record labels reconstruction timing honestly; final forward/reverse restoration must include it.

Final canonical cross-read corrected earlier runtime-contract wording about pending-only cancellation, cast-end Cost and direct-target no-op: employee accepted-use now owns those boundaries; autonomous source semantics remain explicit. This documentation edit is outside the immutable product execution identity and will be included in SOURCE_AFTER and the reversible patch.

Condition-map completeness review: inherited U04 implementation rows for21 IVA and12 F retained selectors/group names but no requirement text. The new U05 map now reads exact original D09 V01–21 and the R07 continuous request §5 F01–12, links completed R06 scope expectations and31 acceptance mappings, and freezes all four reference files. Current-specific differences are explicit for admission/corrosion, future control, finite leases, employee charge, schema/legacy protection and cost accounting. Original U04 rows/audits are unchanged; no prior verdict is inferred from these map additions.

Selector-binding preflight found one missed rename inside current_v10_checkpoint_continue_then_v10_capture_is_semantically_identical. The mapping now explicitly replaces both version components with11; all inherited current selector bindings match actual pass lines. No product assertion or test was changed.

PC 비교 종료: 잘못된 입력의96 after-log 차이는 원래 비직원 초기값 명시로 해소했고, 원래 gameplay 비교는 변경 없이 통과했다. 새64bytes/unit·clone traversal 증가 때문에 비용 완전 동일 비교는 실패 원본으로 남기며, 값 전체를 제외하지 않고 source 근거의 정량 delta만 설명한 별도 비교가 통과했다. 독립 감사는 이 적응·비용 설명의 적용성을 판단해야 한다. CF-D08 문서 권위 교정을 현재 hash로 인수했으며 과거 룰북을 최신 정책으로 취급하지 않는다.
