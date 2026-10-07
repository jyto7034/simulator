# U03 기술 선택 및 실패

P2G schema-only successor repairs preserved; policy meanings unchanged. Python executable is python3 (python absent).
Concrete source boundary refinement follows CODE_REVIEW TD01–08 before mutation.

GEX-10 상세화: application/fault.rs의 Copy capsule은 battle UUID/revision, 닫힌 operation,
native fault(정적 site/정수) 또는 expression code/phase/node만 보관한다. GameCore의 내부 allocation-free
active authority accessor로 진단 이전 identity를 읽는다. command의 기존 ActionScope::Query는 비terminal,
admin/profile/prepared/export는 gate만 적용한다. 첫 raw Err와 성공 후 public projection은 별도 문장이다.
checked gate는 observer begin 이전에 실행하여 반복 거절이 최초 실패 관측을 교체하지 않는다.
첫 latch에서 SessionObserver latest만 clear하고 lease/peak/summary/commit count는 유지한다.
Run JSON에 fault를 추가하고 action_capabilities를 비우며, host는 별도로 typed query→fault snapshot을
직접 구성하여 store.load와 큰 snapshot을 실행하지 않는다. 정상 JSON은 기존 shape를 유지한다.

Focused004에서 실제 missing skill 입력이 기존 InvalidStaticData를 반환함을 확인했다.
TD02 origin을 source producer까지 상세화: handle_activate_skill의 caller-selected missing ID만
GameError::InvalidCommandData로 표시하고 public kind/code/details는 기존 InvalidStaticData와 동일하게
유지한다. trusted roster-bound lookup은 원래 InvalidStaticData로 남는다. 문자열 비교 없이 일반 입력이
fault로 승격되는 것을 막는 승인 PE01/12 구현 교정이다. 새 정책 의미/acceptance 변화 없음.
postcommit 취소 시험은 기본 authoring clone cap이 먼저 실패해 목적 seam에 도달하지 못했다.
시험 전용 유한 clone/metadata/work allowance를 늘려 이미 존재하는 cancellation hook의
application_diagnostic_entry에서만 실제 공개 port의 성공 commit 후 projection 취소를 검사한다.


최종 postcommit probe 보정: 큰 allowance만으로는 미소유 native table provenance를
해결하지 못했다. ValidatedGameData는 scope 밖에서 로드하고 fresh NativeStorageProvenance로
새 app을 구성한 뒤 같은 provenance를 이어받아 실제 public advance의 diagnostic hook에서
취소했다.007의 Debug 제약 compile 실패도 보존했고008은 성공 commit 뒤 authoring_cancelled,
fault 없음과 실제 revision overflow의 전체 root 불변을 모두 검증했다.

Root 검토: application/fault.rs는 GameCore prepare/commit 밖에 있으며 GameCore의
world/application.rs prepare_application/commit_application 구현은 U02와 동일하다.
실제 native 실패는 public Run/battle event/resource 투영을 보존하고, 같은 transactional owner를
통과한 실제 dynamic expression와 revision overflow는 battle_advance_atomic_snapshot 전체
(root RNG/queue/UUID/resource/profile 포함)를 비교한다. 공통 producer는 full GameCore clone의
실패를 버리고 단일 commit_application에서만 root를 교체한다. first-fault capsule과 observer
종료는 그 실패 반환 뒤 application 소유 상태에 기록하므로 rollback에 의해 사라지지 않는다.

최종 matrix: Core default1707,fixtures1707,observation1714 및 Host default81,
observation88 PASS; 관측 heap의 global allocator 시험1건은 혼합 suite에서 의도적으로 ignored,
별도 단일 thread 실행 final_heap001에서 PASS. fmt/check/event_log/Clippy/release PASS.
Clippy의 기존 두 allow와 기존 경고는 원본 그대로이며 warning-free라고 주장하지 않는다.
공개 호환성 실행/PC/R05와 최종 fault 소비자 binding은 뒤따르는 immutable receipt를 확인한다.
