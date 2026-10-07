# B Event owner 근거 정정과 strict restore 연결 — 002

2026-10-06 · root 계획/설계 조정. 이전 001과 publication은 그대로 보존한다.

## 자료 구조 근거 정정

Sol의 caller inventory와 root의 실제 소스 재확인에서 `src/game/world/state.rs:1`은 `crate::owned_collections::OwnedHashMap as HashMap`을 import한다. 이 파일의 RunState 및 RunCheckpointPayload `event_sessions: HashMap<…>`는 기존부터 OwnedHashMap이다. 001의 “기존 raw HashMap”은 alias를 확인하지 않은 잘못된 판단이며 이 기록으로 정정한다. root도 이전 설명을 수용한 책임을 가진다. 새로운 raw→OwnedHashMap migration은 필요하지 않다.

기존 OwnedHashMap은 삽입/삭제 이력과 closed-history CloneShape를 유지한다. 신규 receipt를 visitor가 순회하며 helper의 타입도 actual OwnedHashMap으로 명시한다. 새 native selected membership을 history Vec backing과 source_scene 문자열 및 실제 독립 payload owner로 한정하는 결정은 유지한다. 기존 table backing을 새 owner로 다시 편입하지 않는다. 독립 session/active/run/checkpoint/entry/Continue/DTO clone의 새 payload 선계량은 계속 필요하다. 기존 key의 교체 검증도 유지한다.

EventSession inline field 추가에 따른 기존 table/layout 영향은 실제 allocator/R05 및 필요한 layout 증거에서 별도로 확인한다. selected payload extent만으로 전체 메모리 비용 불변을 주장하지 않으며 cap 또는 predecessor oracle를 완화하지 않는다.

## strict restore 연결

historySome session은 map key와 node_id 일치·노드 존재에 더해 실제 Event category/payload의 event_id와도 일치해야 한다. 다른 종류의 node 또는 다른 Event에 이식을 허용하지 않는다. normal map entry의 실제 권한과 같은 연결이다.

역사 경로가 committed employee-check를 지나는 경우 저장된 choice의 authored DC/aptitude, raw roll 범위, modifier component 합, total roll 및 기존 classify_event_check 결과의 내부 일관성을 확인한다. outcome과 경로를 함께 바꿔도 roll과 모순되는 receipt는 거절해야 한다. 현재 직원·장비를 역사 시점의 소유자로 재구성하거나 현재 loadout에서 과거 modifier를 다시 계산하지 않는다. historyNone legacy session 전체의 새 validation을 추가하는 변경은 아니다.

Sol은 제품 교정을 담당하고 Astra는 빈/중복/순서/분기 및 node/Event binding 변조와 거절 시 상태 보존을 별도 시험한다. 현재 작성 중이며 새 fmt/build/test 실행은 아직 없다. 시험 통과나 최종 완료를 주장하지 않는다.
