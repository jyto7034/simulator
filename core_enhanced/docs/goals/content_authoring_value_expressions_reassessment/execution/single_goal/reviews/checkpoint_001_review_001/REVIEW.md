# R06 checkpoint 001 단계검토

검토일: 2026-09-13 · 판정: **수정 필요** · Findings: 4건 (P1 1건, P2 3건).

대상: `R06_STAGE_REVIEW_TARGET_001`, manifest SHA-256 `e62085bbcbdb8cfe9108f640c7d61859a0e12c29aae8171fa62d64b98474bfe1`.
Goal: `GS-CAVER-P2G50-001 / GOAL-CAVER-01`. 이번 결과는 부분 커널 검토이며 GEX-20/30, WP/Phase/Goal 완료 판정이 아니다.

## 범위·권한·근거

사용자의 “아니다 너가 검토해줘” 요청으로 읽기 전용 부분 검토를 수행했다. 이 reviewer는 앞서 공통 단계검토 지침 정리·R06 설명/현황 확인을 했으나 checkpoint 001의 커널 구현·구현 테스트 작성에는 참여하지 않았다. 이번 추가 probes는 source target 밖의 진단 증거이며 production 테스트를 수정한 것이 아니다. 별도 사람/agent가 검토한 것으로 표시하지 않는다.

검토 입력은 `CHECKPOINT_REVIEW_HANDOFF.md`, 고정 MANIFEST와 initial baseline, PLAN/EXPERIMENTS/EXPERIMENT_NOTES, 확정 Goal의 applicable 조건과 CRA D01/D03/D04/D08/D09, GEX gex-r7의 단계검토·전후 고정 경계다. 신규 커널 4파일과 기존 2파일의 변경을 읽고 원본/결과 archive의 bytes 및 exact 변경 집합을 확인했다.

RON parser/catalog·전투 Snapshot/Live frame·자원 lease/result prefix·Attack/SpellPower·save 전환이 미구현인 사실 자체는 finding으로 세지 않았다. 이번 인계에서 명시적으로 제외한 후속 범위다. 다만 현재 제공하는 커널의 예산/컴파일 자격 검사가 그 자체로 성립하는지는 검토 대상이다.

## F01 — [P1] 집계 전처리·소유자 payload가 work/retained 제한을 우회함

위치: [eval.rs:397](../../../../../../../src/game/value_expr/eval.rs:397), 같은 파일 403–436, 198–209, 212–214, 688–714.

`members()`는 전체 입력을 deep clone하고 정렬·중복 비교·canonical buffer 생성을 끝낸 뒤에야 `charge_members(canonical.len())`를 호출한다. 중복 제거로 실제 평가 대상 수가 줄어도 그 전처리 비용은 이미 발생했다. 이후 group key에 보유하는 owner 문자열, 복제한 member/values map과 임시 vector 등도 retained 계산에서 빠지고, group별 값에는 `size_of::<TypedInteger>() + member.identity.len()`만 부과한다.

공개 커널 API로 다음 두 사례를 재현했다.

- 16,384개 동일 resolved member, meter `max_work=32`, `max_dynamic_members=1`: `Ok(1)`, 보고 usage는 work 3/member 1/retained 210 bytes지만 입력 준비 이후 evaluate 중 추가 live-heap peak가 **4,735,168 bytes**였다. 정규화된 target을 한 번 세는 의미는 유지할 수 있으나 원시 입력의 순회·정렬·복사가 무료여서는 안 된다.
- member 1개, owner 문자열 131,072 bytes, retained 한도 1,024 bytes: compile과 evaluate가 성공했다. evaluate는 retained 210 bytes로 보고했지만 추가 live-heap peak는 **264,250 bytes**였다. plan/input은 측정 전에 만들어 두었다. compile도 같은 입력에서 추가 peak 395,555 bytes를 보였다.

이는 RSS나 공식 성능 벤치마크가 아니다. 직렬 실행한 reviewer 전용 global allocator가 측정한 요청 layout 기반 live allocation 증가다. allocator 메타데이터를 포함하지 않더라도 한도와 크게 어긋나며, 현재 성공 경로에서 예산이 실제 retained payload를 제한하지 못한다.

