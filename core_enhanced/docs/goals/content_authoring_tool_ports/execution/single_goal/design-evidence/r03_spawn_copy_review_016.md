# R03 spawn-copy 읽기 검토016

AUTORUN §8 bounded source 검토이며 완료 감사가 아니다. 조사 중 source/docs/Git 수정, cargo/build/test 실행은 하지 않았다. Root가 알린 INPUT056 compile failure와 INPUT057 재검사는 별도 실행이며, 여기서는 실제 source와 테스트 oracle를 읽었다.

**INPUT056의 charge_work 오기를 확인했고, 나머지 조사 경계에서는 추가 concrete 결함을 찾지 못했다.** 마지막 settlement copy 실패는 성공 callback과 domain Err 양쪽에서 snapshot을 복원하고 Pending을 유지한다. 이 판단은 해당 source slice의 검토이며 전체 clone/Goal 승인 또는 테스트 통과 판정이 아니다.

## Capture와 실제 successor

- 최초 2026-09-10 10:36:20.317693 UTC.
- spawn_kernel.rs SHA-256: `1de6d8a3449561bf1fe0798fb209055a43f7c2fb3ca36a903cf5967cd7dbe3a7`.
- INPUTS_056.json의 `/files/55`가 같은 path/hash를 갖는다. Manifest SHA-256: `0bbf3a24696e47f7941d83b4a3ecf501088a02197821f66c0be878f3a063c810`.
- close 10:40:13.295056 UTC에 spawn_kernel.rs만 `fab3d3c46f54e76281bf98f6e7430192a9599b5c6dd364190f6378b1ae2bd9a0`로 바뀌었다.
- successor의 단 하나인 `crate::authoring::control::charge(`를 메모리에서 `charge_work(`로 되돌린 SHA가 initial과 정확히 같다. 실제 drift는 이 한 식의 교정이다. 다른 inspected files 5개는 동일했다. Repository 파일을 되돌린 것은 아니다.
- 모든 source path/hash/span과 dependency lookup은 JSON에 기록했다.

## SP016-F01 — INPUT056에서 존재하지 않는 control 함수 호출

위치 spawn_kernel.rs:368–373(initial hash 위 참조). `control::charge_work`를 호출하지만 실제 control.rs에는 `charge(units, site)`만 있다(:473–486; SHA `8230d912809e601520f9c2acd7c4dadf0f67a186634309617bd8fa34b4173841`). 따라서 initial source는 compile되지 않는 이름 오류다. Root가 자신의 compile1/tests0 실행으로 먼저 알린 오류를 source에서 확인했다.

최소 수정은 함수명을 charge로 바꾸는 것이다. **close successor에서 그 source 교정은 확인했다.** 후속 compile/test 성공은 이 보조가 실행하거나 검증하지 않았다. 별도 신규 런타임 defect로 부풀리지 않는다.

## Normalization·identity·reservation

clone_for_spawn(:349–354)는 controlled_clone(:24–34)의 preflight/actual clone/post-poll 실패를 ExecutionInterrupted로 전달한다. underlying Attempt가 latched 진단을 계속 소유하므로 generic kernel enum을 정확한 limit code로 오인하지 않는다.

normalize_request(:363–412)는 outer control Result와 inner domain Result를 분리한다. 요청 ID scan charge가 reject 검사보다 앞에 있다. identity_request(:357–360)의 `normalize_request(&request)?.unwrap_or(request)`는 **inner domain rejection에서만** 원래 request를 move한다. outer interruption을 성공 identity로 바꾸지 않는다. 성공 request는 기존 order 정렬된 normalized payload, 거절 request는 원래 order를 갖는다. invalid terrain fixture :1167–1176이 그 identity를 비교한다.

reserve_request(:415–431)는 기존 payload의 같음/충돌을 확인하고, 새 key의 fallible copy를 끝낸 뒤 Pending record를 insert한다. execute_spawn_request의 domain-invalid 경로(:636–643)는 request ID와 cached rejection을 준비한 뒤 reserve/settle한다. 이 구간의 실패가 정상 rejection으로 캐시되는 흐름은 없다.

## Trigger·cache 상태 쓰기 순서

- bind_trigger(:464–493): identity와 필요한 request/trigger ID copies를 준비한 뒤 reserve하고 Bound를 쓴다. 같은 Bound request의 replay는 기존 reserve identity 검사를 재사용한다.
- settle_trigger(:496–524): request ID, cached settlement, trigger ID의 세 fallible copy를 모두 마친 뒤 Settled 상태로 insert한다. 실패 도중 Bound를 Settled로 앞당기는 쓰기는 없다.
- cancel/reject 계열(:527–575): reserve 뒤 final cached settlement copy가 실패하면 새 reservation은 Pending으로 남는다. 이미 Settled인 request의 cached-result copy 실패는 기존 terminal cache를 변경하지 않는다.
- execute cache replay(:623–632)는 identity 확인 후 controlled cached copy를 반환하며 mutation callback을 다시 부르지 않는다.

