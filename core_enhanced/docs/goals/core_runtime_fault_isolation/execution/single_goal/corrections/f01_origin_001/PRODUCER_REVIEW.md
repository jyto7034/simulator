# F01 origin 재검토

기존 FPF-CRFI-003 / PP-CRFI-002 / GOAL-CRFI-01 TD02의 내부 실패와 일반 거부 구분을 구현한다.
정책·외부 JSON·save schema·설정 한도 변경은 없다. 연속 실행 위임(CFD06)과 closed audit F01을 근거로
실제 trusted producer 또는 입력 거부가 이미 처리된 내부 실행 경계에서만 기존 typed Invariant를 만든다.

| 경계 | 기존 공유 오류 | 교정과 일반 거부 경계 |
|---|---|---|
| employee reconciliation | InvalidAction / UnitNotFound | 침식 출처, 침식 명부, incapacitation 명부를 세 고정 site로 구분 |
| closed lifecycle/context validation | InvalidAction | lifecycle와 보유 문맥의 불일치만 application_context. query/profile/admin의 application 분류 규칙은 유지 |
| six live command result checks | InvalidAction | 이미 성공한 native 명령의 예상 밖 outcome/identity만 invariant |
| active advance after triggers | InvalidAction | 원래 존재하던 active owner 소실만 invariant |
| active native step | InvalidAction / UnitNotFound | cursor 검증 뒤 내부 simulation의 전파 실패만 active_battle_step; elapsed 입력에 외부 actor/skill ID는 없음 |
| owned event dispatch | InvalidAction / UnitNotFound | 이미 소유한 event의 실제 dispatch 전파 실패만 typed. 정상 stale-target/cancel/noop 처리는 내부 handler의 기존 Ok 경로 유지 |
| boss runtime boundary | InvalidAction / UnitNotFound | 실제 내부 계획/카운터/미세 단계 실패만 typed. boss action 시도에서의 일반 취소는 해당 handler가 소비하며 경계 밖으로 전파되지 않음 |
| withdrawal retained unit | UnitNotFound | withdrawal 성공 뒤 유닛 소실만 invariant; 사용자가 지정한 미배치 직원 조회는 일반 거부 유지 |
| retreat snapshot/restored phase/session | InvalidAction | 현재 battle에서 허용된 retreat의 저장 문맥 손상만 typed. battle 자체가 없는 호출은 일반 거부 유지 |
| sustained committed reinput child | InvalidAction | 사용자 activation/target 검증과 consume 이후 내부 child invocation/event 누락만 typed; 이전 activation/target 오류는 일반 거부 유지 |

`trusted_failure`는 문자열을 검사하지 않으며 두 공유 variant만 해당 내부 경계에서 변환한다.
다른 native/expression/resource 오류는 원래 typed cause를 보존한다. 새 invariant는 기존 native 첫 실패 ledger를 쓴다.
`cause()`의 일반 UnitNotFound/InvalidAction 판정은 수정하지 않는다. 동기적 오류 회수·private 후보 rollback은 계속
호출자에게 오류를 돌려주며, sticky terminal은 기존 CoreApplication의 published battle mutation 경계에만 존재한다.

음성 대조군은 실제 없는 직원 배치와 잘못된 sustained activation을 공개 application으로 요청한다.
두 호출은 각각 unit_not_found / invalid_action이며 root 변경과 terminal capsule이 없어야 하고 이후 정상 advance가 성공해야 한다.
감사 손상은 test-only 내부 fixture에서 만든다. 외부 사용자가 손상 상태를 만들 수 있다는 주장은 하지 않는다.

검증 영향: V01–04, V10–12, core source의 전이 의존성 및 actor/observer/writer 의존성.
전체 Core feature 3종과 host 2종, 현재 public 소비자, 실제 fault 5모드 및 stock websocket을 새 source에서 재실행한다.
원본 audit·target·오류 실행 로그는 보존한다. 독립 successor가 12 acceptance / 12 validation / 10 dimension /
100 inherited condition / 31 R06 mapping의 최종 의미 판정을 맡는다. U04 implementation은 아직 blocked.
