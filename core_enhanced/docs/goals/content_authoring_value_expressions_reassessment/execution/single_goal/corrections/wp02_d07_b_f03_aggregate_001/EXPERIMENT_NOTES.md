# C06 현재 판단

filter는 aggregate 입력에만 적용한다. 제외된 소환물/적도 delivery의 실제 피해 대상에는 남는다. 네 node의 view를 합치거나 재사용하면 안 된다.
C04/C05 교훈을 따라 release carrier와 runtime_fact_source의 delivery를 함께 소비자에 결속하고 복원한다. slot projection/attack 변경은 private fixture의 시점 변화이며 실제 장비 교체 UI/저장 흐름을 검증하지 않는다.
private encoder는 public selector를 건너뛰므로 C07/C08을 닫지 않는다. InvalidOwner 후 이전 record 보존은 C06 관계 오류 경계이며 C10 전체 실패/취소 계약을 대체하지 않는다.
RON MIN의 직렬화 후 역직렬화 실패는 C03-OBS01로 C15에 유지하며 이번 source에서는 수정하지 않는다.

필터 predicate도 compiler의 canonical node ID를 소비한다. member Read를 aggregate+1로 가정하지 않고 이 fixture의 AST/predicate 개수에 맞춘 명시적 node/read 쌍으로 trace를 검증한다. 기대값을 실제 trace에서 복사하지 않는다.

Cargo가 자동 발견하는 새 integration test도 실제 검증 입력이다. 처음 계획한 source 목록의 byte만 확인하면 이 추가 입력을 놓칠 수 있다. 시작 src/tests 전체 membership으로 부재를 확인한 뒤, 소급 baseline 변경 없이 WORKSPACE_ADDITION과 source after에 별도 출처로 결속한다. 새 실행은345개 입력에 귀속한다. 이 test crate는 product library/PC36/R05에 링크되지 않으므로 production 무변경과 선행 측정 적용성은 유지한다.
