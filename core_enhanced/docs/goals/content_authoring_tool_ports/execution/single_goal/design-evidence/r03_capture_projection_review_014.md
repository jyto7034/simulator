# R03 capture projection 읽기 검토014

AUTORUN §8의 bounded source 검토이며 완료 감사가 아니다. INPUT054라고 전달된 시점의 실제 파일을 별도 해시로 고정해 읽었다. Root의 focused068 2pass/world320 실행 결과를 재실행하거나 독립 검증하지 않았다. Repository/source/docs/Git 변경, cargo/build/tests 실행은 하지 않았다.

결과: **completed candidate native storage 모델에서 fresh 고정 field가 누락되는 1건**을 확인했다. 아래 normal/set source mapping, scope, rollback 검토에서는 추가 concrete 오류를 찾지 못했다. 이는 전체 whole-clone/Goal 승인 결론이 아니다.

Capture: 2026-09-10 10:18:33.239700 UTC; direct consumers/tests 추가 10:19:16.103708 UTC; provenance source 추가 10:21:24.435168 UTC; close 10:22:21.653252 UTC. 파일 11개 재해시 drift 0. 이후 root 수정은 이 보고서에 반영하지 않았다. 전체 source SHA-256와 실제 읽은 span은 JSON에 있다.

## CP014-F01 — fresh fixed storage 미계수

심각도: medium, 명시된 completed-candidate native-storage 모델의 계수 오류.

주 위치: authoring/clone_shape.rs:226–251, SHA-256
`09e3e6d28bb427d64868fde4d2b96d44b17e911b956b04eeca9f62341497942c`.

실제 사용처: world/state.rs:168–195,622–625, SHA-256
`aa6492e023ef52b368dff21587903f093c7b7e34bcc5a23daeeef1b50549231f`.

macro는 :240에 outer record 1을 계수하고 :241–242에서 normal/set만 방문한다. :248의 fresh 값은 actual candidate에 들어가지만 meter에 전혀 들어가지 않는다. “source에서 복사한 동적 storage만 가격”이라는 더 좁은 모델이라면 별도 해석이 가능하지만, root가 이번 검토에서 확인한 모델은 완성된 candidate의 native storage이다. 그 기준에서는 구체적으로 다음만큼 작다.

- BattleEmployeeAttemptState::new는 새 processed_incapacitations HashSet를 갖지만(:625) empty set의 native record 1을 계수하지 않는다. 같은 완성값을 기존 struct_shape_with_sets(:1977–1980)로 방문하면 empty set에 1을 더한다(clone_shape.rs:141–145).
- clone_non_battle_transaction_state의 fresh PlayerProfileState::default와 active_battle:None(:191–194)이 빠진다. 현 native profile visitor(encounter_compendium.rs:31–38)는 default profile에 outer1 + compendium outer1 + empty BTreeSet1 + scalar4 = 7을 더한다. Option::None record 1까지 합쳐 **8 slots** 차이다.
- 원본 profile의 212,992-byte String을 방문할 필요는 없다. 누락은 새 고정 기본값의 storage이며 원본 dynamic profile의 복사 비용이 아니다.

영향: 보고된 capture peak U를 cap으로 허용해도 실제 완성값에 기존 native shape를 적용하면 U+1 또는 U+8이 된다. 작은 exact boundary에서 서로 다른 의미의 slot 수가 같은 max_clone_slots/clone_slots_peak 이름으로 사용된다.

최소 수정 후보: entry poll 이후 fresh 값을 정확히 한 번 생성하고, 그 borrowed 고정 shape를 meter에 더한 뒤 Self로 move한다. 빈 HashSet은 blanket CloneShape를 만들지 말고 별도 fresh_sets 목록에서 fresh capacity envelope+outer record를 더한다. 현재 fresh 값은 default-empty/None뿐이며 source-dependent allocation/clone을 이 목록에 허용하지 않는다. normal/set 목록의 단일 binding → 방문 → Clone 공유는 그대로 유지한다.

Root는 위 수정과 독립 완성값 shape oracle를 추가하겠다고 회신했다. 이는 **예정 disposition**이며 이 보고서의 captured source에서는 수정 미확인이다.

## 비용·borrow·실패 경계 검토

capture_shape의 source expression은 let에 한 번만 binding된다(:234–235). 검토한 다섯 사용처의 값은 모두 기존 소유 field에 대한 borrow 또는 LifecycleContext::phase()의 borrow다. entry poll 전에 field Clone/JSON 변환/큰 transient allocation을 만드는 expression을 찾지 못했다.

normal/set의 같은 binding을 admission과 actual Clone에 모두 사용한다. candidate의 complete struct literal은 새 field 추가를 빠뜨리면 compile error가 나도록 한다. 각 현재 field mapping을 실제 소유 타입과 대조했다.

