# 프로젝트 지도 제작 — 완료 문서 검토

검토 상태: `complete`

최종 verdict: **M01~M10 충족 — 프로젝트 지도 제작 완료.**
검토일: 2026-09-06 (Asia/Seoul). 대상: [project_map.ko.md](../../project_map.ko.md).

## 검토 범위

[PLAN](PLAN.md)의 M01~M10을 기준으로, 지도 작성 뒤 1~7절 전체를 별도 읽기 pass에서 읽고
중요 주장과 코드·데이터·정책·계약의 원문을 다시 대조했다. 별도 에이전트는 사용하지 않았다.
프로젝트 구현의 correctness 감사나 신규 정책의 최종 설계 승인이 아니다.
테스트·빌드·probe는 실행하지 않았으며 Unity 계약·클라이언트 구현은 읽지 않았다.

검토 범위는 지도 주장·근거·탐색 경로·그림·정책 구분·쓰기 제약이다. 발견한 불일치와 미결 정책을
현재 구현에 맞춰 해결하는 작업은 하지 않았다. 데이터 기반 콘텐츠 제작의 정의·조합·공통 실행·확장 위치와
후속 AI의 역할을 확인했으며, 전체 표현력 평가·새 구조 설계·재작성 권고를 완료 근거로 요구하지 않았다.

## 기준과 재확인한 원문

- Git root: `/mnt/f/work/simulator`; 조사·검토 HEAD: `f318414fb0156a67e04c5479000c2a2683fc0c09`.
- 지도 기준은 같은 HEAD와 당시 미커밋 작업 트리다. 관련 수정·미추적·삭제 범위와 전달 코드의 일치 조건은 지도 1절에 있다.
- 원문 재확인은 공개 application/transaction, 상태 owner/capture/restore, manual cast·impact·damage·death,
  wave/boss/terminal, 장착·성장·condition·reward/node commit, 서버 session/actor/store 경계에 걸쳐 수행했다.
- 정책/계약은 유물·침식 결정 기록, 성장/파편/Breakdown 룰북 관련 절, 데이터 publication·runtime codec 절을 대조했다.
  실제 읽은 경로와 수정 내역은 [EXPERIMENTS](EXPERIMENTS.md)의 실행 항목에 구분했다.

## 완료 조건별 근거

| 조건 | 지도 위치 | 직접 대조 근거 | 결과 |
| --- | --- | --- | --- |
| M01 목적·기준·범위 | 1절 | Git revision/status, 사용자 실행 지시, PLAN 범위와 대조. 설계 완료/구현/live 상태·코드 전달 기준·Unity/실행 제외 명시 | 충족 |
| M02 책임·의존·데이터 조합·진입점 | 2절, 4.1, 7절 | `CoreApplication`/private `GameCore`, embedded→validated publication, SkillDef/fragment/equipment/Boss의 정의→소비, Little Red binding·공통 task·Monolith domain 처리·미지원 compile을 대조 | 충족 |
| M03 상태 소유·수명 | 3절, 4.5, 4.7 | `GameCoreState`, `RunState`, `RunCheckpointPayload`, `BattleEntrySnapshot`, `BattleEmployeeAttemptState`, `RuntimeUnit`/`RuntimeItem`, proc/buff maps와 capture/restore/build/withdraw/condition 반영을 대조 | 충족 |
| M04 대표 흐름·실패·commit | 4.1~4.7 | 아래 일곱 흐름의 호출자·구현·변경·소비를 재확인. D01은 양쪽 소스의 가시성/호출 단절로 설명하며 가상 통합 경로를 만들지 않음 | 충족 |
| M05 신규 정책 연결 | 5절의 16행, 6절 | PLAN의 정책 관점 11개와 두 결정 기록을 행별 대조. 태그/연쇄/증폭/사본/관리/수명/성장/침식 시간/처치·직원/전투력·출력/획득 보류 모두 현재 연결·차이·질문에 대응 | 충족 |
| M06 사실·정책·추론 구분 | 1절, 3절 추론, 5~6절 | D01 양쪽 API, D02 condition·policy·node/checkpoint와 룰북, D03 codec와 두 계약, D04~D07의 확인 범위 대조. 승인 목표를 현재 동작으로 기술하지 않음 | 충족 |
| M07 후속 검토 목표·이해·원문 탐색 | 1절, 2.2, 7절 | 보스·장비·파편·유물 RON 제작 목표, 엔진 확장 가능성과 후속 표현력 판단, 유지/부분 재설계/전면 재작성 선택지, 목적별 읽기 순서·이유·상대 경로 확인 | 충족 |
| M08 경로·심볼·본문 정합성 | 전체 | 로컬 링크 157개/서로 다른 원문 파일 86개 존재. 원문 심볼·중요 분기 본문 재확인. Mermaid 3개와 본문·거부/commit 경계 대조, Markdown 구조 확인 | 충족 |
| M09 범위·제약·기존 작업 보존 | 1절, 실행 기록 | 실제 patch 대상은 다섯 산출물. Git 목록 source/data 233경로 hash와 산출물 외 status가 기준과 일치. 금지 실행·정책 변경·Unity 조사·압축/공개 없음 | 충족 |
| M10 별도 문서 검토 | 이 문서 및 EXPERIMENTS의 별도 읽기 기록 | 작성 후 지도 전체 읽기·원문 재대조·필수 조건 대조를 수행하고 아래 문구/그림을 수정한 뒤 다시 확인 | 충족 |

### 일곱 흐름의 핵심 대조

