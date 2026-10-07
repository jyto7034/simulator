# 완료 Event 선택 이력의 현재 층 수명 — 001

2026-10-06 · root 계획/설계 결정. Sol/Astra 역할 분리를 유지한다.

`relic_event_owner001`은 source424/31b4b015…에서 실제0 PASS/4 FAIL, 소스 불변이다. 네 시험 모두 완료 후 이력이 없거나 decline-only owner가 비활성인 상태에서 실패했다. strict restore 변조 matrix 뒤의 성공을 주장하지 않는다.

root와 Sol이 실제 `commit_node_completed_with_recovery`의 무조건 `event_sessions.remove(node_id)`와 `RunState::for_next_floor_from`·`for_run_complete_from`의 `Self::new`를 확인했다. 새 history를 owner/clone/Continue에 연결했지만 완료 경로가 이를 삭제한 제품 수명 연결 누락이다. TD02 한번소비 증표 및 TD05 저장/capture/restore 계약과 이미 합의한 완료 history·합법 분해 이후 복원 검증을 충족하도록 다음을 적용한다.

- 완료되는 Event session에 historySome이 있으면 현재 Run의 현재 층 map에 보존한다. 현재 map의 node/Event payload 결속과 authored DAG의 유한한 history 범위를 유지한다.
- historyNone의 기존 Event completion cleanup은 그대로 유지한다. 기존 완료 노드의 재진입 권한을 새로 만들지 않으며 exact historical retry도 같은 active Event에만 허용한다.
- 다음 층·terminal·새 원정의 기존 current Run 생성/초기화는 유지한다. 전 원정 별도 history ledger를 만들지 않는다. 별도로 소유된 SavePoint/Entry/Continue snapshot이 정당하게 보존하는 이전 값은 그 snapshot의 기존 수명을 따른다.
- retained history는 기존에 승인한 각 독립 Vec/String owner 계량/clone/admission 및 strict restore를 적용한다. completed history의 실제 추가 보유 비용을 최종 측정에서 확인하며 한도 상향이나 누락으로 숨기지 않는다.

Sol은 node_flow.rs 제품 최소 교정을 맡고 Event owner4의 기대를 낮추지 않는다. Astra는 별도 u08_query_owner.rs의 native scope 시험 오류를 교정한다. 파일 소유가 겹치지 않으며 두 writer가 HOLD로 돌아오기 전에는 format/build/run하지 않는다. 현재는 교정 승인/작성 경계이며 교정 후 통과를 주장하지 않는다.