수정 방향: 입력 복사/정렬 전에 raw cardinality·payload·전처리 work를 admission하고, 중복 정규화에도 작업량을 부과한다. owner/member/reference payload, 임시 buffers·group/trace/value 보유량을 포함해 할당 전에 charge하거나, 검증된 불변 bounded 참조를 사용해 복사를 제거한다. 전처리 반복의 work/cancel 점검도 유지한다. adapter가 나중에 입력을 줄일 것이라는 가정만으로 현재 bounded kernel 주장을 닫지 않는다.

재검증: unique/duplicate/large-payload 입력 각각의 limit−1/limit/limit+1, 입력 크기와 무관하게 조기 거부되는 경계, canonical 의미와 정상 집계·취소의 회귀를 추가한다.

## F02 — [P2] Add의 node admission 전에 전체 자식 작업을 큐에 적재함

위치: [compile.rs:460](../../../../../../../src/game/value_expr/compile.rs:460), node 검사는 392–417.

`ValueExpr::Add(values)`는 남은 node budget과 자식 수를 대조하지 않고 모든 자식에 대한 `Task::Enter`를 push한다. 각 자식을 나중에 pop할 때 node 초과를 발견하므로, 입력은 거부되더라도 먼저 입력 폭에 비례하는 메모리를 할당한다. 명시적 반복형 compiler라는 사실만으로 작업 큐가 bounded하지는 않다.

재현: 이미 만들어 둔 65,536개 상수 자식의 Add를 `max_nodes=4`로 compile. 최종 결과는 `BudgetExceeded`이지만, 그 전에 추가 live-heap peak **7,340,826 bytes**를 관찰했다. probe에는 `max_retained_bytes=1024`도 지정했다. 정확히 1,024 bytes가 모든 compile 오버헤드의 허용량인지와 별개로, 4-node admission 실패를 처리하면서 전체 65,536-child task 큐를 만드는 문제가 확인된다.

수정 방향: 즉시 자식 수의 최소 node 비용을 남은 budget과 대조해 선거부하거나, sibling iterator/index를 갖는 bounded task frame으로 한 자식씩 진행한다. 중첩 작업·compiler working buffer도 승인된 상한 안에 두고 사후 검사만으로 보호됐다고 주장하지 않는다.

재검증: 작은 max_nodes와 매우 넓은 Add, 중첩 폭, 정상 node/depth 경계에서 early rejection 및 bounded queue를 확인한다. 아직 공개하지 않은 RON parser 자체의 문제로 분류하지 않는다. 현재 in-memory compiler에서 재현된 문제다.

## F03 — [P2] 더 작은 runtime max_stack이 실제 실행에 적용되지 않음

위치: [eval.rs:50](../../../../../../../src/game/value_expr/eval.rs:50), 값 push 경로 110–112, 164–166. 비교 대상: `CompiledAmountPlan::cost().max_stack`.

입구 검사는 meter 한도가 plan 한도보다 커지는 경우만 거부한다. 더 작은 meter는 허용하지만 실제 required stack과 비교하거나 push 시 검사하지 않는다.

재현: `Add(Read(Source.Attack), Const(1))`의 compiled max_stack은 2. Attack=41, runtime meter.max_stack=1로 evaluate하면 `BudgetExceeded`가 아니라 **Ok(42)**다. 현재 API가 허용하는 더 엄격한 caller 예산을 지키지 못한다.

수정 방향: 실행 전 actual compiled stack requirement를 유효 meter 상한과 비교하고, nested segment에서 동시에 살아 있는 stack/task가 무엇을 뜻하는지도 일관되게 정의·계산한다. max_depth/max_nodes처럼 meter에 함께 노출한 한도의 compile-only/runtime 역할도 명확히 하고 무시되는 설정을 두지 않는다.

재검증: 같은 plan을 작은/동일/큰 runtime meter로 평가하는 경계와 nested aggregates를 포함한다. 이번 확정 재현은 max_stack에 관한 것이며 모든 한도의 위반을 실행 검증했다는 뜻은 아니다.

## F04 — [P2] TargetEmployee 자원 참조가 대상 시점 capability 검사를 빠져나감

위치: [compile.rs:884](../../../../../../../src/game/value_expr/compile.rs:884). 인접 861–882는 Target Stat/HP에만 target-at-moment 검사를 수행한다.

