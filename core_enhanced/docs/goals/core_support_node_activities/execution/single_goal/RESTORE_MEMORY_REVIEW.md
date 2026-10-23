# U16-1 확장 보유 이어하기 계량 검토

실제 작성 Event가 장비64개를 지급한 정상 원정에서 HQ 상점 구입/완료까지 성공했다. 장비71개·소비품2종(occupied73)의 새프로세스 Continue는 기존 16MiB 한도에서 `relic_continue_value requested19939869`로 실패했다. 원문은 `runs/normal_host002/expanded_completed_continue003`, 입력은 해당 실행의 preimage 및 restore_cost001/inputs/ORIGIN.json에 보존했다. active 파일230331bytes/Core payload171980bytes를 성공하도록 수정하지 않았다.

원인은 JSON 객체 필드 하나마다 11개 key/value를 담는 B-tree 노드 전체를 예약한 계산이다. typed decode→checksum Value→stable-capture Value의 독립 예약3회 누적은 정상 계약이므로 제거하거나 환급하지 않았다. 기존 typed decode 예약은 유지하고, 두 Value producer만 객체수 + floor(전체필드수/5) 노드로 정밀화했다. 빈 객체도 root 하나를 예약한다. 각 비루트 노드는 최소5개 키를 가지므로 이 수는 실제 노드수의 상계다. 현재 serde_json의 기본 Map은 BTreeMap이며 prepared cargo-tree에는 preserve_order가 없다. 노드당 11 entry+224bytes 기존 상계, 문자열·배열·정렬·출력버퍼의 독립 예약은 유지한다. 생성 과정의 삽입은 노드를 추가하므로 완료 트리의 노드 상계가 중간 트리에도 적용된다. checked overflow는 실패하며 결과를 먼저 할당하지 않는다.

근거: [Rust BTreeMap 비루트 점유 불변식](https://doc.rust-lang.org/src/alloc/collections/btree/map.rs.html), [Rust 노드 용량과 split 최소점유](https://doc.rust-lang.org/src/alloc/collections/btree/node.rs.html). 실제 컴파일러 rustc1.98.1, 기본 serde_json graph를 prepared-build.json에 고정한다. 이는 Rust의 영구 API 보장이 아니므로 toolchain/Map backend 변경 시 기존 노드 크기 가정과 함께 재검증해야 한다.

`test_parts/restore_cost001/probe/src/bin/value_bound.rs`는 제품 allocation.rs를 그대로 include하고 admission sink만 관측 stub으로 둔다. 정방향/역방향 map20개, 중첩 배열9개와 실패 당시 실제 checkpoint1개에서 실제 allocator peak growth<=제품 예약량, 해제 후 정확 회수를 검사했다. 30/30PASS. 실제 checkpoint Value는 예약3058634bytes/실제peak1321172bytes였다. 이 probe는 정상 host 승인 증거가 아니며 순수 producer 상계 관측이다.

별도 public CoreApplication 진단은 같은 저장 payload/profile과 현행 정상 설정으로 fresh process3회 Continue를 실행했다. 누적 native예약14151702bytes, 실제heap 증가 정점1858602~1859234bytes, 시간0.1185~0.1394초(dev opt0/debug0)였다. 모든 inventory가 정상호스트 완료 직후 값과 같고 최종 owner 해제 후 warm generation baseline으로 정확 회수됐다. 이 수치는 RSS/할당자 파편화가 아니다. 한도 변경 없음. 최종 정상 release host/계측 worker를 같은 소스로 재빌드하고 normal/expanded calibration003·review003·adoption003을 통과했다. runs/normal_host003/expanded_completed_continue004는 같은 확장 보유 상태의 fresh-process Continue를 통과했다. 정상호스트 전체는15프로세스84명령/실제IO4회이며 원래 한도는 유지했다.

회귀: 장비64개 추가+신규3상품 구매/완료 후 default Continue1PASS, 기존 U08 관련44PASS. 누적3단계 및 각 admission 직전1byte 부족의 atomic refusal/정확상한 성공 시험도 통과했다. 기존 fullCore 영향 없는 결과는 유지하고 복원 관련을 현재 소스로 재검증한다.
