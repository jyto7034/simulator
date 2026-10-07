# U08 제품 구현·테스트 작성 역할 분리 인수

작성 시각: 2026-10-06T03:12:57.202412+00:00

최신 사용자 지시에 따라 기존 Core-first 연속 작업을 계속하며 제품 구현과 테스트 작성을 분리한다. U07/U07-M은 Complete / High / 열린 finding 0으로 완료됐고 U08은 GEX-10 수치 구현·집중 검증 중이다. 정책16수용군·10차원·191선행+별도31R06과 P2G 동결 기준은 바꾸지 않는다.

## 실제 인수와 현재성

- `HANDOFF_LATEST.ko.md`는 사용자 작성 2줄 안내다. 원문 “마지막에 하던 것을 마저 진행하라.”를 현재 PLAN·receipt·감사와 대조했다. 원 파일은 변경하지 않았다.
- 인수 시 product409 / SHA256 `591ed14f57d8243752bf4960fa443f81d40f414f06881cef3f98ad154e401568`, fmt006 source_after 전체 구성원과 구현자의 actual current가 일치했다. 이는 다음 테스트 편집 전 경계이며 이후 source를 같은 값으로 부르지 않는다.
- `relic_numeric_focused002`: exit101/94.72844s, 27 PASS/1 fixture FAIL, 당시409/2f40a42b… source 불변. 원 catalog의 constant overflow 거절 때문에 runtime guard에 도달하지 못했다.
- 이후 적용된 delta는 `core_enhanced/src/game/battle/core/sim.rs`와 `core_enhanced/src/game/battle/core/value_expr_runtime/tests/relic_amount.rs` 2파일이다. 전자는 source owner 소실 뒤에도 capture cold binding을 carrier에 보존하는 제품 교정, 후자는 dynamic overflow/withdrawal 시험이다. 모두 직전 Sol 작성 이력으로 보존하고 현재는 미검증이다. 기록 patch를 재적용하지 않는다.
- fmt006 rawexit0/2.17055s, intended source-format runnerexit99. receipt SHA256 `c6e925f0b500fe21a7a8f76fd985402692ee440f1821f5b7bfc18cc70fbce2d3`. focused002 receipt SHA256 `583ebf9233c778d01bba80724684f02b4d9bbc5bc78bbb9f3c48dce1a3542f42`.
- 마지막 유효 전체 focused 범위는 numeric 이전406/48ba52f1의 observation22/off20이다. 현재 numeric 전체 PASS로 확대하지 않는다. 준비된 PC successor는 미실행이다.
- Git index SHA256 `d12a26a2cdef3f40b8aaef940512611fdddb4c02ed902245fc49f5a5958f4749`; 현재 status는197 modified/27 deleted/103867 untracked 항목을 관측했다. 기존 변경을 인수했으며 정리·복원 명목 reset/clean/stash를 하지 않았다. 이는 전체 제품 파일 수와 다른 Git 상태 목록이다.
- 적용 AGENTS.md는 조상 경로와 rg 파일 검색에서 발견하지 못했다. 프로세스 가시성은 각 sandbox namespace에 한정된다. root 조회만으로 호스트 전체 worker 부재를 주장하지 않는다. 구현자가 자신의 실행 세션 종료/receipt 회수를 직접 회신했고 역할 인계 시 새 실행은 없었다.

## 역할과 실제 참여 이력