`ValueRef::Resource`에는 `resources` 여부만 확인한다. owner가 `TargetEmployee`여도 Snapshot이면 `target_at_release`, Live이면 `target_at_consume`가 필요한데 이 조건을 대조하지 않는다. CRA D03의 target HP/stat/resource 행은 release에 실제 대상이 확인돼야 Snapshot이 가능하다고 정한다.

재현: `moment=Snapshot`, `resources=true`, `target_at_release=false`인 capability에서 `Resource(mana, TargetEmployee, Balance)`를 compile하면 **성공**한다. 동일한 미확정 대상의 Stat/HP는 거부하는 것과 비대칭이다.

수정 방향: resource owner가 대상 문맥을 요구하면 선택한 moment의 target capability도 검사한다. 이는 실제 lease/source-copy binding과는 별도의 정적 자격 검사이며 후속 runtime adapter의 책임으로 전부 미루지 않는다.

재검증: SourceEmployee/SourceEquipment/TargetEmployee × Snapshot/Live × resources/target capability 조합, 양성 대상 자원 읽기와 기존 target stat 거부를 대조한다. 이번 재현은 Snapshot이며 Live의 같은 누락은 코드 대조로 확인했다.

## 실제 검증과 보존

- 최초 source/record/evidence 해시 재검사에서 불일치 없음. 이후 [전체 currentness](TARGET_CURRENTNESS_ENTRY.json)는 참조 해시 398회(source 311, 실행 기록 7, evidence 51 및 archive/상태/중복 참조 포함), before archive 307/after archive 311개 내부 원본까지 일치함을 확인했다. 4 added/2 modified/0 deleted도 exact-match다.
- `cargo build -p game_core --lib --offline --locked --message-format=json`: 성공, 실제 현재 source를 사용한 lib artifact로 probes를 링크했다. 기존 경고는 성공과 구분한다.
- `cargo test -p game_core game::value_expr::tests --lib --offline --locked`: [기존 11건을 실제 재실행해 모두 통과](ORIGINAL_UNIT_RERUN.log).
- 기존 회귀 47건과 all-target/fmt/Clippy 기록은 고정된 command/output hash를 확인하여 재사용했다. 이번에 그 전체를 다시 실행한 것으로 세지 않는다. 두 baseline lint allow와 원래 Clippy/fmt 실패 기록도 보존했다.
- [별도 probes](review_probes.rs): **7건 중 2 passed / 5 failed**, exit 101. 실패 5개가 위 4개 원인에 대응한다. [실제 출력](REVIEW_PROBES.log). 성공한 두 control은 동적 나눗셈의 양/음 분자·분모와 4 rounding mode(총 3,096 조합), source-file 변경과 EffectSite 변경에 대한 fingerprint 검사다. 전체 i128 영역의 완전한 증명은 아니다.
- [검토 종료 currentness](TARGET_CURRENTNESS_EXIT.json)에서 원래 target·source·실행 기록·evidence 불변을 재확인한다. 이번 report/probes/log는 frozen target 밖의 이 review 디렉터리에만 작성했다.

재현 방법: repository root에서 위 cargo build 후 `rustc --edition=2021 --test <이 디렉터리>/review_probes.rs --extern game_core=/mnt/f/work/simulator/target/debug/libgame_core.rlib -L dependency=/mnt/f/work/simulator/target/debug/deps -o <새 임시 경로>/review_probes`로 빌드하고 `review_probes --test-threads=1 --nocapture`로 실행한다. 메모리 계측은 직렬 실행이 필요하다. 원래 실행 파일·작업 트리·원본 evidence에 덮어쓰지 않는다.

## 다음 경계

이 checkpoint는 검토 가능한 부분 구현으로 잘 분리했고 원본 보존·결과 기록도 확인된다. 다만 bounded compiler/evaluator와 capability admission이라는 현재 완료 주장에 위 결함이 있어 부분 PASS를 주지 않는다. 기존 11건 테스트 통과만으로 이 경계를 승인할 수 없다.

다음 구현 구간에 앞서 **사용자가 별도로 요청한 교정 → 새 snapshot/검증 → 재검토**가 필요하다. 코드 수정·새 target 발행·다음 구현·GEX-20/30은 수행하지 않았으며 이 검토 결과만으로 자동 승인되지 않는다.