모든 성공 status write 이후 이 함수들 내부에는 새로운 fallible copy가 없다. cooperative cancellation의 마지막 poll 이후 도착하는 신호까지 atomic timestamp로 보장한다는 주장은 아니며 enclosing authoring acceptance는 계속 필요하다.

## Snapshot lifetime와 마지막 copy

execute_spawn_request(:646–700)의 owner 순서는 다음과 같다.

1. normalized request를 reserve한다.
2. whole-core preflight(:649–650), native snapshot Clone(:651), post poll(:652–653).
3. self의 log COW 준비(:654–657). 준비 실패는 before로 복원한다. 기존 helper는 no-scope이면 바로 반환한다(event_log.rs:444–474).
4. placement callback 직후 poll(:663–666)이 실패하면 before 복원 후 ExecutionInterrupted를 반환한다. domain Err를 먼저 정상 Rejected로 바꾸지 않는다.
5. settlement를 만들되 before를 아직 보존한다. **마지막 clone_for_spawn(:689–695)이 실패하면 callback 결과와 무관하게 before로 복원한다.**
6. copy 성공 뒤에만 domain rejection이면 before를 복원(:696–698)하고, terminal settlement를 기록(:699)한다.

before가 reservation **후** snapshot이므로 어느 interrupted branch에서도 해당 request는 retry 가능한 Pending이다. 성공 callback의 partial state나 domain rejection terminal record를 남기지 않는다. rollback은 control::Attempt를 복원하지 않아 fuel/first failure를 되살리지 않는다.

No-scope에서는 copy wrapper가 그대로 native Clone을 반환하고 BattleCore preflight/log preparation/poll은 no-op이다. request canonicalization, duplicate-unit rejection, domain-error whole-core restore, cached replay, trigger exact-once의 분기 의미가 유지된다. before의 drop을 마지막 copy 뒤로 늦춘 것 외에 새로운 gameplay/ID/RNG 작업을 넣지 않는다. 임의 callback의 외부 side effect까지 복원한다는 의미는 아니다.

## 테스트 oracle가 실제 겨냥하는 경계

- :1121–1165의 low clone cap 검사는 callback 0회와 empty request registry를 확인한다. execute와 cancel의 identity fallback에서 control failure가 정상 request/cancellation이 되지 않는 경계다.
- :1007–1074는 normalization/reservation prefix의 peak를 별도로 측정해 그보다 큰 BattleCore snapshot을 거절한다. callback 0, Pending, 이후 성공 retry를 확인한다.
- :1078–1117은 callback 안에서 skill_cast_seq와 event log를 바꾸고 cancel한다. 첫 callback 뒤 중단, log/seq 복원, Pending을 확인한다.
- **:1181–1257은 domain_error=false/true 둘 다 측정한다.** fixture의 authored placement order [1,0]이 정렬되어 callback 두 번을 통과한다. full usage-1에서 `calls=2`와 `failure.site=owned_clone_copy`를 동시에 요구하므로 prefix normalization/whole-core copy의 동명 failure를 마지막 copy 실패로 오인하지 않는다. 두 callback 뒤의 유일한 controlled copy는 :689이다. skill_cast_seq 복원, Pending, no-scope retry 결과까지 확인한다.
- :1260–1297은 cached replay callback을 panic으로 금지하고, last-copy one-under 실패에서 spawn_kernel Debug 상태와 다음 cached reply를 비교한다.
- ordinary tests :1303–1353,1395–1464는 bound trigger identity, removal-first cancellation/replay, 128-placement canonical order, duplicate ID rollback을 다룬다.

한계: 마지막-copy fixture는 skill_cast_seq만 직접 바꾸며 event queue/log는 바꾸지 않는다. 그 두 equal assertion 자체는 해당 실패의 큐/로그 복원에 대한 독립적인 mutation oracle가 아니다. 로그 mutation 복원은 앞선 cancellation fixture가 직접 다룬다. 새 focused tests는 trigger ID/settlement copy 각각의 one-under failure를 별도로 실행하지도 않는다. Source의 write ordering은 확인했다. 필요하면 Bound 상태에서 settle_trigger 마지막-copy cap을 실패시켜 Bound 유지/성공 retry를 직접 검사하면 된다. 이를 현재 production 결함으로 세지는 않았다.