| 담당 | 모델·배정 확인 | 현재 권한 | 독립성 |
| --- | --- | --- | --- |
| root | 사용자 지정 gpt-6-astra/high 계획 역할의 현재 세션 | 요구·수용 기준·설계·순서·검증 목적·조정·계획 문서 | U08 설계 참여, 최종 감사 불가 |
| /root/u07_implementation | 기존 실제 gpt-6.1-sol/xhigh 배정(최초2026-10-05T08:53:34Z), 현재 역할 수락·owned-session 종료 회신 확인 | 제품/데이터/구현 문서/포맷/빌드/합의된 실행/원시 증거/제품 교정 | 기존 제품과 테스트 작성 이력 보존, 이제 테스트 작성 불가 |
| /root/u08_test_author | 이번 요청 후 gpt-6-astra/high를 명시한 spawn 성공 및 실제 역할 수락 회신 | 테스트/fixture/probe/기대값/oracle/comparator·테스트 교정, 합의 시 실행 | U08 테스트 작성 참여, 최종 감사 불가 |
| 최종 독립 감사자 | U08 비참여 기존 Astra/high 또는 지정 모델 새 배정의 실제 가용성을 최종 동결 시 확인 | 전체 Goal 감사·finding 반환 | 계획/제품/테스트 비참여 필수, 직접 교정 불가 |

원 감사자 이름만으로 현재 사용 가능하다고 간주하지 않는다. 현재 감사 pass는 시작하지 않았다. 과거 Sol 테스트를 Astra 작성으로 재표기하지 않으며 역할 변경 자체를 전체 재작성·재실행 사유로 삼지 않는다.

## 파일 소유와 첫 실행 계약

첫 테스트 보강 동안 제품 소스를 고정한다. 테스트 담당자가 `value_expr_runtime/tests/relic_amount.rs`를 독점 편집한다. 필요한 모듈 등록 `value_expr_runtime/tests.rs`, inline cfg(test)가 있는 `game/relic/numeric.rs`, host `game_server/src/observability/tests.rs`도 파일 전체 편집 소유를 순차 인수한다. 테스트 목적의 제품 API/실행 의미 변경 권한은 없다. 같은 파일의 제품/테스트 영역이 달라도 동시 편집하지 않는다.

구현자는 public numeric plan/preview, freeEvent 실제 권한, Sell 명시 거절, normal death/save 소비의 남은 제품 경계를 읽기 전용 조사한다. 테스트 담당자 준비 완료→파일 목록/기대 결과 인계→모든 입력 writer hold→Sol의 skip_children 포맷 및 새 focused 실행→원시 결과/receipt 회수→결함 분류 뒤 다음 변경 순서다. 테스트 담당자가 직접 실행할 때도 같은 전역 입력 동결 계약을 따른다.

테스트/fixture/판정 결함은 Astra, 제품 결함은 Sol, 요구·핵심 전제 변경은 root가 맡는다. 기대값부터 낮추지 않는다. 성능은 기능 worker 종료 뒤 새 isolate/label로 실행한다. 기본8MiB·PC576/36의31성공5거절·원래 hard guard·R05 기존 단독 경고 범위를 유지한다.

## 정확한 다음 행동

1. 기존 Sol6개 테스트를 보존하고 Astra가 dynamic Snapshot/Live, same-effect selector AND·정확 provider/form, runtime final narrow/rollback 반례를 필요한 만큼 추가한다.
2. 현재 제품2파일 미검증 교정과 새 테스트를 고정해 작은 집중 검증을 실행한다. 실패는 역할별로 교정하고 새 label로 확인한다.
3. 작은 numeric/host layout 경계가 안정되면 early PC를 수행한다. 정상 공개 queue/projectile/area 및 모든16수용군 증거는 이후 최종 검증에서 마무리한다.
4. evidence selection→impact→현재 adaptation→coverage→GEX10→문서→exact freeze/정역 복원→별도 독립 전체감사 순서를 유지한다. Complete/High/open0 뒤 다음 roadmap을 진행한다.

workspace와/tmp만 쓰며 networkrestricted/approvalnever 및 .git/.agents/.codex/.aws 읽기 제한을 우회하지 않는다. 현재 해결 불가능한 장애는 없고 사용자 재승인 대기 없이 계속한다. 중지·세션 인계·실제 장애 때는 사용자 요구에 따라 기존 HANDOFF_LATEST를 날짜·일련번호로 원본 보존한 뒤 최신본을 갱신한다.
