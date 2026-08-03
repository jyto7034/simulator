# Employee Psychology State Contract Experiment Notes

## 초기화 기록

- goal_name: `employee_psychology_state_contract`
- 선행 Phase 1/2/3: complete
- production baseline: Core 775, game_server 30, Unity EditMode 240, ordinary PlayMode 57

## 구조적 판단

- personality는 authored Profile이고 Psychology는 run-local mutable aggregate다.
- trust score, derived band와 semantic memory는 checkpoint/retreat lifecycle을 공유하므로 한 Phase에서 전환한다.
- high-risk/refusal/modifier/dialogue 콘텐츠는 aggregate 계약과 분리된 후속 authoring이다.

## Source-of-truth 충돌

해소됨. current runtime의 authored personality는 이미 Phase 1 Profile에 있었고, 기존 trust 모듈의 dormant cue/modifier/high-risk 타입만 공개 surface로 남아 있었다. 실제 Rest mutation을 semantic Psychology event로 이관하고 dormant surface를 제거했다.

## 정책 질문

현재 시작을 막는 질문은 없다. 보류된 콘텐츠 수치가 필수로 드러나면 임의 결정하지 않는다.

## 외부 시스템과 직접 의존성

- Core roster/checkpoint/battle entry snapshot
- game_server shared DTO transport
- Unity roster/result/presentation reader

## Dirty Worktree 주의사항

- employee_trust, employee, battle/world/snapshot와 Unity roster 파일은 선행 Phase 및 사용자 변경과 겹친다.
- 최신 파일 내용을 기준으로 patch하고 선행 Phase contract를 보존한다.

## 접근하거나 검증하지 못한 소비자

- 배포된 외부 durable save/database
- workspace 밖 비공개 client

## 적용하지 않은 대안과 이유

- personality를 Psychology 안에 복제하지 않는다.
- trust band를 mutable field로 저장하지 않는다.
- opaque string cue append-only compatibility를 남기지 않는다.

## 생략한 검증과 근거

- live RON migration은 실제 Psychology authoring이 없어 생략했다.
- explicit live-server PlayMode 4개는 외부 server 의존 테스트라 ordinary 57개와 분리했다.

## 남은 위험

- 배포된 외부 durable save가 있다면 no-compatibility cutover와 별도 migration 승인이 필요하다.
- 후속 high-risk/refusal/dialogue content는 새 event kind/source와 명시 정책을 별도 Goal에서 authoring해야 한다.

## 범위 밖 후속 후보

- trust high-risk/refusal/modifier/dialogue 콘텐츠
- account memorial/legacy

## 최종 판단

- `EmployeePsychology`가 run-local trust와 memory의 유일한 authority다.
- source dedupe ledger는 memory expiry 뒤에도 남아 과거 사건의 재적용을 막는다.
- Profile personality, condition/death, recruitment identity는 선행 Phase authority를 복제하지 않는다.
- Core/server/Unity exact projection과 broad regression 증거가 완료 조건을 충족한다.
