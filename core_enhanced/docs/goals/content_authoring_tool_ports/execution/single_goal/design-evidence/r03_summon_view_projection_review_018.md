# R03 summon-view projection 읽기 검토018

AUTORUN §8 bounded source 검토이며 완료 감사가 아니다. source/docs/Git 수정, cargo/build/test 실행은 하지 않았다. Root의 compile/test 진행 결과를 이 보고서의 실행 근거로 사용하지 않았다.

**검토한 projection/scan/materialization/sort/consumer 경계에서 수정이 필요한 concrete production defect를 찾지 못했다.** 아래는 그 판단의 source 근거와 테스트가 직접 확인하지 않는 범위다. 전체 Goal, whole clone 또는 모든 summon-runtime 비용의 완료 판정이 아니다.

## Capture / INPUT pin

최초 capture: 2026-09-10 10:53:46.932285 UTC. 직접 owned-leaf source 추가 capture: 10:55:03.317290 UTC. close: 10:57:45.278378 UTC. 파일 7개 재해시 drift 0.

- boss_runtime.rs: `e7049c14d235cf5415b9785c33b8450dfff12b36571f830d5f7d9475090f3516`.
- summon_runtime.rs: `d045c86228b29964214d3c544af889aeb1983707ed4bd9cc354b1a5303320e0c`.
- INPUTS_059.json: `88645999ce1a35a2b8b6405b1e30e6aaff8800bd6bb25e17de5379e1d92ba536`; `/files/29`, `/files/56`가 위 두 path/hash와 각각 일치한다.
- 모든 path/hash와 실제 source span은 동반 JSON에 보존했다.

## Field 비용과 첫 복사 경계

boss_runtime.rs:200–242의 owned BossSummonTaskView 7개 field와 projection macro의 7개 typed reference가 일치한다. :224는 reference-object 자체의 shape가 아니라 **명시된 owned T의 CloneShape**를 호출하며, :229는 같은 typed field에 T::clone을 호출한다. complete target literal에 struct update/default 생략이 없어 새 owned field를 빠뜨리면 컴파일에서 드러난다.

7개 중 task는 ActiveBossSummonTask의 실제 11개 field(:174–185)를 전부 방문하는 native struct_shape(:306–318)로 연결된다. request/definition/batch/route Strings, placement의 zone String, occupancy Vec, lifetime/insufficient-space/removal-signal까지 direct owner와 visitor를 대조했다. 이 inspected task payload에는 compiled graph Arc를 따라 확장하는 필드가 없다. 관련 native leaf는 boss_runtime_data.rs:230–280,794–814와 summoned_unit_data.rs:22–42이다.

선계수 총합은 **Vec record 1 + 각 selected element slot 1 + owned view record 1 + 7개 child shape**다(:1571–1580). 이는 기존 Vec<T>::CloneShape의 1+len+각 owned T shape와 정확히 같은 계수 구조다(clone_shape.rs:72–79). output 전체를 합산한 하나의 meter.finish 이후에만 Vec::with_capacity(count)(:1583)와 source.into_owned()(:1588)를 실행한다.

여기서 “전체 admission”은 완성 output의 native storage와 copy work다. **정렬 work는 복사 뒤 각 비교 직전에 별도 admission**된다. 따라서 정렬 단계 오류에서 이미 만든 local Vec를 폐기할 수 있으며, 모든 후속 정렬 work까지 첫 복사 전에 예약했다는 주장은 하지 않는다.

## Count / overflow / 두 pass의 동일성

count 증가(:1577) 직전에 selected element slot과 owned view를 모두 finite meter에 admission한다. max_clone_slots는 positive/hard 2^20으로 검증된다(control.rs:103–107). 최소 두 slots 이상이 매 selected view에 필요하므로 count는 usize::MAX에 도달하기 전에 admission 실패한다. count += 1의 unchecked 표기에서 현재 hard-limit/ordering을 깨는 overflow 경로를 찾지 못했다.

두 pass는 같은 visit_summon_view_sources를 같은 immutable &self로 호출한다. selector가 보는 actors/control/pattern/tasks/result는 소유 값이며 현재 projection child Clone이 이를 변경하지 않는다. 첫 pass callback은 meter/count만 바꾸고 두 번째는 local Vec만 바꾼다. 따라서 두 번째 pass가 preflight보다 더 많은 view를 만들어 capacity를 넘기는 mutable ingress를 찾지 못했다.