| 지도 흐름 | 직접 대조한 확정·실패·소비 경계 |
| --- | --- |
| 4.1 데이터 | `ParsedGameplayRon::into_validated` → candidate → generated preview → `ValidatedGameData` → setup/runtime. validation 실패 때 publication 없음 |
| 4.2 외부 요청 | 서버 session/actor dispatch와 현재 public port의 D01; `execute_application`의 현재 lifecycle→draft→도메인/검증→root 교체; 이후 `ApplicationResult::from_behavior`의 `AcceptedRecoveryPending` |
| 4.3 전투 구성 | `EquipmentLoadoutTransition` 양방향 변경, `SkillFragmentLoadout::equip_with_policy`·`apply_to_profile`, preview/setup validation, stat_pipeline·`spawn_scenario_group`의 runtime 소비 |
| 4.4 시전·연쇄 | manual start/end·Pause/tick, `invoke_ability`·step/deferred delivery, area/projectile impact→공통 명령, damage HP 확정→death trigger. `UnitDied` command는 무처리 |
| 4.5 직원·attempt | `roster()`의 active attempt 선택, 배치/철수·재배치 검증, runtime ID별 incap 반영, 결과 root 반영·새 attempt와 Retreat entry restore, 재접속 보존의 차이 |
| 4.6 전투 결과 | death epoch→wave/boss settlement→required group/terminal→employee_conditions와 CombatResult→ClaimAll grant·직원/조우/node commit; 패배·retry·failure decision |
| 4.7 저장·재접속 | SavePoint node 완료/회복→capture·V5 export→서버 publication 의도/D01; Load count·Continue/import·profile/peak 보존; Attach/Detach와 live resync/cursor의 별도 수명 |

정책 대응에서는 “같은 효과의 동일 종류 증폭”을 전체 산술 합산으로 확대하지 않는지,
순환 제한을 한 연쇄 **경로**의 동일 사본/효과 재방문으로 적었는지, 요구 태그 미충족과 비호환 거부를
구별했는지 확인했다. 무기만 강화·주문력·개화 선택, 침식의 실제 시전 대체·효과 없는 kill·정상 wave·직원 대가와
강제 연출 시간, 보류된 획득 정책을 현재 코드나 새 구현안으로 채택하지 않았다.

## 검토 중 수정과 최종 문서 확인

- 유물에 자체 성장 단계가 있는 듯한 비교 문구를 독립 사본의 소유·장착·효과 수명으로 수정했다.
  유물 자체 강화 없음, 전투 효과 범위와 방어구/액세서리 자체 효과 허용을 정책 요약에 명시했다.
- 상태 표의 active session을 실제 `Option<ActiveBattleSession>`로 수정하고 직원 투입 제한의 근거를 보완했다.
- death command를 kill 실행으로 오해하지 않도록 무처리 분기와 HP 경로를 설명했다.
- transaction 그림에 draft 이전 lifecycle 거부를 추가했다. root commit 뒤 공개 projection과 서버 API 단절은 유지했다.
- 태그 순환 제한을 경로 단위로 분명히 하고, 적 catalog는 실제 `GameDataBase.corroded_employee_data`로 탐색 가능하게 적었다.

수정 문구는 각각 정책 원문, `world/state.rs`, `employee.rs`, `commands.rs`, `world/application.rs`, `data/mod.rs`에 다시 맞췄다.
링크·상대 경로 존재, 심볼 탐색·주요 본문, 표 열 구조·fence 짝·줄 끝 공백·마지막 개행을 문서 확인으로 검사했다.
Mermaid는 3블록/6개 fence 표식이며 호출 의존·데이터 흐름·transaction 화살표를 구별한다.
renderer/패키지 설치나 runtime 실행으로 그림을 검증한 것은 아니다.

기존 변경 보존은 EXPERIMENTS의 hash/status 값으로 확인했다. 이는 기록한 Git 목록 source/data와 status 범위의 증거다.
다른 모든 파일의 byte 동일성이나 무시된 파일 전체를 증명한다고 확대하지 않는다.

## 검증과 제한 기록

- 사용자 보고: `all test`를 실행했을 때 모두 통과했다고 확인함.
- 이 Goal의 테스트·빌드·probe 실행: 사용자 지시로 제외.
- 테스트 수·pass/fail/skip: 해당 없음. 실행 결과를 생성하거나 재확인했다고 표현하지 않음.
- check·clippy·benchmark·서버 실행·fixture·설치도 수행하지 않음. 문서 검토는 읽기·diff/상태·경로·내용 확인으로 한정.
- 필수 원문 접근 불가·미조사로 남긴 M01~M10 항목: 없음. 수정이 필요한 지도 문구는 위 검토에서 반영함.
- 남은 미결정: 유물 콘텐츠/수치/획득·일반 보스 반복, 파편 기존 진행/저장 전환·개화 재료,
  침식 숫자/개별 연출, 전투력 가중 등. 지도 6절과 EXPERIMENT_NOTES에 열거함.
- 미확인·의도적 비범위: 실제 빌드/runtime/성능/배포·서버 통합·Unity 소비·전 콘텐츠 표현력·제작/배포 체계.
  D01을 미조사 공백으로 면제한 것이 아니라 양쪽 원문을 조사해 확인한 단절로 기록함.

## 최종 판정

`complete`. 승인된 지도를 작성했고 필수 조사와 별도 문서 검토를 마쳤다. 현재 구조·미구현 목표·명세 불일치·미결 정책이
원문 근거와 함께 분리되어 후속 AI가 필요한 소스를 선택할 수 있다. 이 판정은 현재 설계 승인·무결함 보증·정책 동결·
전체 콘텐츠 표현 가능성이나 서버 통합 성공 판정이 아니다. 별도 구현·새 Goal·압축/공개 작업은 수행하지 않았다.