- nonbattle projection: state.rs:172–190의 copied roots와 :191–194의 의도된 fresh roots가 GameCoreState 필드(:142–165)를 채운다.
- checkpoint: macro field 목록(:388–433)에서 run fields와 economy/UUID, checkpoint-consumed set을 대조했다. set이 clone 시 normal fields 뒤로 이동하나 key/ID/RNG 생성이나 재삽입은 하지 않는다.
- BattleEntrySnapshot: :541–576의 전체 field가 대응하며 game_state는 기존 phase의 native clone이다.
- employee attempt: :615–625의 roster clone + fresh empty set이다.
- EconomyGrantState: grant_state.rs:20–37의 다섯 mutable roots가 각 source와 일치한다. commit_into(:40–45)는 그대로 move한다.

active scope에서 meter.finish까지 성공해야 :245–249의 actual Clone이 실행되며, post poll 실패는 local candidate를 반환하지 않고 drop한다. capture가 immutable source만 빌리므로 이 실패가 그 source의 UUID/economy를 직접 바꾸는 경로는 없다. no-scope에서는 원래 field clone과 fixed constructors를 실행하며 Result는 Ok로 감싼다. “일반 모드의 객체 생성/Clone 의미가 같다”는 범위이며 메서드 반환 타입의 소스 호환성 전체를 감사한 것은 아니다.

## Rollback과 provenance

with_state_transaction(node_flow.rs:784–803)은 capture 실패 시 operation 전에 반환한다. operation이 Err이면 live profile을 mem::take하고 before state를 복원한 뒤 profile을 직접 move한다. fresh default profile에 대한 merge와 달리 큰 profile allocation을 추가하지 않는다. 확인한 monotonic profile 흐름에서 before/during 지식이 잃어지는 경로를 찾지 못했다. 이 함수는 **operation이 반환한 Err**를 기준으로 복원하며, 모든 내부 swallowed sticky failure를 여기서 자동 rollback한다는 더 강한 보장은 하지 않는다. 바깥 application/authoring acceptance는 여전히 필요하다.

mastered_elite_sweep.rs:168–173은 pristine profile을 별도로 controlled_clone한 뒤 nonbattle projection을 만든다. :325–328은 pristine state/profile을 함께 복원하고 pending plan을 설치한다. cfg(test) active_run rollback wrapper(:170–215)는 새 Result를 전파하고 기존 live-profile merge를 유지한다. 이 보조 범위는 다른 미전환 ordinary Clone 부위의 완료 판정이 아니다.

control::run은 NativeStorageProvenance::default(None)를 사용하고(:196–209), fresh()만 Some(zero)를 만든다(:160–170). capture macro는 새 run/scope를 열거나 ledger를 field로 clone하지 않는다. Monolith visitor(monolith.rs:104–136)는 같은 active scalar bound를 각 map 방문 때 읽는다. unknown은 None → CloneMeter의 usize::MAX → 기존 finite clone admission failure가 되어 zero가 되지 않는다(control.rs:376–399). 따라서:

- run/retained checkpoint에 Monolith가 실제 포함된 일반 unknown scope capture는 충분한 다른 예산 아래에서 clone-limit 실패한다.
- Monolith를 소유하지 않는 economy/employee projection이나 run=None의 empty nonbattle state에는 불필요한 ledger 요구를 추가하지 않는다.
- native capture/inner restore는 provenance를 되감거나 insertion credit을 증가시키지 않는다. A/B Session ledger의 최종 commit은 이 slice 밖의 의무다.

## 테스트 oracle의 범위

world/tests/combat.rs:12506–12603은 다섯 capture의 exact/one-under work·slots 경계, source run snapshot 보존, 큰 uncaptured profile의 비용 독립성을 검사한다. source 필드가 빠지거나 잘못 방문되어도 **그 capture가 스스로 보고한 usage**를 다시 cap으로 사용하므로 CP014-F01을 검출하지 못한다. Root가 계획한 완성 candidate의 별도 CloneShape 비교가 이 빈틈을 직접 겨냥한다. 특히 nonbattle helper의 Ok(Some)와 fresh 값 내용도 함께 확인하면 단순 outer Ok 이상의 oracle가 된다.

:12606–12654는 실제 operation에 진입하여 UUID/economy/profile를 바꾸고 late charge Err를 반환한 뒤 economy/UUID 복원, profile before+ during 보존, TLS 해제를 검사한다. operation 도달 Cell이 있어 초기 admission 실패를 rollback 성공으로 오인하지 않는다. 다만 이 source는 run=None이므로 Monolith ledger 포함 transaction rollback까지 증명하지 않는다.

또한 현재 exact-cap 테스트만으로 actual Clone이 preadmission 이전에 호출되지 않았다는 실행 증거를 얻는 것은 아니다. source 순서는 확인되었다. 후속으로 그 성질을 잠그려면 작은 Clone counter fixture로 one-under에서 clone calls=0을 검사하는 방식이 직접적이다. 이 권고를 별도 production defect로 세지 않았다.