scan은 actor마다 filter 이전에, eligible pattern의 task마다 Summon/result filter 이전에 charge한다(:1531–1549). 사라진 actor identity, control/pattern 부재, non-Summon task, settled Summon도 실제로 접근한 eligibility scan 비용을 피하지 않는다. preflight의 units=2가 두 pass를 선계수하고 materialization의 units=0도 control::inspect를 거쳐 sticky failure/cancellation을 확인한다. callback 또는 charge 오류는 즉시 ?로 반환된다.

no-scope에서는 preflight pass와 count-capacity 예약을 생략하고 Vec::new 후 한 번만 scan한다. charge(0)/final poll은 기존 control의 no-scope Ok 경로이며, ordinary traversal과 native sort를 수행한다. 이것은 작업량 추정/논리 slot 모델이지 source+output coexistence의 allocator/RSS 측정은 아니다.

## 정렬과 오류 후 수명

active 정렬(:1591–1611)은 left <= right이면 멈추고 **left > right에서만** 인접 swap한다. 따라서 같은 request_id의 원래 actor/task emission 순서를 유지하며 no-scope stable sort_by(:1613)와 동일한 ascending String order를 만든다. unsafe 비교나 key 재생성은 없다.

각 비교 직전 min(left.len(),right.len())+1을 u64 checked conversion/add로 계산하고 charge한다(:1597–1604). 문자열 비교가 읽을 공통 prefix의 최대 byte 길이와 길이/순서 판단을 보수적으로 계수한다. conversion/add overflow는 u64::MAX로 보내 finite work limit에서 거절한다. cursor 감소는 cursor>0 안에서만 일어나며 index/cursor/swap 범위가 벗어나는 경로는 없다.

insertion sort의 최악 O(n²)는 remaining fuel로 실제 비교마다 중단된다. no-scope에는 그 알고리즘을 강제하지 않는다. 비교 charge 또는 materialization 중 scan0 poll이 실패하면 partially built/sorted Vec는 local drop되고 runtime은 immutable하다. 마지막 element copy 중 도착한 취소도 이후 comparison 또는 complete poll(:1615)에서 검사한다.

consumer summon_runtime.rs:152–175의 map(...).transpose()?는 projection Err를 empty list로 삼지 않는다. projection이 실패하면 trigger registration/pending insertion loop에 진입하지 않는다. 이것은 **projection 오류**에 대한 보존 근거다. 그 뒤 다른 consumer 작업 전체의 atomicity나 아직 미전환된 copy 비용까지 이번 결과로 닫지는 않는다.

## 새 테스트 oracle

summon_runtime.rs:1363–1474를 직접 읽었다.

- 3개 actual waiting view의 ordinary/controlled Debug 전체를 비교한다(:1383–1393). 별도의 completed Vec<BossSummonTaskView> native visitor를 적용하여 clone_slots_peak가 같음을 비교한다(:1394–1407). 따라서 projection의 Vec/element/owned-view record 누락을 스스로 보고한 usage만 재사용하는 방식보다 직접 검출한다.
- native owned-copy work보다 projection work가 큰지 확인하고, exact work+slots cap에서 같은 결과/usage를 요구한다(:1408–1421).
- work 또는 slots one-under에서 오류와 immutable runtime 유지(:1422–1443)를 확인한다.
- 같은 runtime의 tasks를 settled로 바꾼 후 빈 output의 slot peak=1, work>2를 요구한다(:1445–1461). max_work=1이 actor_scan에서 실패해야 하므로 empty result를 이유로 scan을 생략하는 회귀를 직접 겨냥한다(:1462–1474).

실행 증거와 구분할 한계:

1. one-under loop는 두 error code 중 하나를 허용하고 실패 site를 특정하지 않는다. work cap이 마지막 sort 비교를 겨냥하는지, slots cap이 materialization 이전을 겨냥하는지 source 순서로는 판단할 수 있으나 테스트가 그 위치를 별도 증명하지는 않는다.
2. 같은-key tie와 의도적인 역순 request_id/긴 공통-prefix fixture가 명시되어 있지 않다. stable sort parity는 현재 비교/swap 조건으로 확인했다.
3. 취소 token 또는 private scan callback의 강제 Err를 이 신규 test에서 주입하지 않는다. local drop/consumer 전파는 source를 확인했다.
4. child native visitor는 completed-output oracle와 공유된다. 이 test가 각 child visitor 자체의 모든 heap 비용을 독립 검증하는 것은 아니다.

필요하면 각 limit별 expected code/site, equal-key emission-order와 역순 sort fixture를 추가하면 위 실행 공백을 좁힐 수 있다. 이 권고를 현재 production defect로 세지 않았다.
