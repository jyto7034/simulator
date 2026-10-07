# F02 오류 출처와 관련 producer 검토

GOAL-CRFI-01 TD02의 기존 trusted internal / ordinary refusal 구분을 구현한다.
FPF-CRFI-003·PP-CRFI-002의 의미, 공개 오류 schema, 설정·저장 형식은 바뀌지 않는다.
연속 실행 위임 CF-D06 및 closed u03_20261004_002 F02가 교정 근거다.

| 생산 경계 | 구분 / 교정 |
| --- | --- |
| present sustained Goodbye 정의 조회 | caller activation·caster·skill identity 검증 이후, 보유 record가 참조하는 정의 누락만 Invariant/reinput_goodbye_definition |
| admitted cast deadline | 입력 target와 eligibility 검증 및 action 접수 뒤 max(1,focus)의 checked_add 실패만 CounterOverflow/admitted_cast_deadline. 0/양수 의미 동일 |
| live deployment → native unit | 모든 staged application의 기존 문맥 검사에서 보유 배치 참조의 실제 native unit 존재 확인. application_deployed_unit. 공개 capability DTO가 panic하기 전에 실패한다. 조회량을 기존 authoring/native work accounting에 반영 |
| caller activation 누락 / supplied target 부적합 | 기존 InvalidAction 유지 |
| Goodbye None | Goodbye를 지원하지 않는 유효 overlay의 재입력 거부; 일반 InvalidAction 유지 |
| active Hello 중복 | validate_sustained_start의 정상 거부 유지. admitted 함수 전체를 일괄 변환하지 않음 |
| client skill ID 불일치 / 없음 | 기존 public preflight wire 및 복구 가능성 유지 |
| swap first/second UnitNotFound 후보 | 개별 leaf 변환은 제거. 실제 공개 경로가 capability DTO에서 먼저 panic함을 격리 회귀로 확인했고, 공통 문맥 검사에서 보유 참조 불일치를 거부하므로 모든 해당 mutation owner가 같은 invariant를 소비 |
| native tactical stance/swap/relocation post-validation lookups | 같은 동기 실행 안에서 preflight가 존재를 보장하며 사이에 해당 unit/state를 삭제하는 producer가 없다. caller 부적합/미배치·dead/활동 잠금은 기존 eligibility 거부 유지 |

전역 cause 분류는 변경하지 않는다. 실제 trusted producer는 기존 first-native-fault ledger를 이용하고
기존 expression/native 원인을 덮어쓰지 않는다. 새 검사도 후보 생성 전/commit 전의 기존 owner 경계다.
private GameCore는 동일 typed 실패를 반복 반환하고 complete root rollback을 유지한다.
CoreApplication에서만 published battle first fault를 보존하고 반복 명령·tick·writer를 막는다.
대조군의 unsupported fixture는 정상 RON에서 Goodbye None과 독립 정적 child 표현식으로 컴파일한다.
테스트용 내부 손상을 외부 사용자 입력으로 만들 수 있다는 주장은 하지 않는다.

영향은 V01–04/V10–12 및 모든 Core/host source 전이 소비자다. 전체 feature matrix·16 공개 소비자·
5 fault 관측 모드/실제 stock WebSocket을 갱신하고 100 inherited/31 R06 조건을 유지한다.
전체 semantic completion은 구현 비참여 GPT-6 Astra / High successor 감사가 판정한다.
