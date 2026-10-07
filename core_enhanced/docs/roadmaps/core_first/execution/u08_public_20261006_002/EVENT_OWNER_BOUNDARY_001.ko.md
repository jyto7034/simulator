# B Event receipt의 신규 native owner 경계 — 001

구현 전 Sol 조사와 root의 enter_event_node/commit_event_session_required_run 실제 코드 확인으로 결정했다. EventSession은 기존 raw HashMap의 value이고 Event 진입 때 key가 만들어진다. corruption 선택은 그 기존 key의 value를 교체한다.

신규 선택 native membership은 history의 추가 heap(Vec backing, 각 source_scene 문자열, 별도 heap에 있을 때 실제 독립 payload header)이다. active/run map/checkpoint/entry/Continue의 실제 독립 history owner와 조회 DTO·중간 clone을 각각 포함한다. 공유 Arc handle을 deep-count하지 않는다. CloneShape와 allocation 이전 admission을 함께 연결한다.
historySome opt-in에서 전체 EventSession clone의 기존 문자열/committed 데이터까지 보수 선계량하는 것은 허용된다. 기존 Advance/Resolve/State query/enter의 raw clone도 새 history를 복제하므로 새 Choose handler만 계량하면 완료가 아니다.

기존 raw HashMap backing 전체를 신규 receipt owner로 편입하거나 capacity로 실제 bucket 수를 추정하지 않는다. 기존 remove/clear 경로가 있어 tombstone-free capacity 가정을 둘 수 없다. 이 작업에 OwnedHashMap·closed-history migration을 추가하지 않는다.
실제 선택 경로에서 기존 key의 존재를 보장·검증한다. historySome 상태에서 새 map key 삽입/할당이 가능하다면 누락으로 넘기지 말고 근거와 대안을 root에 반환한다.

EventSession의 inline field 추가로 value/table layout이 달라지는 것은 별도 실제 메모리 영향이다. 이를 '새 heap0' 또는 '전체 table 비용 불변'이라고 주장하지 않는다. 전체 allocator/R05와 필요한 layout 증거에서 raw 전후·원인을 연결한다. 여기의 selected native extent는 기존 table 전체나 RSS 수치가 아니다.
oldNone canonical/legacy 동작과 noRelic 비용 보존은 실제 결과 또는 명시적 비영향 근거로 확인한다. payload 계량 범위를 좁게 정의했다는 이유로 기존 cap을 올리거나 새 거절을 허용하지 않는다.

Astra에게 같은 범위를 전달했으며 decline-only 활성화, 각 owner와 기존 장면 진행/조회 clone, strict restore·decode 전 hint와 예산 실패/rollback을 검증한다. 현재 B/C 제품·시험 작성 중이고 실행은 HOLD다.
