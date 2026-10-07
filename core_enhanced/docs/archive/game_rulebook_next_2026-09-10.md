> **보존본 — 2026-09-26 재구성 이전의 2026-09-10 초안.** 아래의 상태·미정·통합 여부는 당시 기록이다.
> 최신 차기 설계는 [게임 설계서](../game_rulebook_next.md), 이동과 후속 합의 반영은
> [통합 기록](../decisions/game_rulebook_next_restructure_2026-09-26.md)을 따른다. 상대 링크만 보존 위치에 맞춰 조정했다.

# Game Rulebook Next — 차기 게임 규칙 통합 초안

작성일: 2026-09-10. 정책 갱신: 같은 날 저지 판정용 타일에 관한 사용자 합의 반영.

**상태: 기존 룰북과 사용자 합의 정책의 통합 검토 초안. 현재 Gameplay canonical과 구현 지시는
[game_rulebook.md](../game_rulebook.md) 및 각 주제의 승인 계약을 따른다. 이 문서는 최종 동결·구현 완료·
canonical 교체를 선언하지 않는다.**

기존 룰북을 참조하는 작업을 유지하면서, 앞으로 적용할 게임 규칙을 한 문서에서 읽고 검토하기 위해
작성했다. 기존 규칙의 본문을 출발점으로 삼아 합의된 유물·성장·진행·저지 변경을 해당 장에 통합했다.
서로 연결할 정책이 아직 결정되지 않은 곳에는 그 경계를 명시했다.

## 문서 책임과 읽는 법

| 구분 | 내용 |
| --- | --- |
| 문서 등급 | 차기 gameplay 통합 검토 초안 |
| 소유 범위 | 이 초안에 통합한 게임 정체성, 런·노드·전투·성장·유물·손실 정책과 미결정 경계 |
| 소유하지 않는 범위 | 현재 canonical 교체, 실제 구현 상태 판정, RON/schema·DTO·세이브 설계, 구현 Goal과 이행 승인 |
| 기존 규칙 기준 | 2026-09-10에 읽은 작업 트리의 [game_rulebook.md](../game_rulebook.md) |
| 합의 변경 근거 | 아래 입력 문서와 후속 대화의 사용자 결정. 문서 날짜만으로 제안·예시를 승인으로 바꾸지 않음 |
| 함께 확인할 문서 | [skill_target_contract.md](../skill_target_contract.md), [core_runtime_contract.md](../core_runtime_contract.md), [data_loading_contract.md](../data_loading_contract.md), [engineering_workflow.md](../engineering_workflow.md), [README.md](../README.md) |

본문의 상태는 다음과 같다.

| 표시 | 의미 |
| --- | --- |
| 별도 표시 없는 기존 규칙 | 통합의 출발점으로 보존한 현행 룰북 내용. 이번 정리에서 새로 재승인하거나 runtime과 대조 검증한 것은 아님 |
| 합의 변경 | 입력 문서에서 사용자 결정으로 기록한 목표 정책을 반영. 구현 완료를 뜻하지 않음 |
| 미정 / 재검토 | 결정되지 않았거나 합의 변경과 기존 규칙 사이에 추가 결정이 필요한 부분. 임의의 기본값으로 채우지 않음 |
| 기존 기준 / 현행 비교 | 재설계 중인 영역의 이전 규칙을 보존한 것. 해당 영역의 새 설계를 확정하는 근거로 쓰지 않음 |

각 장의 합의 변경이 그 장의 기존 기준과 맞닿는 곳에서는 표시된 미정 경계까지 함께 읽는다.
특히 필수 관문, 재료 경제, 파편 획득과 반복 조우의 연결은 아직 하나의 실행 가능한 최종 정책으로
동결되지 않았다. 미정 표의 후보를 구현하거나 기존 동작을 자동 제거하는 지시로 읽지 않는다.

본문에 남긴 `현재 live`, bootstrap ID, 수치, schema·command 이름과 구현 상태 설명은 기존 룰북에서
물려받은 기준 시점의 설명이다. 새로 추가한 유물·주문력·개화 형태 선택까지 구현됐다는 뜻은 아니다.
정확한 구현 계약은 인접 canonical이 소유하며 목표 변경과의 차이는 후속 구현 검토에서 다룬다.

세부 계약의 위치:

- 타겟팅·범위·자동 시전: [skill_target_contract.md](../skill_target_contract.md).
- 전투 상태·결정론·event·checkpoint·결과 확정: [core_runtime_contract.md](../core_runtime_contract.md).
- RON authoring·loader·schema·validation: [data_loading_contract.md](../data_loading_contract.md).
- Unity command·snapshot·result: 외부 `/mnt/f/unity projects/ark/docs/unity_core_contract.md`.
- Unity 전투 시작·update·resync·종료: 외부 `/mnt/f/unity projects/ark/docs/core_unity_battle_transport_contract.md`.

## 통합 입력과 결정 출처

| 입력 | 통합한 내용 | 취급 |
| --- | --- | --- |
| [기존 룰북](../game_rulebook.md) | 게임 흐름, 탐사, 전투, 보상·성장, 영구 profile, 결과·실패 | 기존 기준 |
| [유물 시스템 설계 노트](../tmp/run_relic_system_design_notes_2026-08-21.ko.md) | 무기·파편 성장, 공용 3슬롯, 범용·전용 유물, 태그·연쇄, 희귀도, 침식된 유물, 보존과 표시 | 2026-09-08까지 기록된 사용자 결정과 명시적 미정 사항 |
| [런 진행·성장 정책 기록](../tmp/run_progression_growth_policy_notes_2026-09-07.ko.md) | 필수 Elite·NormalBoss, 선택 일반 전투, 강화 재료 획득 목표, 직원·층수 검토 범위 | 초기 제안과 다른 부분의 최신 사용자 결정 |
| [전면 재설계 제안 v1](../tmp/run_progression_relic_redesign_proposal_2026-09-07.ko.md) | 검토할 대안과 초기 계산의 맥락 | 미승인 AI 제안. 문서 전체나 나머지 항목을 자동 채택하지 않음 |
| 2026-09-10 저지 규칙 대화 | 최초 접근·통과 중 저지의 판정용 타일, 연출 위치와 판정의 분리, 저지 상대 기본 공격 | 사용자 합의. 정확한 저지 범위·거리 해제 기준·복수 신규 후보 우선순위는 미정 |

이 초안은 위 네 문서로 시작했으며, 이후 저지 규칙 대화의 합의를 추가 반영했다.
연결된 환상체별 파편·장비·유물 콘텐츠와 침식의 상세
발동 정책 전체를 새로 검토·동결한 것은 아니다. 해당 내용은 소유 문서를 연결하고, 위 입력에서
확인되는 합의와 미정 상태만 요약한다.

기존 룰북 기준본 SHA-256: `482e964cc7015945dec29cf28293d58d9e315c8db47ef60e0f4765e3f1a36c6d`.
기존 파일의 이후 편집을 이 초안에 자동 반영하지 않는다. 다음 갱신 때 변경 구간과 합의 상태를
확인해 반영하며, 기존 룰북·원본 메모·소비자 참조 경로는 이번 정리에서 변경하지 않는다.

## 통합된 주요 변경

| 주제 | 차기 목표 정책 | 남은 경계 |
| --- | --- | --- |
| 전투 진행 | Elite·NormalBoss는 필수 관문, 일반 침식체는 건너뛸 수 있는 선택 전투 | 층별 배치, Gate 해금, 재도전·패배 소진, 모노리스·소탕과의 연결 |
| 선택 전투 보상 | 일반 침식체 전투를 선택해야 무기 강화 필수 재료를 얻음 | 분쇄·상점·시작 보급 등으로 이 조건을 우회하지 않는 재료 경제 |
| 일반 강화 | 무기의 제공 공격력·주문력 성장. 방어구·액세서리·파편·유물은 일반 강화 없음 | 비용·수치·강화 한도, 기존 강화 상태 전환 |
| 파편 개화 | 형태 해금 후 기본·개화 형태를 모두 유지하고 Maintenance에서 선택 | 개별 recipe, 추가 환상체 재료, 저장·조건 판정 |
| 유물 | 직원당 공용 3슬롯, 범용·전용 효과와 연쇄, 사본별 소유 | 획득 정책, 개별 콘텐츠·수치, 실제 효과 구성과 전투력 평가 |
| 침식된 유물 | 침식 아이템을 기존 유물에 적용해 강한 효과와 생존 위험을 함께 부여 | 적용·해제·재적용·보존과 개별 패널티 |
| 저지와 좌표 | 적은 저지 시점의 판정용 타일을 유지. 연출 위치 보정으로 타일을 바꾸지 않고 저지 관계로 기본 공격 대상 판정 | 정확한 범위·타일 경계·거리 해제·복수 신규 후보 선택과 기존 targeting/runtime 계약 반영 |

## 탐색

- [게임 개요](#게임-개요)
- [탐사와 노드](#탐사와-노드)
- [전투](#전투)
- [성장, 스킬 파편과 도감](#성장-스킬-파편과-도감)
- [유물](#유물)
- [비전투 노드](#비전투-노드)
- [결과와 실패](#결과와-실패)
- [고급 정책과 레거시](#고급-정책과-레거시)
- [미정 사항과 미승인 제안](#미정-사항과-미승인-제안)

## 게임 개요

이 장은 플레이어가 접하는 게임의 정체성, 모드, 런의 기본 순서와 시작 roster 구성을 소유한다.
command field나 runtime 저장 구조는 의미를 설명하기 위한 seam이며 정확한 구현 계약은 인접 canonical을 따른다.

### 게임 정체성

이 게임은 환상체를 소유 기물로 쓰는 게임이 아니라, 소수의 직원을 이끌고 봉쇄된 시설을 탐사하는 로그라이크 전술 전투 게임이다.

핵심 판단은 아래 네 가지다.

- 어떤 직원과 장비, 스킬 파편, 유물 조합으로 다음 노드에 들어갈지 정한다.
- 노드 브리핑과 위협 경고를 읽고 배치와 전투 계획을 세운다.
- `DefenseRoute` 전투 중 전술 포인트 기반 배치, 철수, 수동 스킬, 후퇴, 일시정지, 배속을 조작한다.
- 보상, 무기 강화·파편 개화·유물 조합, 직원 손실 위험을 관리하며 다음 노드를 고른다.

현재 공식 전투 축은 명일방주식 실시간 배치 방어인 `DefenseRoute` 하나다. 보스 전투도 별도 전투 장르를 되살리기보다 `DefenseRoute` 파생으로 설계하는 방향을 우선 검토한다.

### 게임 모드

> **기존 기준 / 모드 재설계 미정:** 아래 Standard·Endless 구조를 비교 기준으로 보존한다.
> 6층 본편·선택 연장, 8층 비교안, 튜토리얼 축소, FinalBoss 배치와 런 종료 방식은 아직 채택되지 않았다.
> 새 필수·선택 전투 구분의 Standard 적용 범위와 모드별 배치도 후속 결정 대상이다.

게임은 상위 구조로 `Standard`와 `Endless`를 가진다. 두 모드는 같은 전투·직원·장비·스킬 파편
기반 규칙을 사용하지만 서로 독립된 run이며 목적, 맵 생성과 진행 정책이 다르다.

Runtime은 런 생성 시점부터 `GameMode`를 명시적으로 저장해야 한다. `GameMode`는 `StartNewGame`
command의 필수 입력이며, 직원 선발 완료 후 생성되는 `RunProgression`에 그대로 저장된다. `GameMode`는
노드맵 생성, `Gate` 배치, 다음 Floor 전환, 클리어/실패 조건의 source of truth다. 모드별 규칙을
generator나 UI 추론으로 흩어 두지 않는다. 모드 선택 화면에는 Endless를 처음부터 표시하지만,
영구 profile의 튜토리얼 완료 플래그가 없으면 진입을 거부하고 완료 조건을 안내한다.

Standard:

- 본편에 들어가기 전 게임의 대략적인 흐름과 조작을 알려 주는 고정형 튜토리얼이다.
- 랜덤 생성 Floor를 사용하지 않고, 세 Floor 모두 미리 작성된 고정 맵과 진행 순서를 사용한다.
- 전투 난이도도 튜토리얼 RON이 작성한 고정값을 사용한다. Certified 전투력, run peak나 Endless
  Floor anchor로 적 스탯·웨이브·보상을 재조정하지 않는다.
- Floor 1의 핵심 전투는 일반 침식체 `Normal` node다.
- Floor 2의 핵심 전투는 `Elite` 환상체 node다.
- Floor 3의 최종 관문은 간소화된 `NormalBoss` 환상체 node다. `FinalBoss`는 출현하지 않는다.
- Floor 3의 일반 보스를 승리하면 Standard run을 완료하고 영구 튜토리얼 완료 플래그를 기록해
  Endless 진입을 해금한다.
- 튜토리얼 도중 authored script가 임의의 스킬 파편을 지급해 장착·사용·성장을 체험시킬 수 있다.
  지급 시점과 정확한 fragment ID는 튜토리얼 RON이 명시한다. 이 지급은 Endless의 정규 파편 획득
  규칙이 아니며 Standard run 밖으로 전달되지 않는다.
- 튜토리얼 파편도 일반 파편처럼 즉시 분해할 수 있다. 필수 학습 전에 분해해 이후 진행 조건을
  만족하지 못하면 자동 재지급·보호·rollback 없이 Standard를 실패하고 새 run으로 재시도한다.
- Standard의 Elite/Boss 승리는 Endless의 격파 횟수, 숙달, 모노리스와 정규 파편 획득 조건에
  포함하지 않는다.
- 장기 보스 징조, 모노리스 반복 성장, Elite pool 순환과 소탕은 Standard 콘텐츠가 아니다.
- Standard 완료 뒤에도 모드 선택에서 다시 시작할 수 있다. 반복 완료는 추가 전역 보상을 지급하지
  않지만 실제 조우·격파한 침식체와 환상체는 일반 규칙대로 전역 도감에 기록한다.

Endless:

- Standard 완료 뒤 진입하는 본편이며 장기 생존과 심화 콘텐츠를 담당한다.
- Floor가 계속 이어지며, 플레이어는 누적되는 손실과 보상을 관리한다.
- 특정 보스가 `provisional_boss`로 떠오를 수 있고, 기존 노드 위에 보스 징조 source가 얹힐 수 있다.
- 플레이어가 같은 보스 계열의 징조 source를 반복 선택하면 보스 체인이 확정되거나 진행도가 누적될 수 있다.
- 조건이 완성되면 보스 방이 생성되거나, 일부 체인은 다음 노드 선택지를 잠그는 강제 보스 방으로 이어질 수 있다.
- 다음 층 이동 source는 `Gate` 노드다. 기존 facility 연결은 Gate의 부모로 Elite·Boss·일반 node를
  허용한다. 차기 진행에서는 일반 전투를 필수 관문의 유일한 통로로 만들 수 없으며, 필수 관문과
  Gate의 정확한 연결·해금 조건은 아래 필수·선택 전투 정책과 함께 재설계한다.
- 기본적으로 한 층에는 `Gate`가 하나지만, 특수 층에서는 여러 `Gate` 후보를 허용할 수 있다.
- 조건부/Locked Gate, 비밀 Gate, 완전 숨김 Gate는 시스템상 미래 확장 후보지만 기본 live data에 임의로 작성하지 않는다. 그런 노드는 명시 정책이 있을 때만 추가한다.
- 고요한 오케스트라 같은 장기 보스 기믹, 특수 보스 파편, 고위험 보상과 누적 리스크는 끝없는 탐사 콘텐츠다. 백야의 상세 체인은 향후 정책부터 다시 승인할 설계 후보다. 개발 중 generic BossOmen end-to-end 검증에는 현재 one-step `white_night` chain과 `suppress_white_night`를 유일한 live bootstrap FinalBoss로 사용하며, 이는 최종 백야 콘텐츠 승인이 아니다. 이 하나를 격파하면 그 시점의 live 집계 대상 FinalBoss 전체 격파 기록을 달성한다.
- 숙달한 Elite, NormalBoss와 FinalBoss의 각 반복 pool은 아래의 cycle 규칙으로 계속 순환한다.
- 자동 `RunComplete`는 없다. 모든 FinalBoss와 숨겨진 보스를 격파해도 Floor를 계속 생성하며,
  플레이어의 자발적 포기 또는 복구할 수 없는 Run 실패만 Endless를 종료한다.

### 핵심 플레이 루프

기본 런 흐름은 아래 순서다.

```text
런 시작
-> 시작 직원 선발
-> Floor 맵 생성
-> 노드 선택
-> 노드 진입 전 확인과 준비
-> 전투/Event/저장/정비/상점/보상/본사 연락 처리
-> 노드 결과 반영
-> 다음 노드 선택
-> 모드별 종료 조건 확인
```

Standard에서는 고정된 세 Floor를 순서대로 학습하고 Floor 3의 간소화된 일반 보스를 클리어하는
것이 종료 조건이다. Endless에서는 `Gate`를 통한 다음 Floor 이동, 보스 징조 체인, 누적 손실 관리와
숙달 encounter의 반복 성장이 핵심 진행을 만든다. Endless는 자동 완료 없이 Floor를 계속 생성한다.

전투 노드는 한 번의 전투로 끝나지 않을 수 있다. 전투 전체 후퇴는 BattleEnd/패배 판정 전에
진입 상태를 복원하며 남은 시도를 줄이지 않는다. FinalBoss 패배 결과 확정은 기존 공통 실패 선택
정책을 따른다. 필수 Elite·NormalBoss의 시도 소진 뒤 진행 처리는 새 관문 정책과 맞춰 결정한다.

### 시작 직원 선발

런 시작 시 플레이어는 시작 직원 후보 중 출전할 직원을 고른다.

> **인원 미정:** 직원 5~8명은 유력한 검토 범위다. 보유 상한·전투 등록 인원·동시 배치 상한·예비
> 인력을 구분해서 결정한다. 시작 인원·채용 시점도 미정이며, 아래 starter 후보 6명은 출전 인원이나
> 보유 상한을 뜻하지 않는다. 초기 AI안의 보유 5명·등록 4명을 확정값으로 쓰지 않는다.

- 모든 시작 후보와 본사 채용 후보는 하나의 authored `EmployeeDefinition` catalog에서 온다. `starter_eligible`은 시작 후보 자격이며, 시작 때 선택하지 않은 starter definition과 모든 non-starter definition은 같은 런의 미합류 채용 queue에 들어간다.
- `employee_definition_id`는 콘텐츠에서 안정적인 authored identity다. roster 합류 뒤 생성되는 `uuid`는 해당 런 안의 직원 instance identity이며 둘을 서로 대신 사용하지 않는다.
- definition은 이름, 역할, 배경, initial loadout, personality의 risk/social 두 축, `CombatAptitude`, 기본 HP/공격/방어/마법 저항, `basic_attack_speed_percent`를 명시한다.
- risk disposition은 `Brave`, `Cautious`, `SurvivalInstinct`, social disposition은 `Obedient`, `Defiant`, `Comradely` 중 각각 정확히 하나다. personality는 UUID나 신뢰 상태에서 추론하지 않는다.
- `CombatAptitude`는 `Balanced`, `Defensive`, `Offensive`의 가시성 정보다. 현재 장비 제한이나 불일치 페널티를 만들지 않는다.
- 직원 기본 공격 속도 100은 중립이다. 무기가 기본 공격 주기를 소유하고 최종 주기는 `ceil(weapon_base_interval_ms * 100 / basic_attack_speed_percent)`다. 비무장 직원은 roster와 safe-zone 장비 관리에는 남지만 battle gameplay source가 될 수 없으므로 기본 직원 공격 주기로 보완하지 않는다.
- specialties는 현재 production schema/runtime/DTO에 없다.
- 미합류 직원 queue 순서는 run seed로 한 번 결정한다. 본사 연락은 queue head에서 authored 후보 수만큼 active session으로 옮기며 snapshot 재요청이나 재접속으로 다시 뽑지 않는다.
- 채용한 definition은 새 run-local UUID와 함께 roster로 이동하고, 선택하지 않은 후보는 제안된 순서 그대로 queue tail에 돌아간다. 응급지원이나 본사 상점을 선택하면 현재 후보 전부가 같은 순서로 tail에 돌아간다.
- 같은 definition은 queue, active recruitment session, roster 중 정확히 한 곳에만 존재한다. checkpoint load는 이 소유권과 queue 순서를 검증·복원하고 Floor 전환도 그대로 유지한다.
- starter-eligible 6명은 일반 보상 pool에 들어가지 않는 `standard_training_sword`와 각 authored initial
  loadout을 받는다. 시작 직원은 이후 런 전체의 핵심 자원이다.
- 직원은 장비, 스킬 파편, 유물, run-local Psychology, physical damage, trauma를 통해 장기적으로 관리된다.
- 무기를 장착한 출전 가능 직원이 없고 실제로 출전 가능성을 복구할 경로도 남아 있지 않으면 런 실패다.
  무기를 지급하지 않는 본사 채용은 그 자체로 비무장 roster를 복구하는 전투 경로가 아니다.

## 탐사와 노드

이 장은 플레이어가 Floor를 탐색하고 node content를 선택·해결하는 규칙을 소유한다. Snapshot field,
RON field와 validation 세부는 각각 외부 Unity canonical, [data_loading_contract.md](../data_loading_contract.md),
[core_runtime_contract.md](../core_runtime_contract.md)의 경계 요약으로만 읽는다.

### 필수 관문과 선택 전투

**합의 변경 — 진행·성장 정책 기록.**

| 전투 종류 | 진행 역할 | 선택과 보상의 의미 |
| --- | --- | --- |
| Elite 환상체 | 필수 관문 | 진행의 핵심 전투 |
| NormalBoss | 필수 관문 | 진행의 핵심 전투 |
| 일반 침식체 | 선택 전투 | 건너뛸 수 있지만 무기 강화 필수 재료를 얻으려면 선택해야 함 |

- 일반 전투를 완료하지 않아도 필수 관문에 도달할 수 있어야 한다. 지도 연결에서 일반 전투가
  관문의 유일한 통로가 되면 이 정책을 만족하지 않는다.
- 일반 전투 미참여를 이유로 필수 관문 입장을 잠그지 않는다. 선택 전투를 생략한 편성의 승리는
  보장하지 않으며, 강화 부족과 직원 상태를 보고 위험을 판단한다.
- 선택 전투는 추가 위험을 감수하고 이후 필수 전투를 위한 무기 성장을 준비하는 기회다.
- 매 층 Elite와 NormalBoss를 모두 배치한다고 정한 것은 아니다. 필수 전투 수와 층별 배치는 미정이다.
- Event·정비·상점 등 비전투 node의 존속과 기존 역할을 이 결정으로 삭제하지 않는다.
- FinalBoss의 수·배치·종료 구조, 선택 방 최대 2개 제한, 필수 관문의 재도전 수는 새로 확정하지 않는다.

**재검토:** 기존 모노리스의 선택형 Elite 재현과 숙달 소탕에 필수 관문 정책을 어떻게 적용할지,
필수 관문의 패배 횟수 소진을 어떻게 처리할지, Gate를 언제 열지는 미정이다. 기존 pool·시도 규칙을
복사한 것만으로 이 연결이 해결됐다고 보지 않는다.

### Floor 맵과 노드 선택

Floor 맵은 봉쇄된 시설의 방과 복도를 탐사하는 지도다. 플레이어는 임의 node로 순간이동하지 않고,
core가 현재 위치와 authored facility connection으로 계산한 도달 가능한 node만 선택한다.

- Standard와 Endless는 같은 facility engine을 사용한다. Standard는 Floor별 고정 authored
  template·assignment를 사용하고 랜덤 pool에서 맵이나 핵심 encounter를 뽑지 않는다. Endless는
  Mode/Floor별 live schedule이 template pool과 assignment profile을 고르며, pool의 deterministic
  순서·cycle·사용 이력은 run/checkpoint 상태다.
- Facility template는 semantic room slot, slot별 허용 category, tag, typed terminal과
  `Bidirectional`/`ForwardOnly` connection을 소유한다. Runtime은 별도 node edge graph를 저장하거나
  depth/lane 좌표로 topology를 재생성하지 않는다.
- Floor 생성 시 모든 slot의 category, kind/content와 seed를 한 번 확정한다. Profile의
  min/max/exact와 slot allowed category를 만족하는 완전 배정이 없으면 category repair나 구 generator
  fallback 없이 content error다.
- Node는 Combat, SavePoint, Maintenance, Shop, Reward, HeadquartersContact, Boss, Gate 등의
  단일 category를 가진다.
- Start는 실제 node이며 Floor 시작 시 `Completed + Revealed` 상태와 현재 위치가 된다.
- 완료 node는 content를 다시 실행하지 않는 transit 지점으로 사용할 수 있다. Core는 완료된
  transit path와 connection direction을 반영한 최종 `map_navigation.selectable_node_ids`를 투영한다.
- Connection은 `Bidirectional` 또는 `ForwardOnly`다. Unity는 graph와 선택 가능성을 재계산하거나
  transit 이동을 별도로 연출하지 않고 core가 허용한 목적지를 즉시 선택한다.
- Combat/Boss 선택은 `NodeConfirm` preview를 열 뿐 현재 위치를 바꾸지 않는다. 실제 진입과
  node 소비는 출전 가능한 직원이 한 명 이상인 `ConfirmEnterNode` 성공 뒤 확정된다.
- 직원 카드 표시 순서는 Unity-local 설정이며 core gameplay, checkpoint와 battle retreat의
  복원 대상이 아니다.

Node state와 visibility의 gameplay 의미:

- `Available`: 선택 가능한 미해결 node.
- `Completed`: 해결됐고 transit으로만 사용할 수 있는 node.
- `Locked`/`Unavailable`: 조건 또는 현재 progression 때문에 선택할 수 없는 node.
- `Revealed`: 종류와 preview를 표시하는 기본 상태.
- `Obscured`: 종류·보상·상세를 숨길 수 있지만 core가 selectable로 투영하면 선택 가능하다.
- `Concealed`: Unity에서 숨기며 선택할 수 없다. 명시적인 비밀방/조건부 기믹에만 사용한다.

Gate는 다음 Floor로 이동하는 독립 node다.

- Elite/Boss가 Gate 역할을 겸하지 않는다. Standard Floor 1~2의 Floor 전환과 Gate 위치·해금 조건은
  고정 튜토리얼 맵이 직접 작성한다.
- Gate는 보이는 것과 선택 가능한 것이 다르다. Unity-local 확인을 거친 확정 command에서 core가
  현재 선택 가능성을 다시 검증한다.
- Gate는 SavePoint가 아니다.
- 전환 시 생존 직원의 trauma를 RunPolicy `gate_trauma_recovery`에 따라 줄인다. Live 값은
  `CurrentPercent { percent: 10 }`이며 현재 trauma의 10%를 올림한다. Critical과 Breakdown도
  생존 직원으로 포함하고 Dead는 제외하며 physical damage와 checkpoint는 바꾸지 않는다.
- Floor가 바뀌어도 roster condition, 장비, 스킬 파편, 유물 보유·장착, 소비 아이템과 resource는 유지된다.

시설 지도 좌표는 core의 `map_template_id + slot_id`와 Unity-local template가 연결한다. Slot이
없을 때 임의 좌표로 fallback하지 않는다. 차단 구역·오염·경고 같은 분위기 표현은 선택과 진행에
영향을 줄 때만 core node가 되고, 그 외에는 Unity template/art가 소유한다.

조건부 보너스 방은 Floor 생성 때 slot/category/content/seed까지 미리 확정된 `Concealed` node다.
Authored prerequisite slot들을 순서와 무관하게 모두 성공 완료한 transaction에서만 같은 map의
authored connection과 함께 한 번 `Revealed`로 발현한다. Retreat, 실패, 취소와 단순 방문은 발현을
일으키지 않으며 reconnect/checkpoint load는 content를 재추첨하지 않는다. 발현 방은 선택형 콘텐츠만
허용하고 Gate, SavePoint, 필수 Boss 같은 진행 필수 room은 대상이 될 수 없다.

### Certified 전투력과 Floor 난이도

**합의 변경 — 유물 평가:** 현재 활성화된 유물 효과를 Certified 전투력에 포함하고, 이후 Endless
Floor 난이도 결정에도 반영한다. 조건부 효과·태그 변형의 평가와 가중치는 미정이다. 공격력·주문력
분리와 파편 계수도 실제 모델에서 검토해야 한다. 아래 기존 certified model의 version·입력 목록이
이 목표 변경까지 이미 지원한다고 보지 않는다. 초기 AI안의 전투력 추종 난이도 제거는 미승인이다.

- 이 절의 적응형 Floor 난이도는 Endless에만 적용한다. Standard는 certified 전투력을 readiness
  표시에 활용할 수 있지만 authored 튜토리얼 전투의 난이도에는 합성하지 않는다.
- Core는 지금 전투에 진입할 수 있는 직원(`alive && physical_damage <= 4 && trauma < 100`)의 level,
  영구 combat stat, 장착 장비/강화, 장착 skill fragment, 공격 주기, block과 공격 범위를 기존 입력으로
  사용한다. 차기 모델은 위 활성 유물 효과와 새로운 성장 능력치를 포함하도록 검토하며 versioned
  certified power를 계산한다. 이 경계 안의 현재 HP 및 physical damage/trauma 크기,
  active consumable과 전투 중 임시 효과는 raw power 계수가 아니라 readiness 경고에만 반영한다.
  Critical, Breakdown, Dead 직원은 합산하지 않으며 roster의 eligible power가 0이면 상대 위협은
  `VeryHigh`다.
- 첫 Floor anchor는 starter/loadout commit 뒤 certified power와 authored 최소값의 최댓값이다. 이후
  실제 Combat 진입 때만 run peak를 올리고, 새 Floor anchor는 authored 최소값, 이전 anchor + 최소
  상승분, run peak의 최댓값으로 한 번 확정한다. 같은 Floor의 편성 변경은 node를 재스케일하지 않는다.
- Ordinary Combat는 Floor 생성 transaction에서 Easy/Standard/Hard quota를 결정론적으로 나눈다.
  Elite/NormalBoss/FinalBoss/BossOmen/MonolithRemanifestation은 ordinary quota와 분리된 role이다.
- 확정 난이도는 target power, enemy stat, generated-wave budget과 수량형 reward budget을 함께 소유한다.
  Hard와 특수 role의 reward multiplier는 Standard보다 커야 한다.
- Unity에는 exact certified/anchor/target/multiplier를 공개하지 않는다. 확정 band, 현재 certified 대비
  상대 위협과 Trauma/physical damage/출전 가능 인원 readiness만 표시한다.
- Run peak는 checkpoint load, retreat, restart와 active-run backup 복구로 감소하지 않는다. Current Floor
  anchor와 power model version은 checkpoint의 map/node difficulty와 같은 세대로 저장·복원한다. 같은
  checkpoint에서 같은 Gate를 재진행하면 같은 run peak/live policy에서 같은 다음 anchor를 만들며 load가
  상승분을 누적하지 않는다. 구 Floor-number scaling과 overflow clamp는 authority가 아니다.

### Safe Node와 Safezone

`Safe Node`는 맵에서 선택하고 한 번 소비하는 비전투 node이며 `Safezone`은 node 사이의 준비
UX다.

- Safe Node 계열은 SavePoint, Maintenance, HeadquartersContact, Shop, Reward다.
- SavePoint는 선택지를 고르는 중간 mode 없이 `SavePointState`와
  `selected_event.type == "save_point"`를 쓰는 독립 통과형 node다. Maintenance도 별도 상태를
  쓰는 독립 node다.
- Safezone은 node를 소비하거나 보상·회복·정비 효과를 자동 지급하지 않는다.
- Safezone에서는 node 탐색, 소비 아이템 사용과 원자적 loadout 교체, 유물 장착 변경을 할 수 있다. 장비 이동과
  같은 type slot 교체는 전부 성공하거나 전부 실패하며 장비를 소비·조합하지 않는다.
- Maintenance가 무기 강화, 파편 개화·형태 선택과 무기·방어구·액세서리·파편·유물의 허용된
  분쇄를 소유한다. 파편·방어구·액세서리·유물의 일반 강화는 없다.

### Event Node

Event는 ordered illustration/text panel, 직원 판정과 분기 결과를 소유하는 독립 node category다.
Global Visual Novel이나 다른 node category의 fallback이 아니며 Standard와 Endless 기본 Floor에
최소 하나가 생성된다.

- Core는 event/scene identity, ordered panel id, interaction, eligibility, committed check와 진행
  상태를 소유한다. Unity는 image/text asset과 panel cursor만 로컬로 처리한다.
- 선택지가 없는 scene도 command로 진행하며 Unity가 다음 scene이나 eligibility를 계산하지 않는다.
- 직원은 저장된 Profile에 Medical, Security, Analysis, Reconnaissance, Containment,
  Maintenance 중 Event aptitude를 정확히 하나 가진다. 선택지의 aptitude와 일치하면 +3이고,
  성격·신뢰·기억·컨디션 수치·전투 스탯은 범용 보정치가 아니다.
- checked choice가 `aptitude`, DC 8/12/16/20과 optional key를 소유한다. 같은 aptitude에 적용되는
  현재 직원의 장착·소유 장비 중 가장 높은 +1/+2 하나만 더하며 동률은 장비 instance UUID
  오름차순이다. Inventory-only 장비는 적용하지 않고 key는 소비하거나 보정치를 암시하지 않는다.
- Core는 Event session의 choice/performer와 독립적인 pure 1d20을 한 번만 확정한다. Natural 1은
  critical failure, natural 20은 critical success이며 2..19는
  `raw_roll + employee_modifier + equipment_modifier >= dc`면 성공한다.
- 확정한 choice, performer, raw roll, typed modifier/key provenance와 outcome은 exact duplicate,
  reconnect/checkpoint/retreat 뒤에도 바꾸거나 다시 굴릴 수 없다. Invalid identity, condition,
  missing key와 stale preview는 상태·RNG를 변경하지 않는다.
- 실패는 command error가 아니라 authored failure scene이며, 모든 reference와 effect는 검증 뒤
  원자적으로 commit한다.
- 한 Event에는 reachable employee-check scene이 최대 하나다. 현재 첫 live checked Event는
  `sealed_archive_recovery_01`이고 Analysis DC16/key/80 Enkephalin과 Maintenance DC12/no-key/30
  Enkephalin 선택을 제공한다. `white_night_confession_01`은 advance-only다.

Event choice는 기존 combat pipeline을 시작할 수 있다.

- Event category를 Combat으로 바꾸지 않고 choice가 명시한 encounter를 사용한다.
- 승리하면 Event가 완료된다. 패배 결과를 commit하면 해당 Event 전투는 즉시 실패로 종료되고 같은
  Event에서 재시도할 수 없다. 결과 확정 전 battle retreat는 공통 rollback 규칙을 따른다.
- Event source를 사용하는 `BossOmen` 전투는 이 일반 Event 완료 규칙을 상속하지 않는다. 첫 omen과
  active chain 후속 omen의 실패 처리는 아래 `FinalBoss pool과 BossOmen Chain` 절이 소유한다.

### 일반 인벤토리

일반 인벤토리는 장비 instance와 소비 아이템 definition stack을 보유하며 gameplay admission cap이
없다.

- 장비 instance 하나와 서로 다른 소비 아이템 stack 하나가 각각 한 슬롯을 점유한다. 재화,
  장비 재료, 스킬 파편과 파편 가루는 포함하지 않는다.
- 유물은 사본별 런 공용 보유와 보유량 무제한 정책을 따른다. 일반 인벤토리의 표시용 슬롯 집계에
  유물을 포함할지는 미정이며, 현재 장비 instance나 소비 아이템 stack으로 임의 분류하지 않는다.
- 소비 아이템은 수량 상한 없는 `definition_id` stack이며 수량 증가는 새 슬롯을 요구하지 않는다.
- `reserved_slot_count`는 50에서 시작해 새 슬롯 commit이 경계를 넘을 때 10단위로 증가하고
  제거 시 줄지 않는 표시용 high-water 값이다. Collection capacity나 overflow 저장소가 아니다.
- 보상, Event, Shop과 admin 지급은 슬롯 수 때문에 실패하거나 지연되지 않는다.

### 소비 아이템

소비 아이템은 `ViewingMap`과 `NodeConfirm`에서 사용하는 run 자원이다.

- Core가 대상과 효과를 검증하고 성공 시 definition stack 수량을 정확히 1 줄인다.
- 직원의 단일 active modifier가 이미 있으면 성공한 command가 기존 modifier를 즉시 제거하고 새
  modifier로 교체한다. 환불, 차단, 별도 overwrite 확인 modal은 없다.
- Metadata의 static UUID가 definition-backed stack의 wire selector이며 copy별 owned UUID는 없다.
- 적용한 직원 modifier duration은 전투 결과 commit마다 한 번 감소한다. Retreat와 non-combat
  node에서는 감소하지 않는다.
- 전투·결과·Safe Node·Shop·Reward 내부에서는 사용할 수 없으며 사용한 아이템은 되돌리지 않는다.
- Unity는 authoritative quantity와 command/snapshot 결과를 표시하고 local count를 추론하지 않는다.

### 상점

Shop은 Enkephalin으로 authored merchandise를 구매하고 허용된 보유품을 판매하는 Safe Node다.
Headquarters와 Unknown Shop은 같은 transaction/session 모델을 사용하되 서로 다른 merchandise
profile을 사용한다.

**합의 변경:** 유물은 상점에 판매할 수 없다. 유물의 상점 구매처 채택은 미정이다. 상점의 재료
지급·구매·분쇄 경로는 일반 전투 전용 무기 강화 필수 재료 정책과 함께 검토한다.

- Core가 offer, price, refresh와 sell eligibility를 계산하며 Unity는 재추첨하거나 재계산하지 않는다.
- Offer는 session 내 고유 identity, typed merchandise, 양수 quantity와 총 구매가를 가진다.
  장비 구매는 새 owned instance를 만들고 소비 아이템은 definition stack에 수량을 더한다.
- 구매한 자리는 자동 보충하지 않으며 non-repeatable 상품은 같은 session의 refresh에 재등장하지 않는다.
- Refresh는 authored 비용과 횟수를 원자적으로 소비해 page 전체를 교체한다. 비용·횟수·후보가
  부족하면 상태를 바꾸지 않는다.
- Sell은 현재 owned/착용 상태, tag, quantity와 quote를 검증한다. 귀속·장착 장비와 0원 거래는
  허용하지 않는다.
- 구매가는 base price percentage를 올림, 판매가는 내림하며 live validation은 same/cross-Shop
  arbitrage와 산술 overflow를 거부한다.

### 노드 진입 전 브리핑

전투 진입에 필요한 기본 정보는 항상 무료로 제공한다. 재화를 소비해 미리보기의 정보량을 늘리는 기능은 두지 않는다.

기본 브리핑에는 아래 정보가 포함된다.

- 전투 모드와 임무 변형
- 전장 템플릿과 배치 가능 구역
- 주요 route 또는 예상 진입 방향
- 위험도와 보상 후보
- 주요 적 또는 핵심 위협 요약
- 위협 경고와 그 신뢰도

정확한 웨이브 수, 모든 등장 타이밍, 숨은 기믹의 세부 값은 기본적으로 전투 중 관찰과 리트라이 학습 영역으로 둔다. 단, 플레이어가 납득 가능한 배치를 할 수 없을 정도로 정보가 부족하면 별도 소비 자원 없이 기본 브리핑 자체를 보강한다.

`EnemyBriefing`은 플레이어에게 보여줄 정보이며 실제 스폰 수량/종류의 source of truth가 아니다. 실제 전투 스폰은 `SpawnWave.enemy_entries`가 담당한다.

## 전투

이 장은 플레이어가 경험하는 전투 진입, 배치, 적 구성, 공격과 피해 피드백의 의미를 소유한다.
타겟 판정 알고리즘은 [skill_target_contract.md](../skill_target_contract.md), event/checkpoint/result 생산자
불변식은 [core_runtime_contract.md](../core_runtime_contract.md), 정확한 wire shape와 message order는 외부
Unity canonical이 소유한다.

### 전투 모드

현재 live 전투 모드는 명일방주식 실시간 배치 방어인 `DefenseRoute` 하나다.

- 플레이어 유닛은 배치 위치에서 route 적을 공격하고 `block_capacity`만큼 저지한다. 저지된 적은
  경로 전진을 멈추며, 판정용 타일과 저지 관계는 아래 규칙을 따른다.
- 모든 필수 wave·적 처리와 보호 대상 생존이 성공 조건이며 보호 대상 파괴나 scenario 조건이
  실패를 만든다. Pause, 배속과 battle retreat는 server simulation 기준이다.
- 일반 전투와 보스는 같은 transport, 전장, 배치와 wave 모델을 공유한다.
- 생존 전투는 별도 `Encirclement` mode가 아니라 `DefenseRoute / Defense + survive_timer_ms`로
  작성하며 timer 도달 시 남은 적과 무관하게 승리한다.

`DefendPoint`, `Encirclement` mission variant, `Recovery`, `DefendAndEscape`,
`TacticalGroupPlan`과 formation AI는 현재 live 계약이 아니다. `SplitRoom`은 전투 mode가 아니라
필요할 때 battlefield template로 표현한다.

### 저지 판정과 적의 타일

**합의 변경 — 2026-09-10 사용자 대화.** 명일방주식 저지를 목표로 하며, 이번에 합의한 핵심은
저지된 적의 판정용 타일을 아군의 타일로 일괄 변경하지 않는 것이다. 원작의 세부 수치·예외 전체를
이 대화만으로 확정하지 않는다.

#### 판정용 타일과 연출 위치

- 저지가 성립해도 적의 판정용 타일은 변경하지 않는다. 적은 저지가 성립한 시점에 점유하던
  타일을 저지 중에도 유지한다. 같은 타일 점유는 저지 관계의 필수 조건이 아니다.
- 최초 접근 중 아군 바로 앞에서 저지된 적은 접근해 온 쪽의 앞 타일에 판정 좌표를 둔다.
  여기서 앞은 아군의 facing으로 고정한 방향이 아니라 그 적이 접근해 온 쪽이다.
- 아군의 타일을 통과하던 중 저지된 적은 그 아군과 같은 타일에 판정 좌표를 둔다.
  최초 접근 때 적이 멈추는 앞 타일로 되돌려 놓지 않는다.
- 연출은 적이 아군 가까이에서 멈춰 싸우는 모습으로 표현한다. 서로 겹치지 않게 표시 위치를
  조금 보정할 수 있지만, 이 연출 보정으로 판정용 타일이나 저지 여부를 바꾸지 않는다.
- 타일 기반 공격 범위·장판·타일 효과는 각 적의 판정용 타일을 사용한다. 연출상 가까이 보인다는
  이유로 앞 타일의 적을 아군 타일의 효과 대상으로 옮기지 않는다.
- 이 타일 유지는 저지가 유지되는 동안의 규칙이다. 실제 이동·강제 이동으로 저지가 해제된 뒤에도
  과거 타일을 고정하거나 적을 과거 저지 위치로 복원하는 규칙은 아니다.

#### 최초 접근과 통과 중 새 저지

아래 예시는 목적지가 왼쪽, 스폰 지점이 오른쪽인 직선 route이며 A는 1저지 아군이다.
B와 C는 각각 저지 가능한 적이다.

| 순서 | 상황 | 판정용 타일과 결과 |
| --- | --- | --- |
| 1 | B가 접근해 A 바로 앞에서 최초 저지됨 | B는 A 앞 타일에서 멈추고 그 위치에서 A를 공격 |
| 2 | A가 B를 저지하는 동안 C가 접근 | A의 저지 수가 가득 차 C는 B의 타일과 A의 타일을 통과하며 목적지로 이동 |
| 3 | C가 A의 타일 위를 통과하는 중 B가 사망 | A에 저지 여유가 생겨 현재 위치의 C에 대해 새 저지 판정 |
| 4 | A가 C를 저지 | C는 A의 타일을 판정용 타일로 유지하며 전진을 멈춤 |

- 적의 접근 시점뿐 아니라 저지 여유가 생겼을 때도 현재 저지 가능한 위치의 미저지 적을
  다시 판정한다. 이미 통과 중이라는 이유만으로 후보에서 제외하지 않는다.
- 적이 점유한 타일이나 아군의 점유 타일 자체가 모든 후속 적을 막는 벽이 되지는 않는다.
  저지 여유가 없으면 이 예시의 C는 계속 이동한다.
- 새 후보가 나타났다는 이유만으로 이미 저지한 적을 다른 적으로 교체하지 않는다.

#### 저지 상대의 기본 공격

- 아군과 자신이 저지한 적은 서로 다른 판정용 타일에 있어도 저지 관계를 근거로 서로를 기본
  공격 대상으로 삼을 수 있다. 이를 위해 적의 좌표를 아군의 좌표로 덮어쓰지 않는다.
- 저지 상대에 대한 이 판정은 일반 기본 공격의 위치·방향·사거리 때문에 서로를 공격할 수 없게
  되는 상황을 방지한다. Attack/Defense 태세, Hard CC, 행동 제한, 공격 주기와 피해 유효성 등
  나머지 기본 공격 규칙은 계속 적용한다. 저지 성립이 즉시 공격이나 동시 공격을 강제하지 않는다.
- 다른 유닛의 공격이나 스킬·장판에 이 기본 공격 예외를 확장하지 않는다. 각각 자신의 대상·범위
  규칙과 해당 적의 판정용 타일을 사용한다.

#### 저지 해제와 남은 결정

사망·철수·저지 불가 전환·Hard CC·전술 이동·자리 교대·강제 이동과 저지 수 감소는 각 절의
명시적 해제·재판정 규칙을 따른다. 연출 위치만 바뀌는 것은 실제 이동이나 저지 해제로 취급하지 않는다.

정확한 저지 시작 범위, 이동 중 위치를 판정용 타일로 구분하는 경계, 실제 거리 이탈 시 해제 기준,
여러 신규 후보 중 선택 순서와 같은 시각의 이동·사망·새 저지 판정 순서는 후속 결정 대상이다.
앞선 AI 제안인 별도의 더 넓은 유지 범위, 신규 후보의 근접 우선, 원작 분석 자료의 반경·갱신 주기는
이번 합의로 자동 채택하지 않는다. 기존의 “거리 이탈만으로 해제되지 않는다”를 무제한 거리 유지의
확정 규칙으로 사용하지 않는다.

현재 [skill_target_contract.md](../skill_target_contract.md#basic-attack-range-authoring)의 저지 적을
같은 logical tile로 다루는 설명과 관련 runtime 판정은 이 목표 정책의 후속 반영 대상이다.
이 초안 갱신은 기존 계약·코드·Unity 표현이 이미 변경됐다는 뜻이 아니다.

### 전술 포인트

플레이어 표시명은 `전술 포인트`, code/DTO 이름은 `tactical_resource`다. Active battle마다 player-side
전역 pool 하나를 만들며 첫 live 값은 current `20`, maximum `99`, authoritative battle time 1초당 `1`
자연 회복과 최초 배치 base cost `10`이다.

- Current는 항상 `0..=maximum`이고 maximum은 battle start에 고정된다. 자연 회복과 명시 skill effect는
  초과분이나 감소분을 저장하지 않고 clamp한다. Maximum 변경 effect, encounter별 override, debt, 부분
  지불과 암묵적인 환경 drain은 없다.
- 최초 배치와 회수/전투불능 뒤 재배치는 같은 pool에서 전액을 보유할 때만 비용을 한 번 지불한다. 실패한
  command는 resource, deployment, salt, cooldown, event와 revision을 바꾸지 않는다. 철수는 비용도 환불도
  없다.
- Defense 직원의 전술 이동은 `5`를 한 번 지불하며 직원별 battle-time `10,000ms` cooldown을 시작한다.
  `DeployCostReduction`은 이 비용에 적용하지 않는다.
- 개별 skill은 `ModifyTacticalResource(amount)`로 signed current delta만 명시할 수 있다. DefenseFocus와
  Rest는 전술 포인트를 생산하지 않는다.
- 자연 회복은 simulation battle time의 온전한 1초 cadence를 따른다. Pause wall time에는 회복하지 않고
  Resume 뒤 catch-up하지 않으며 maximum에서 넘친 회복도 bank하지 않는다.
- Checkpoint/resync와 root InBattle snapshot은 같은 battle-local current/maximum/recovery를 반환한다.
  전투 종료와 Retreat는 pool을 폐기하고 다음 battle은 initial `20`에서 다시 시작한다.

### Hard CC와 Silence

전투 제어 제한은 command마다 `Stun`/`Freeze` 이름을 다시 나열하지 않고 현재 active buff의 분류에서
파생한다. 현재 `Stun`과 `Freeze`는 Hard CC, `Silence`는 Silence capability를 제공한다. 서로 다른 active
instance가 같은 capability를 제공하면 하나라도 남은 동안 제한이 유지되며, Hard CC 해제 시각은 남은
Hard CC의 가장 늦은 expiry다.

- Hard CC는 새 automatic 기본 공격과 automatic/manual skill 시작을 막는다. 적용 순간 아직 release되지
  않은 기본 공격 windup과 skill cast/focus는 취소하고 해당 action이 소유한 lock도 함께 해제한다.
- 이미 release된 기본/skill projectile, delayed skill step, persistent area, buff 적용과 확정된 파생
  effect는 caster의 Hard CC와 무관하게 기존 lifecycle을 계속한다. 이미 소비한 resonance와 시작한 proc
  cooldown은 환불하거나 되감지 않는다.
- Hard CC 동안 보행을 멈추고 새 이동을 막으며 blocker 또는 blocked enemy가 Hard CC이면 engagement를
  즉시 해제한다. Positive resonance gain도 중단한다. 피해, periodic effect, passive/reactive trigger와
  battle-time cooldown은 계속 흐른다.
- Silence는 새 automatic/manual skill만 막는다. 이미 시작한 skill과 기본 공격, 이동, block, damage,
  resonance gain과 철수는 그대로 유지한다.
- Manual skill의 typed 거부 reason은 Hard CC에 `hard_crowd_control`, Silence에 `silenced`를 사용한다.
  둘이 겹치면 Hard CC reason이 우선한다. 실패한 시작 요청은 resonance, cooldown, event와 battle revision을
  바꾸지 않는다.
- 자발적 철수는 두 capability와 무관하게 허용하며 아래 철수 lifecycle을 그대로 따른다.

### 철수와 재배치

Active 배치 player unit은 태세, 기본 공격 windup, skill cast/focus, Hard CC, Silence와
`FieldWorking` 여부에 관계없이 자발적으로 철수할 수 있다. 사망·전투불능·이미 철수한 unit은
철수할 수 없고, 실패한 요청은 전투 상태나 revision을 바꾸지 않는다.

- 철수는 전술 자원을 소비하거나 이미 지불한 배치 비용을 환불하지 않는다. 해당 runtime instance를
  즉시 `Withdrawn`으로 전환해 official field/checkpoint에서 제거하고 engagement를 해제한다.
- Release 전 기본 공격과 skill cast/focus는 취소한다. 이미 발사된 기본/skill projectile과 이미 생성된
  `GroundFixed` area는 독립 delivery로 원래 lifecycle을 계속한다. 철수 unit을 향해 비행 중인 hostile
  projectile은 target이 inactive이므로 miss한다.
- `FollowCaster`, `FollowTarget`, aura, channel/focus처럼 source runtime에 종속된 효과는 철수와 함께
  종료한다. 철수 unit이 적용했거나 자신에게 적용된 buff/debuff는 각각 `CasterWithdrawn` 또는
  `TargetWithdrawn`으로 종료한다.
- 철수 재배치 HP는 철수 순간
  `min(max_hp, current_hp + floor(max_hp * 30 / 100))`으로 잠근다. 대기 중 회복이나 후속 condition/max HP
  변화는 이 값을 소급 변경하지 않는다.
- 전투불능은 runtime instance마다 physical damage `+1`과 authored trauma를 정확히 한 번 attempt-local
  condition에 반영한다. 최신 condition이 combat eligible일 때만 재배치 후보가 되며 시작 HP도 그
  condition에서 다시 계산한다.
- 재배치는 같은 employee identity의 새 runtime instance를 만들고 현재 persistent condition,
  확정 loadout/fragment, 유효한 battle-scoped modifier와 배치 초기 효과를 다시 구성한다. 이전 instance의
  buff, resonance, target, cooldown, cast/action, block과 aura는 상속하지 않는다. `InitialSkillCharge`와
  authored on-deploy effect는 매 배치 다시 적용한다.
- 자발적 철수/전투불능 대기는 각각 battle time `30초`/`90초`다. 재배치 base cost는
  `ceil(base_deploy_cost * 150 / 100)`이고 `DeployCostReduction`은 최초 배치와 재배치에 모두 적용한다.
  Pause 중 대기는 진행하지 않으며 checkpoint/resync는 잠긴 HP, ready time과 cost를 그대로 투영한다.

### Attack/Defense 태세와 타일 효과

Active 배치 직원은 authoritative `Attack | Defense` 태세 하나만 저장한다. 최초 배치와 모든 재배치는
`Attack`으로 시작하고 그 battle time부터 직원별 `1,500ms` 전환 cooldown을 시작한다. 양방향 전환은
비용이 없고 같은 cooldown을 사용한다.

- 같은 태세 요청은 `already_active`, cooldown 중 반대 태세 요청은 `stance_cooldown`, Hard CC 중 전환은
  `hard_crowd_control`로 거부한다. Silence, 기본 공격 windup과 skill cast/focus는 전환 자체를 막지 않는다.
  실패는 태세, 효과, cooldown, 전술 포인트, event와 revision을 바꾸지 않는다.
- `Attack`은 평상시 기본 공격과 automatic/manual skill 시작을 허용하지만 즉시 공격을 강제하지 않는다.
  `Defense`는 새 regular 기본 공격과 모든 automatic/manual skill 시작을 막는다. 전환 전에 시작한 windup,
  cast/focus, projectile, delayed/proc/multihit와 `ExtraAttack`은 취소하지 않고 완료하며 passive/reactive
  triggered effect도 계속 동작한다.
- `Defense`의 효과는 별도 저장 state가 아니라 현재 tile과 장착 무기 profile에서 파생한다. Ground에서는
  `DefenseFocus`, Platform에서는 `Rest`이고 `Attack`이면 어느 tile에서도 파생 효과가 없다.
- Ground DefenseFocus는 authored base block에 `block_capacity_bonus`를 더하고 general incoming damage
  reduction을 type별 reduction과 가산한 뒤 최종 `0..=100%`로 clamp한다. 해제 즉시 base 값으로 돌아간다.
  Capacity 감소 시 기존 engagement는 route progress 내림차순, spawn order, unit ID 순으로 남기며 새 적이
  기존 적을 밀어내지 않는다. Hard CC 동안 effective block은 0이지만 damage reduction은 유지된다.
- Platform Rest는 Defense 진입 뒤 온전한 `1,000ms`마다 자신의 HP `3`, resonance `5`를 각각 최대치로
  clamp 회복한다. full interval 전 첫 tick, recently-hit 제한, revive, 전술 포인트 회복과 catch-up은 없다.
  Attack 전환이나 Rest 조건 상실은 즉시 중단하고 다시 조건을 얻으면 새 full interval을 요구한다.
- 같은 timestamp의 모든 확정 피해가 Rest보다 먼저다. 그 피해로 사망하면 Rest가 되살리지 않는다. Hard CC는
  HP/resonance Rest를 모두 건너뛰되 cadence는 유지하고, resonance gain lock은 resonance만 건너뛴다. Skipped
  tick은 이후 보상하지 않으며 실제 변화가 0이면 HP/resonance event를 만들지 않는다. Pause는 battle clock과
  cadence를 함께 동결한다.

### 전술 이동과 아군 점유

`TacticalRelocateUnit`은 Defense 상태인 active 배치 직원의 같은 runtime instance를 player deployment
zone 안의 새 position/facing으로 즉시 옮긴다. 철수·재배치, 보행, pathfinding, 예약 또는 도착 state가 아니다.

- Source는 Defense여야 한다. Hard CC와 release 전 기본 공격 windup/skill cast-focus는 막지만 Silence와
  현재 block engagement는 막지 않는다. 같은 position+facing은 `already_active`, 같은 position에서 facing만
  바꾸는 요청은 정상 이동과 같은 비용/cooldown으로 허용한다.
- Destination은 valid/non-obstacle player deployment tile이고 현재 무기 `deployment_affinity`와 맞아야 한다.
  Active player employee와 defense object는 각자 logical tile 하나를 점유하므로 최초 배치, 모든 재배치와
  전술 이동은 다른 아군이 점유한 tile을 `position_occupied`로 거부한다. 이동 source 자신의 현재 tile은
  검사에서 제외하고 enemy와의 tile overlap은 허용한다. Occupied tile을 자동 push/swap하지 않는다.
- 성공은 모든 검증 뒤 position/facing, 직원별 `10,000ms` cooldown, 전술 포인트 `5`, engagement/target와
  tile effect를 한 번에 반영한다. HP, buff, resonance, Attack/Defense, stance cooldown, runtime identity와
  이미 시작된 action/delivery cooldown은 보존한다. 실패는 이 상태와 event/revision을 전혀 바꾸지 않는다.
- 기존 block engagement를 해제한 뒤 새 위치에서 deterministic하게 다시 계산한다. Range, aura/area 포함
  관계도 새 위치를 사용한다. 이미 release된 homing projectile은 이동한 target을 계속 추적하고 fixed
  projectile과 `GroundFixed` area는 원래 좌표를 유지하며 `FollowCaster`/`FollowTarget`은 새 좌표를 따른다.
- Ground↔Platform 이동은 두 tile profile을 모두 가진 `any` 무기만 가능하다. Defense를 유지한 채 old
  DefenseFocus/Rest를 제거하고 새 효과를 원자 적용한다. Platform block은 0이고 Ground 복귀는 authored
  base+bonus로 재계산한다. 새 Platform Rest는 이동 시점부터 full `1,000ms`를 기다리며, 같은 tile의
  facing-only 이동은 기존 Rest cadence를 재시작하지 않는다.
- 철수/전투불능 뒤 재배치는 새 runtime instance이므로 이전 instance의 전술 이동 cooldown을 상속하지 않는다.

`TacticalSwapUnits`는 서로 다른 active 배치 직원 두 명의 현재 position을 별도 destination payload 없이 한
transaction에서 맞바꾼다. Occupied relocation을 swap으로 바꾸거나 swap 실패를 relocation으로 바꾸는 fallback은 없다.

- 두 직원 모두 Defense이고 각자의 전술 이동 cooldown이 준비되어야 하며, Hard CC, release 전 기본 공격
  windup/skill cast-focus와 exclusive movement segment가 어느 한쪽에라도 있으면 전체를 거부한다. Silence와
  기존 block engagement는 허용한다.
- 각 직원은 상대의 current tile에 대해 player zone, valid/static tile, weapon affinity와 Ground/Platform
  profile을 전부 통과해야 한다. Occupancy 검사에서는 두 participant만 함께 제외하므로 그 tile의 제3 active
  player employee/defense object는 전체 swap을 `position_occupied`로 거부한다.
- Command는 각 직원의 새 facing을 명시한다. Server는 기존 facing을 자동 복사하거나 서로 교환하지 않는다.
  성공은 전술 포인트 `5`를 command당 한 번 지불하고 두 직원의 cooldown을 같은 `time+10,000ms`로 설정하며
  event와 battle revision도 각각 한 번만 증가시킨다.
- 양쪽 engagement를 먼저 해제하고 두 body/facing/tile effect를 모두 commit한 뒤 block/target/range/area를
  한 번 재계산한다. 따라서 중간 duplicate/empty occupancy, sequential relocation, walking/reservation state는
  관찰할 수 없다. HP, buff, resonance, stance, action cooldown, runtime identity와 release된 delivery는 각
  직원에게 남는다. Ground↔Platform swap도 Any profile의 effect/Rest epoch를 양쪽 최종 tile에서 계산한다.
- 어느 preflight나 resource 검사가 실패해도 양쪽 position/facing/effect/cooldown, resource, event와 revision은
  전혀 바뀌지 않는다.

### 수동 지정 스킬

모든 live skill은 effect 대상인 `SkillTarget`과 별도로 선택 주체 `SkillCastSelection` 하나를 필수로
명시한다. 허용 값은 `self`, `automatic_unit`, `player_unit`, `player_tile`이며 weapon archetype, 첫 step,
effect target에서 선택 주체를 추론하거나 default/fallback하지 않는다.

- `self`는 caster 자신, `automatic_unit`은 Core의 authored target rule을 사용하며 별도 targeting focus 없이
  `ActivateSkill`을 즉시 제출한다. 이 두 mode에 explicit target을 보내면 `skill_target_type_mismatch`다.
- `player_unit`과 `player_tile`은 각각 explicit `Unit`/`Tile` payload가 필요하다. 누락은
  `skill_target_required`, 종류 불일치는 `skill_target_type_mismatch`이고 다른 target kind나 automatic
  target으로 보정하지 않는다.
- PlayerUnit은 active hostile lifecycle, air eligibility, effect usefulness와 최종 reach/range를 검사한다.
  PlayerTile은 valid battlefield tile과 reach/range만 검사하며 occupant를 요구하지 않는다. Caster의 manual
  mode, Attack stance, action lock, Hard CC/Silence와 full resonance도 target보다 먼저 모두 통과해야 한다.
- PlayerUnit/PlayerTile 선택은 authoritative tactical focus를 시작한다. Unpaused battle은 focus 동안 effective
  speed `X0_5`, 이미 Pause인 battle은 frozen 상태를 유지한다. Focus는 purpose/caster/skill과 종료 때 복원할
  playback만 저장하며 hover와 미제출 candidate를 저장하지 않는다.
- `PreviewSkillTarget`은 current revision에서 candidate 하나를 읽기만 하고 event/revision을 만들지 않는다.
  최종 `ActivateSkill`은 같은 payload를 다시 검증한다. 실패는 focus, playback, resonance, event와 revision을
  전부 유지하고 Cancel은 비용 없이 focus를 끝내며 이전 playback을 정확히 복원한다.
- 성공은 captured Unit/Tile로 manual cast를 시작하고 focus 종료/playback 복원을 같은 revision commit에
  반영한다. 이후 selected unit이 무효가 된 direct effect는 no-op하며 다른 unit을 찾지 않는다. Unit 기반
  TileArea와 PlayerTile area는 acceptance 시점 tile을 유지한다. 이후 무효화나 빈 tile에도 refund/rollback은
  없고 `PlayerUnit|PlayerTile + RetargetOnStep` authoring은 거부한다.
- Checkpoint/resync는 focus와 restore playback을 그대로 반환한다. Pause에서 시작한 focus는 성공/취소 뒤에도
  Pause다. Active-skill 전용 추가 cooldown이나 수동 기본 공격은 없다.
- 첫 live PlayerUnit path는 `fragment_freischutz_black_round`와 awakened progression의
  `freischutz_magic_bullet`뿐이다. 기존 range/focus/damage/delivery tuning은 유지하며 live PlayerTile skill은
  아직 없다.

### 현장 작업

현장 작업은 tactical point, static defense object 또는 tile kind를 재사용하지 않는 별도 setup object다.
Incomplete object의 authored work cell을 active Player 직원이 최종 점유하면 별도 command나 비용 없이 같은
mutation에서 자동 assignment된다. 직접 배치와 재배치도 work cell이면 즉시 `Defense + FieldWorking`으로
시작한다.

- Progress는 직원이 아니라 object가 소유하며 interruption policy는 `Preserve` 하나뿐이다. 각 eligible worker는
  흐른 battle time `1ms`마다 `1ms`를 기여하고 여러 cell의 기여는 합산된다. Required work에서 clamp하며 worker가
  0명이 되어도 줄거나 초기화되지 않는다.
- `FieldWorking`은 regular 기본 공격과 automatic/manual skill 시작, block, 태세 전환, Ground DefenseFocus와
  Platform Rest를 막는다. Worker는 피해를 정상적으로 받고 enemy overlap/attack은 assignment를 끊지 않는다.
  자발적 철수, 전술 이동과 자리 교대는 계속 허용된다.
- Hard CC는 assignment와 누적 progress를 보존하되 active 동안 그 worker의 기여만 멈춘다. 해제 뒤 놓친 시간을
  보충하지 않는다. Pause는 simulation clock 자체를 멈추므로 기여도 진행하지 않고 Resume 뒤 catch-up하지 않는다.
- 철수·사망·이동·교대는 모든 position commit이 끝난 최종 점유에서 assignment를 한 번 reconcile한다. Cell을
  떠난 직원은 `Ready + Defense`와 현재 tile의 일반 DefenseFocus/Rest로 복귀하고, 새 적격 직원은
  `FieldWorking`이 된다. Completed cell은 더 이상 assignment하지 않고 underlying tile로 돌아간다.
- 최초 required 도달 update에서 object는 정확히 한 번 Completed가 되고 typed effect를 한 번 실행한다. Full HP
  repair도 완료·소모되며 재사용하지 않는다. Assignment start/stop과 completion만 semantic event이고 현재
  progress/status/worker mapping은 checkpoint/reconnect가 소유한다.
- 첫 live object는 `suppress_freischutz`의 `black_box_emergency_repair`다. Presentation은
  `EmergencyRepairTerminal`, object와 단일 Ground work cell은 `(2, 6)`, required work는 `10,000ms`, completion은
  보호 대상 `black_box_device` HP를 최대 `100` 회복한다.

`AssignFieldWork`, `CancelFieldWork`, 별도 work cost, Reset mode, generic scripting/status framework와 추가 live
object는 현재 계약이 아니다.

### Pause와 직렬 전투 명령

Pause는 전투 simulation clock만 정지한다. 현재 배치, 철수, 태세 전환, 전술 이동/교대, 수동 스킬 시작, 배속 변경, 전투 후퇴와
read-only 상태 요청은 Pause 여부만으로 금지되지 않으며, frozen battle time의 최신 상태에서 각 명령의
평상시 eligibility를 그대로 검사한다.

- 플레이어 mutation은 한 번에 하나씩 처리한다. 앞 command의 authoritative response와 최신
  `battle_state_revision`을 받은 뒤 다음 command를 보낸다. Local command plan/batch와
  `ResumeBattleWithPlan`은 없다.
- 성공한 command의 배치/철수, 자원 소비, skill cast 시작과 stored speed 변경은 Pause 중에도 즉시
  반영되고 revision을 1 증가시킨다. 실패는 state, resource, event와 revision을 바꾸지 않는다.
- 뒤 command가 실패해도 앞 command의 성공을 되돌리지 않는다. 같은 frozen time의 순서는 server actor
  처리 순서, revision과 event sequence가 소유한다.
- 적 이동/공격, pending cast resolve, buff duration, 재배치 대기와 자연 회복처럼 battle-time 진행이
  필요한 동작은 멈춘다. Resume 뒤 Pause 시간을 소급하거나 비축하지 않는다.
- Pause 중 배속 변경은 clock을 진행시키지 않고 Resume 뒤 사용할 stored speed만 바꾼다.
- State/resync와 preview query는 최신 Pause state를 읽되 revision을 증가시키지 않는다. Retreat 성공은
  active battle과 Pause state를 함께 종료한다.
- 연결이 끊기면 server가 active battle을 자동 Pause하고 시간 제한 없이 같은 process에 보존한다. 재접속은
  동일 상태를 paused로 받으며 명시적 Resume 전에는 진행하지 않는다. Server process가 종료되면 active
  battle을 복구하지 않고 마지막 완료 SavePoint로 돌아간다.

### 전투 시작과 Unity 경계

전투는 live battle state를 사용한다.

- Scene construction은 `battle_setup_snapshot`, 연출 timing은 `battle_update.events_delta`,
  현재 상태와 복구는 `battle_update.checkpoint`, 결과 통계는 `selected_event.result_stats`가
  source of truth다.
- Active event/result는 전투 종료 뒤 player history로 누적하지 않는다.
- Core가 공격·스킬 판정과 preview를 계산하며 Unity는 authored metadata에서 재계산하지 않는다.

생산자 불변식은 [core_runtime_contract.md](../core_runtime_contract.md), targeting은
[skill_target_contract.md](../skill_target_contract.md), JSON shape와 message order는 외부 Unity
canonical을 따른다.

### 전장과 배치

Node마다 route, 장애물, 배치/출현 구역이 다른 비사각형 전장을 사용할 수 있다.

- `battle_setup_snapshot`이 실제 scene source이고 `combat_preview`는 진입 전 briefing이다.
- 배치와 range 후보는 tile 기반, 이동과 충돌은 2D 연속좌표 기반이다.
- Player tactical slot의 logical allied occupancy는 Core active player combatant/defense-object position이
  소유하며 배치 preview와 최종 deploy/redeploy/relocation commit이 같은 검사를 사용한다.
- 크기는 `Small`, `Medium`, `Large`이며 Boss는 기본적으로 Large다.
- Archetype은 `OpenHall`, `Corridor`, `ChokePoint`, `Ambush`, `Surrounded`,
  `SplitRoom`, `ObstacleRoom`, `BossArena`의 데이터 분류다. `SplitRoom`은 현재 live
  encounter 배정 대상이 아니다.
- ASCII template/route authoring은
  [data_loading_contract.md](../data_loading_contract.md#battlefield-template-authoring), route와
  movement projection은
  [core_runtime_contract.md](../core_runtime_contract.md#route-and-battlefield-projection)가 소유한다.

### 적 구성과 웨이브

일반 wave의 주력은 `CorrodedEmployee`이고 `Abnormality`는 Elite/Boss/특수 encounter의 중심
위협이다. `FacilityEntity`는 아직 live 대상이 아니다.

Encounter 분류:

- `Normal`: 순수 침식 직원 전투. Primary abnormality와 환상체 bonus objective가 없다.
- `Elite`, `NormalBoss`, `FinalBoss`: 정확히 하나의 `primary_abnormality_id`를 가지며
  각각 elite, 일반 boss, final-boss급 encounter에 사용한다. Standard Floor 3은 간소화된
  `NormalBoss`이며 `FinalBoss`가 아니다.
- `ReferenceFinalBoss`: FinalBoss vertical path를 검증하는 명시적 nonproduction class다. Normal map,
  BossOmen pool/cycle, 파편/repeat reward, 모든 FinalBoss 기록과 일반 progression에 들어가지 않는다.
- 환상체 `threat_class`는 최소 Elite이고 Boss 환상체만 Boss다. 실제 spawn은 encounter
  classification이 아니라 wave data가 소유한다.

Wave는 `SpawnWave`로 관리한다.

- 각 Wave는 stable trigger ID, priority, closed primary trigger와 optional deadline을 명시한다. Primary는
  absolute battle time, 선행 group clear, bound actor HP/사망/지점 도달 중 하나다.
- 조건이 최초 binding 시 이미 참이면 같은 boundary에 한 번 발동하고, actor death는 binding 뒤 실제 death만
  관찰한다. Same-process reconnect는 trigger epoch를 유지해 재발동하지 않는다.
- `required_for_victory: false` wave는 승리 필수 대상이 아니다.
- Spawn 전 required Wave도 Victory를 보류하며 모든 required group이 실제 spawn·처리된 뒤에만 승리할 수
  있다. Deadline과 primary가 같은 시각이면 primary 원인을 사용하고 Wave 간 dependency cycle은 data 오류다.
- Manual source는 typed `CorrodedEmployee(profile_id, tier, count)` 또는
  `Abnormality(abnormality_id, tier, count)` entry를 사용한다.
- Preset 생성도 허용하지만 결과는 CombatPreview 때 `enemy_entries`로 확정하며 전투 중
  재생성하지 않는다.

#### Live Boss Runtime Reference

`suppress_little_red`는 production content에 opt-in한 교체 가능한 Boss Runtime reference다. 별도
`reference_cwbs_final_boss`는 conditional Wave, Summon, neutral signal, named outcome와 Victory registration을
연결하는 nonproduction reference다. 둘 다 최종 캐릭터 설계나 lore 계약이 아니다.

- `primary_abnormality` wave의 spawn index 0만 `little_red_reference_encounter`의 같은 이름 actor slot에
  bind한다. 이 wave는 기존 5초 timing과 구성에 `defense_main` route만 추가한다.
- Little Red는 spawn 뒤 reachable active hostile 중 minimum path-cost target 하나를 고정하고,
  `AcquireTarget -> MoveToTarget -> RejoinForwardRoute`를 한 번 실행한 뒤 기존 `RouteCombat`으로 돌아간다.
  Target 없음, invalidation, path failure와 timeout은 모두 authored rejoin/route exit를 따르며 implicit
  retarget, hidden retry 또는 두 번째 pursuit를 만들지 않는다.
- 기존 `f-01-57_little_red` stats, support wave, reward, 5초 wave timing과
  `little_red_hunt_the_prey` leaf skill은 이 reference 때문에 바뀌지 않는다. Automatic skill의 step-level
  targeting은 기존 skill runtime authority이고 Boss target lock을 재작성하지 않는다.
- 전용 bounty, Big Bad Wolf 연계, art/animation/VFX와 public Boss lifecycle은 없다. Existing opponent
  capsule, battle event와 checkpoint projection이 presentation/transport 경계다.
- CWBS reference의 summon은 독립 definition/limit/provenance와 exact batch를 가지며 cleanup signal로
  non-kill despawn한다. Summoned unit은 Wave/Boss/보상/파편/도감 authority가 아니고 승리를 막지 않는다.
- CWBS reference는 ID/class/display name으로 nonproduction임을 드러내며 authored reward, node bonus와
  progression을 모두 비활성화한다. Production 승격은 별도 content authoring/balance review가 필요하다.
- 다른 production encounter는 `boss_runtime`에 opt-in하지 않으며 기존 ordinary planner와 authored combat
  semantics를 그대로 사용한다.

### 기본 공격과 스킬 타겟팅

타겟팅의 상위 원칙은 단순하다.

- 어떤 공격이든 deterministic expected final damage가 0보다 크면 유효 후보로 본다.
- 피해가 0이어도 선택한 적 또는 AoE 안의 적에게 적용되는 적대적 target-applied effect가 있으면 유효 후보로 본다.
- 피해도 없고 적대적 대상 효과도 없으면 자동 평타/스킬 후보에서 제외한다.
- 후보가 여럿이면 기본 공격은 무기의 `targeting_profile`, 스킬은 스킬 자체의 명시
  `UnitTargetRule`과 deterministic fallback/tie-break 규칙을 따른다. 스킬이 무기 profile을
  상속하는 경로는 없다.
- 피해 가능 대상과 효과만 가능한 대상 사이에 별도 우선순위는 두지 않는다.

**합의 변경:** 저지 상대의 기본 공격 위치·사거리 판정과 적의 독립적인 판정용 타일은
[저지 판정과 적의 타일](#저지-판정과-적의-타일)을 따른다. 현재 인접 targeting 계약에 남은
같은 logical tile 처리는 후속 정합화 대상이며, 스킬·다른 공격자의 범위 예외로 확대하지 않는다.

적대적 효과의 분류, 타겟 filter 평가 시점, blocker-first, route enemy 재이동, cast/step retarget,
projectile lifecycle, `TileArea`, range authoring의 세부 source of truth는
[skill_target_contract.md](../skill_target_contract.md)다.

### 피해 타입과 피드백

`Physical`, `Magic`, `True`는 빌드 효율 차이를 만들지만 영구 단일 타입 면역으로 특정
딜러 보유만 검사하지 않는다. 높은 방어/저항과 일시 기믹은 허용하되 관통, 저항 감소, 고정 피해,
제어, 저지, 아이템과 timing 같은 복수 counterplay를 둔다.

Core가 계산한 `HpChanged`와 `final_damage`가 피해·HP·사망의 source of truth다. Unity는 다음
`feedback_tags`만 표시하고 방어 계산을 재구현하지 않는다.

- `critical`: 치명타
- `mitigated`: 크게 감소한 피해
- `fixed_damage`: 고정 피해
- `immune`: 피해 또는 상태가 완전히 막힘

`piercing`, `shield`, `blocked`, `resisted_status`는 현재 공식 피해 feedback tag가 아니다.

### Runtime 스탯·HP·사망 전이

전투 중 스탯 modifier는 causal unit이 있는 skill/장비 effect와 system/직접 command처럼 source가 없는
경우를 구분한다. Persistent battle-entry stat 조합은 이 runtime event 경로를 사용하지 않는다.

- 모든 runtime modifier는 산술과 현재 HP clamp를 한 번 계산하고 `StatChanged`를 한 번 기록한다. Skill은
  caster, 장비 trigger effect는 trigger owner를 `source_instance_id`로 보존하며 source 없는 command는
  `None`을 유지한다.
- MaxHealth 변경으로 current HP가 실제로 줄어들면 `StatChanged` 다음에 damage metadata가 없는
  `HpChanged(reason: Command)`를 정확히 한 번 기록한다. HP delta가 없으면 `HpChanged`를 만들지 않으며,
  MaxHealth 증가가 current HP를 자동 회복시키지 않는다.
- Clamp 결과가 0이면 같은 전이 안에서 기존 death finalization을 실행한다. Active 0 HP를 terminal state로
  남기지 않고 movement/action, buff, graveyard와 `UnitDied`, `OnDeath`/`OnKill`/`OnAllyDeath`를 일반 사망과
  같은 authority에서 한 번만 처리한다.
- Causal unit이 있으면 그 unit이 killer이고 일반 OnKill credit을 받는다. Source가 없는 system/직접
  modifier 사망은 killer와 OnKill credit이 없다.
- 관찰 가능한 인과 순서는 `StatChanged → HpChanged → 기존 death sequence`다. MoveSpeed modifier의 runtime
  body speed 갱신과 다른 스탯의 기존 산술·rounding은 그대로 유지한다.

## 성장, 스킬 파편과 도감

이 장은 전투와 node resolution 뒤 플레이어가 얻거나 누적하는 보상, 성장, 장비, 스킬 파편과
도감의 gameplay 의미를 소유한다. 수량 commit과 runtime transaction은
[core_runtime_contract.md](../core_runtime_contract.md), authored schema와 catalog validation은
[data_loading_contract.md](../data_loading_contract.md)가 소유한다.

### 성장 체계

**합의 변경 — 유물 시스템 노트.**

| 대상 | 차기 성장 역할 |
| --- | --- |
| 무기 | 일반 강화로 해당 무기가 제공하는 공격력·주문력을 높임 |
| 방어구·액세서리 | 능력치와 자체 특수 효과를 제공. 효과 복잡성을 제한하지 않으며 일반 강화는 없음 |
| 스킬 파편 | 일반 강화 제거. 개화로 새 형태를 해금하고 기존 형태와 선택 가능 |
| 유물 | 기존 효과 강화와 새 효과 부여로 조합 구성. 자체 레벨·반복 강화 없음 |
| 직원 | 기존 XP·level을 비교 기준으로 보존. 제거 제안은 미승인 |

무기 강화 한도 +3, 층별 강화 개방과 투자 가루 75% 회수는 초기 AI 제안이다. 확정된 일반 강화
제거 범위와 기존 강화 진행·보상·세이브의 전환 방법은 별도로 다룬다.

### 공격력·주문력과 파편 피해

**합의 변경:** 직원의 공격력과 주문력은 별도 능력치다. 무기는 둘 중 어떤 능력치를 얼마나
제공하는지 명시하며, 모든 무기가 둘 다 제공한다고 가정하지 않는다.

- 파편마다 공격력·주문력·둘 모두 중 참조할 능력치와 계수를 명시한다.
- 무기 강화는 그 무기가 제공하는 능력치를 높이고, 이를 참조하는 파편의 피해도 함께 증가시킨다.
- 파편 일반 강화가 없어도 직원·무기의 성장과 유물 효과에 따라 실제 위력은 달라진다.
- 아래 식은 개념식이며 실제 계수·피해 산술·반올림·능력치 합성은 후속 설계한다. 기존 runtime에
  같은 이름의 독립 주문력 필드가 있다고 가정하지 않는다.

```text
공격형 파편 = 기본 피해 + (직원 공격력 + 무기 공격력) × 공격 계수
주문형 파편 = 기본 피해 + (직원 주문력 + 무기 주문력) × 주문 계수
혼합형 파편 = 기본 피해
            + (직원 공격력 + 무기 공격력) × 공격 계수
            + (직원 주문력 + 무기 주문력) × 주문 계수
```

### 보상과 무기 강화 재료

**합의 변경:** 무기 강화에 필요한 필수 재료는 일반 침식체 전투를 선택해야 얻는다. 일반 전투를
여러 대체 가능한 획득처 중 하나로만 취급하지 않는다. 정확한 종류·수량·지급 조건·강화 비용과
일반 전투의 다른 보상은 미정이다.

유물 분쇄로 기존 장비 가루를 얻는 합의도 유지한다. 다만 장비 가루만 있으면 무기를 강화할 수
있도록 연결하면 일반 전투를 생략할 우회 경로가 생길 수 있다. 이 연결은 **미해결 정책 경계**다.

함께 결정할 항목:

- 유물·장비 분쇄 가루와 무기 강화 필수 재료의 관계.
- 시작 보급, 보스 보상, Event·보급실·상점이 일반 전투의 재료 획득을 대체하는지.
- 강화 장비 분쇄의 투자 재료 회수가 최초 획득을 우회하거나 재료를 증식시키는지.
- 별도 재료 추가, 기존 가루의 용도·획득처 조정, 분쇄 정책 조정 중 채택할 방법.

이 문서에서는 새 재화나 유물 분쇄 폐기를 정하지 않는다. 기존 보상 recipe는 새 경제의 확정
예산이 아니며, 특히 보스·상점·Event에 무기 강화 필수 재료를 자동 배정하지 않는다.

기존의 통화·장비·성장 보상, 성공·실패의 지급 구분과 원자적 결과 확정은 비교 기준으로 보존한다.
스킬 파편은 아래 환상체별 전투 조건 또는 Standard의 명시적 튜토리얼 script를 기존 지급 기준으로
삼으며, 획득 구조 재설계는 아직 동결되지 않았다. `ResearchProgress`는 폐기 대상이다.
기존 지급 정책에서 Ordinary Combat, Reward node, Shop, HeadquartersContact와 일반 authored reward
pool은 스킬 파편을 지급하지 않는다. 신규 출발 지급·첫 제압 지급의 채택 여부와 구분해서 보존한다.
Live encounter의 일반 자동 보상 `ClaimAll`은 기존 기준이며, 유물 획득의 선택 방식은 따로 미정이다.

### 직원 XP와 성장

**기존 기준:** 직원 XP·level·battle tier 제거는 승인되지 않았다.

- 새 직원은 level 1, 현재 level XP 0에서 시작한다.
- XP requirement, 생존 XP와 battle tier는 `run/policy.ron`이 소유한다. 기존 live 값은 모든 level
  100 XP, 전투 결과 생존 직원은 승패와 무관하게 10 XP, tier는 level 1/3/6부터 I/II/III다.
- 같은 직원이 여러 번 배치돼도 생존 XP는 한 번만 지급하고 retreat에는 지급하지 않는다.
- 한 grant가 여러 level을 넘을 수 있고 남은 XP는 다음 level에 유지된다. Battle tier는 현재 stat
  multiplier가 아니다.
- `GrantExperience.amount`는 대상 직원 한 명당 지급량이다. `amount: 20`, 대상 3명이면 각자
  20 XP를 받는다. `AliveRoster`는 생존 직원, `CombatParticipants`는 명시 전투 참가자,
  `SelectedEmployee`는 선택 직원, `AllRoster`는 생사와 무관한 모든 직원을 대상으로 한다.
  정확한 대상 집합과 staging은 [core_runtime_contract.md](../core_runtime_contract.md#reward-and-employee-growth-boundary)가 소유한다.
- `GrowthStack`은 XP/level과 별도이며 현재 production producer가 없고 level-up으로 증가하지 않는다.

### 연구도 폐기와 스킬 파편 획득

> **기존 기준 / 획득 재설계 미정:** 아래 Elite 3승·Boss 승리 지급과 숙달 소탕의 재획득을
> 보존한다. 시작 파편 지급, Elite 첫 승리 지급, 모노리스·숙달·소탕 제거는 미승인 제안이다.
> 필수 관문 도입 이후 정확한 보상·반복 조우 구조는 후속 결정한다.

기존의 기본 연구도, 환상체 대응 연구, 본사 분석·송신, `research_progress`,
`skill_fragment_progress`, `pending_research_deliveries`와 pending 수령 절차는 모두 폐기한다.
연구 완료를 스킬 파편 지급이나 Endless 종료 조건으로 사용하지 않는다.

게임에서 별도 수식어 없이 말하는 “파편”은 스킬 파편 하나만을 뜻한다. 환상체와 파편의 관계는
`환상체 1 : 스킬 파편 N`이다.

- 모든 환상체는 확장에 따라 두 개 이상의 고유 스킬 파편을 가질 수 있다.
- 각 스킬 파편은 정확히 하나의 환상체에만 대응한다.
- 일반적인 환상체는 파편 하나만 가진다. 파편 지급 source는 RON에 정확한 `fragment_id` 하나를
  명시하며 플레이어 선택, 무작위 추첨이나 대응 파편 일괄 지급을 하지 않는다.
- 미래에 한 환상체가 여러 파편을 가지더라도 각 획득 조건은 RON이 가리킨 파편 하나만 지급한다.
- 같은 run에서 해당 reward source의 파편 지급 완료 ledger가 이미 기록된 뒤 같은 조건을 다시
  만족하면 RON의 명시적 `repeat_reward`를 적용한다. 강화 재료, XP 등의 effect 또는 `None`을 직접
  작성하고 암묵적 대체 보상은 만들지 않는다.
- 유일한 예외는 숙달 Elite **소탕**이다. 해당 source의 지급 완료 ledger는 남아 있지만 RON의
  `fragment_id` 보유 count가 0이면 `repeat_reward` 대신 같은 파편 한 개를 재지급한다. 한 번의 소탕은
  재획득과 `repeat_reward`를 함께 지급하지 않는다. Count가 1 이상이면 기존 `repeat_reward`를 지급한다.
  직접 전투, NormalBoss와 FinalBoss에는 이 재획득 예외를 적용하지 않는다.
- 마지막 사본을 분쇄하고 이후 숙달 Elite를 소탕하는 분쇄→재획득 cycle은 횟수 제한 없이 허용한다.
  재획득은 지급 완료·숙달 ledger를 reset하지 않으며 identity별 개화 진행도를 보존한다. Cycle의
  실제 빈도는 숙달 repeat pool 배정과 소탕 가능 조건이 제한한다.

Endless의 정규 획득 trigger는 encounter 등급으로 결정한다.

- `Elite`: 같은 run에서 동일한 Elite 환상체를 세 번째로 격파한 승리 결과에서 파편 지급 trigger가
  성립한다. 최초 3회는 직접 전투해야 하며 소탕으로 달성할 수 없다.
- `NormalBoss`: 해당 보스 node의 승리 결과에서 파편 지급 trigger가 성립한다.
- `FinalBoss`: 해당 보스 node의 승리 결과에서 파편 지급 trigger가 성립한다.
- `ReferenceFinalBoss`: 파편과 repeat reward trigger가 없으며 production reward source로 authoring할 수 없다.
- 지급 또는 `repeat_reward`는 전투 결과 commit에 포함하며 본사 수령을 기다리지 않는다.
- 파편과 획득 진행은 run-local이다. 한 run의 savefile은 다른 run의 파편, 격파 횟수와 숙달에
  영향을 주지 않는다.
- Standard가 중간에 지급하는 임의 파편은 사용법 학습을 위한 명시적 예외이며 위 trigger나 다른
  run의 소유권을 만들지 않는다.

### 영구 도감, 튜토리얼 해금과 개인 기록

게임은 플레이어가 승리한 attempt에서 실제로 조우한 환상체와 침식 프로필의 로컬 영구 도감을 유지한다.
이는 전투 history나 재생 기록이 아니라 canonical ID의 단일 `registered_ids` set이다.

- 전장에 실제 spawn된 적군 `Abnormality`/`CorrodedEmployee`만 후보다. 노드 preview, briefing, 아직 spawn되지
  않은 Wave와 `SummonedUnit`은 도감에 올라가지 않는다.
- Final Victory commit만 attempt의 모든 후보를 등록한다. 후보의 사망·생존, primary 여부와 개별 credit은
  관계없으며 Defeat, retry exhaustion, Retreat와 process 종료는 등록하지 않는다.
- 커밋된 set은 local-install profile의 단조 union이며 checkpoint load, 새 run, RunFailed, 일반
  non-battle transaction 실패와 active-run transaction rollback으로 사라지지 않는다.
  Run/checkpoint/transport player UUID와 분리되고, title의 확인된 전체 profile reset만 삭제할 수 있다.
- profile에는 ID set만 저장하고, 표시 이름·위험 등급과 authored 순서는 현재 live catalog에서 해결한다. 현재 catalog에서 사라진 dormant ID는 profile에 보존하지만 current snapshot에는 투영하지 않는다.
- `RunSnapshotDto.encounter_compendium`은 현재 catalog 전체를 authored 순서로 투영한다. 미등록 항목은 ID와
  `registered:false`만 가진 `???` placeholder이고 등록 항목만 metadata를 공개한다.
- 범위 안 consumer는 Core projection과 host의 `player_profile_persistence` 상태만 표시하며 battle event,
  preview, local catalog로 발견·격파·튜토리얼 완료·개인 기록을 재계산하지 않는다.
- 도감과 별도로 `standard_tutorial_completed`에 해당하는 영구 플래그 하나를 유지한다. Standard
  Floor 3의 간소화된 NormalBoss 승리를 최종 commit할 때만 false에서 true가 되며, true이면 Endless
  진입을 허용한다. 이 플래그는 run 성장이나 파편 소유권이 아니다.
- 영구 profile은 Endless에서만 집계하는 순수 기념용 개인 기록으로 `최고 도달 Floor`, `최장 무사망
  연속 Floor`, `한 run에서 모든 FinalBoss 격파`를 보존한다. Standard는 세 기록에 기여하지 않는다.
- 수치 기록은 기존 최고값을 넘는 즉시 갱신하고, 모든 FinalBoss 격파는 한 Endless run에서 달성
  시점의 non-empty BossOmen chain registry를 모두 완료하면 영구 달성한다. `ReferenceFinalBoss`는 이 집계에
  포함하지 않는다. 이후 새로운 production FinalBoss가
  추가되어도 이미 달성한 기록을 취소하지 않는다.
- 개인 기록은 active run savefile과 분리해 조건 달성 즉시 profile에 commit한다. 과거 checkpoint나
  run savefile load, Run 실패와 새 run으로 되돌리지 않으며 title의 확인된 전체 profile reset만
  삭제할 수 있다. 기록은 능력치, 장비, 재료, 재화, 파편과 콘텐츠 해금을 지급하지 않는다.
- 게임 전역에 보존되는 gameplay profile은 도감, 튜토리얼 완료/Endless 해금과 위 개인 기록뿐이다.
  직원, 장비, 재화, 파편, 유물, 격파 횟수, 숙달, 모노리스와 pool 상태는 active run savefile에만 속한다.

### Endless 환상체 격파와 반복 조우

> **기존 기준 / 관문 연결 미정:** 이 장의 pool·모노리스·소탕 정책은 삭제 승인이 없어 보존한다.
> 필수 Elite·NormalBoss의 배치, 스킵 가능성, 반복 전투 대체와 패배 소진 처리를 새 진행에 연결하는
> 방법은 미정이다. 아래 자연 배정·미방문 반환·패배 소진을 새 관문 정책의 예외로 자동 채택하지 않는다.

Endless는 Elite/Boss/FinalBoss 환상체별 격파와 파편 지급 상태를 run-local ledger로 보존한다.
연구 point, requirement와 completion percentage는 갖지 않는다.

- Elite의 세 번째 승리는 해당 run의 숙달 조건을 만족한다.
- 숙달은 파편 보유와 분리된 격파 ledger다. 지급받은 파편을 장착·소비·분쇄하거나 더 이상 보유하지
  않아도 같은 run의 숙달과 소탕 자격은 유지한다.
- 파편 지급을 해결한 뒤 동일 환상체를 다시 격파하는 목적은 추가 성장 보상이다.
- 모든 환상체의 ledger를 채우거나 모든 FinalBoss를 격파해도 Endless를 완료하지 않는다.
- run-local pool, active chain, 숙달, 완료, 파편과 보상은 checkpoint에 포함한다. 과거 checkpoint를
  load하면 함께 rollback하고 전역 도감, 영구 개인 기록과 단조 증가하는 `loads_used`는 되돌리지 않는다.

#### Elite assignment pool

Elite 자연 배정은 `미숙달 pool`과 `숙달 반복 pool`을 명시적으로 구분한다.

- 미숙달 pool은 아직 자연 승리하지 않은 모든 Elite identity로 시작한다. 각 Floor의 Elite slot은
  same-Floor 중복 금지를 적용한 미숙달 후보로 먼저 채운다. 현재 Floor에 배정 가능한 서로 다른
  미숙달 후보 수가 Elite slot보다 적을 때만 남은 slot을 숙달 반복 pool 후보로 채운다.
- 자연 출현은 `Stage 1`이다. 자연 승리는 해당 identity를 미숙달 pool에서 제거하고 모노리스 진행을
  연다. 모든 node attempt를 소진해 현재 node가 패배 완료돼도 영구 실패로 기록하지 않고 미숙달
  pool에 남긴다. 같은 Floor 중복 금지는 유지하며 이후 Floor에서 자연 승리할 때까지 다시 후보가 된다.
- Floor에 배정됐지만 방문하지 않은 identity는 해결된 것이 아니며 다음 Floor의 미숙달 pool 후보로
  되돌린다.
- 자연 승리 뒤 모노리스 Stage 2·3까지 총 세 번 승리하면 숙달 반복 pool에 추가한다.
- 숙달 반복 pool은 첫 숙달 identity가 생기면 활성 상태를 유지하되 위 미숙달 우선 규칙 뒤의 남은
  slot에만 사용한다. 미숙달/숙달 양쪽에서 유효한 고유 후보가 없으면 authored Elite slot을 일반
  침식체 Combat으로 바꾼다.
- 숙달 반복 pool은 한 cycle에서 각 identity를 정확히 한 번씩 run seed 기반 결정론 순서로 배정하고,
  전부 소진하면 다시 채운다. 최근 Floor 제외는 없으며 cycle 경계에서 같은 identity가 연속 Floor에
  등장해도 허용한다.
- 숙달 identity도 Floor에 배정됐지만 직접 전투나 소탕으로 해결하지 않고 Gate를 통과하면 현재
  repeat cycle 후보로 되돌린다.
- 같은 Floor에는 동일한 Elite identity를 둘 이상 배정하지 않는다. cycle reset 뒤에도 현재 Floor에
  이미 배정한 identity를 다시 쓰지 않으며, 서로 다른 숙달 후보가 부족한 나머지 Elite slot은 일반
  침식체 Combat으로 바꾼다.

#### NormalBoss assignment pool

- NormalBoss pool은 한 cycle에서 각 authored identity를 한 번씩 해결한 뒤 전부 다시 채운다.
- 승리하면 현재 cycle에서 소진하고 해당 run의 지급 완료 ledger가 없으면 RON의 `fragment_id`, 이미
  있으면 `repeat_reward`를 지급한다. NormalBoss에는 소탕을 적용하지 않는다.
- 모든 attempt를 소진한 패배는 현재 cycle에서만 소진된 것으로 본다. 다음 cycle에서는 다시 후보가
  되어 파편 획득에 재도전할 수 있다.
- Floor에 배정됐지만 방문하지 않은 NormalBoss는 해결된 것이 아니며 현재 cycle pool로 되돌린다.

### Elite 모노리스와 강화 재현

Endless의 모노리스는 이미 자연 출현한 Elite를 더 강한 상태로 재현해 성장 보상과 세 번째 격파의
파편 지급 trigger를 노리는 선택형 콘텐츠다. 모든 Elite 모노리스를 수행할 의무는 없으며 플레이어가
필요한 성장과 파편에 따라 대상을 고른다. Standard에는 모노리스 규칙을 적용하지 않는다.

- 자연 Elite 최초 승리만 해당 환상체의 모노리스 구매 자격을 연다. 한 identity는 run당 한 번만
  구매할 수 있고 3 uses를 가진다.
- Endless Floor 1~3에는 떠돌이 상인이 없다. Floor 4+의 전용 상인은 첫 page 3칸에 eligible
  모노리스를 최대 2개 배치하고 나머지는 고위험 consumable로 채운다. 한 번의 유료 refresh는 page
  전체를 item-only 3칸으로 바꾼다.
- NodeConfirm은 순수 Available Normal Combat에서만 `모노리스 사용 (N)`과 환상체별 카드를 투영한다.
  최종 확정은 node 구조를 유지하고 encounter/difficulty를 다음 stage로 바꾸면서 1 use를 차감한다.
- 강화 재현은 Stage 2와 3이며 각 node에서 최대 2회 시도한다. 첫 패배는 같은 node를 재시도하고 모든
  attempt를 소진한 패배는 해당 identity를 run-local 영구 실패로 기록한다. 영구 실패 뒤에는 다음
  stage, 파편 지급, 숙달 반복 pool과 소탕을 열지 않는다. Retreat는 attempt를 소비하지 않는다.
  자연 Elite, Stage 2, Stage 3의 승리를 같은 identity의 1·2·3번째 격파로 세며 Stage 3 승리는 RON이
  명시한 `fragment_id`의 지급 trigger와 숙달을 함께 성립시킨다.
- 변환된 difficulty는 typed stage를 소유하고 monolith ledger의 stage와 반드시 일치한다. Stage 1
  자연 출현을 기준으로 Stage 2의 초기 스탯 가중치는 150%, Stage 3은 200%다. 두 값은 개발 과정의
  플레이 검증으로 조절 가능한 RON tuning이며 gameplay 구조 정책을 다시 열지 않고 변경할 수 있다. 어떤 스탯과 encounter
  구성에 가중치를 합성할지, 정확한 반올림·웨이브·보상 값은 각 RON이 직접 작성하고 공통 규칙으로
  추론하지 않는다.
- Stage 보상은 Monolith role multiplier와 먼저 합성한 뒤 원래 authored 수량에 한 번만 적용하고
  한 번만 반올림한다. Unity에는 exact multiplier 대신 `enhanced`/`final_enhanced` tier만 투영한다.
- stage 3 승리 시 use가 정확히 1 남으면 source identity를 가진 잔재로 전환한다. 서로 다른 경로에서
  모은 잔재가 네 개가 되면 다음 Floor의 Shop 하나를 보장한다. 상인은 Shop보다 먼저 일회성 VN으로
  oldest four 교환을 제시하며 수락은 run-local 변질된 모노리스를 주고 거절은 같은 run에서 비가역적이다.
- 변질된 모노리스는 현재 runtime에서 consumer가 없는 dormant run-local key다. 숨겨진 보스,
  관리자 연출과 지속형 capstone의 target gameplay는 아래 FinalBoss 정책을 따르며 구현은 후속이다.

Bonus objective:

- Encounter가 명시한 objective만 존재하며 primary abnormality가 없는 Normal encounter에는 없다.
- 승리 결과에서 완료 event log/result stats로 평가하고 objective별 전투당 한 번만 추가 성장
  보상을 지급한다. 각 objective의 RON이 강화 재료와 `GrantExperience`를 포함한 reward effect와
  정확한 수량을 직접 명시하며 여러 effect를 함께 가질 수 있다. 스킬 파편 획득 조건에는 영향을
  주지 않는다.
- Bonus `GrantExperience.amount`는 나눠 갖는 total pool이 아니라 직원별 지급량이다. 배치, 무장과
  현재 전투 가능 여부에 관계없이 결과 commit 시 roster의 모든 살아 있는 직원이 각자 전액을 받고
  `Dead` 직원만 제외한다. 직접 level을 지급하지 않고 기존 XP/level-up 규칙을 따른다.
- `ClearWithin(time_ms)`는 battle start부터 victory까지의 시간을 검사하며 늦게 spawn하는
  boss에는 사용하지 않는다.
- `DecisiveDamage(percent)`는 primary abnormality에게 한 공격 원천이 준 non-DoT 실효 피해가
  최대 HP의 지정 비율 이상인지 정수 비교한다. 여러 tick을 합산하지 않고 마지막 일격일 필요도 없다.
- “환상체 부위”는 별도 스킬 파편이 아니라 bonus 강화 재료를 표현하는 presentation으로 사용할 수
  있다.

### 숙달 Elite 소탕

소탕은 이미 파편 획득 단계까지 직접 전투한 Elite를 같은 run에서 다시 만났을 때 반복 성장의
피로를 줄이는 빠른 전투다.

- 현재 run에서 동일한 Elite 환상체를 이미 세 번 격파한 경우에만 소탕 후보가 된다.
- 최초 세 번의 격파, Standard, NormalBoss와 FinalBoss에는 소탕을 적용하지 않는다.
- 숙달 Elite node에서는 `직접 전투`와 `소탕`을 함께 제시하고 플레이어가 선택한다. 소탕도 일반
  전투 진입과 마찬가지로 출전 가능하며 무기를 장착한 직원이 최소 한 명 있어야 한다.
- 플레이어가 소탕을 선택하면 실제 전투 setup/update simulation을 시작하지 않고 전투와 독립된
  typed 소탕 결과를 만든다.
- 소탕도 해당 node를 해결하는 명시적 선택이며, 결과 확인과 최종 commit 경계를 우회하지 않는다.
- 직접 승리와 같은 authored 기본 보상을 지급한다. Source fragment 보유 count가 0이면 같은 파편 한
  개를 재지급하고 `repeat_reward`는 지급하지 않으며, count가 1 이상이면 RON `repeat_reward`를 지급한다.
  분쇄→재획득 cycle은 의도적으로 허용한다. 기본 전투 XP도 배치·무장·condition과 관계없이 roster의
  살아 있는 모든 직원에게 각자 전액 지급하고 Dead만 제외한다.
- 실제 전투가 없으므로 physical damage, trauma, 전투불능과 사망을 만들지 않고 모든 직원 condition을
  그대로 유지한다. `ClearWithin`/`DecisiveDamage`를 평가하거나 bonus 보상을 자동 지급하지 않는다.
- 일반 승리처럼 활성 소비 아이템의 node 단위 duration을 한 번 차감하고 결과 commit에서 node를
  `Completed`로 소비한다.
- battle record와 전투 통계는 만들지 않는다. 첫 command는 기본 보상, 재획득 파편 또는
  `repeat_reward` 중 하나와 직원별 XP를 담은 동일한 typed 결과를 stage하고, 확인 command가 그 저장된
  계획을 재계획 없이 원자 commit한다. 정확한 DTO와 transient state 경계는 runtime contract가 정한다.

### 장비

장비는 일반 장비와 E.G.O 장비로 나뉘며 무기, 방어구, 악세서리 슬롯을 사용한다.

- 무기는 기본 공격·range·targeting과 block 수치를 제공한다. 각 무기의 필수
  `weapon_profile.block_capacity`가 직원의 기본 저지 수를 단독으로 결정하며, 직원 class/
  `CombatAptitude`나 방어구는 이 값을 변경하지 않는다. 무기가 없으면 0이다.
- 각 무기는 `deployment_affinity`와 usable tile별 profile을 직접 소유한다. `ground_only`는
  `ground_defense_focus`만, `platform_only`는 `platform_rest`만, `any`는 둘 다 명시해야 하며 runtime은
  archetype, employee profile, effective block 또는 default에서 값을 보완하지 않는다.
- First-live 30개는 Staff 16, Bow 1, Gun 1, Sword 10(training 포함), Spear 1, Shield 1이다. Staff는
  PlatformOnly/base block 0/Rest HP 3·resonance 5, Bow/Gun은 GroundOnly/base 0/DefenseFocus +0·10%,
  Sword/Spear는 GroundOnly/base 1/+1·10%, Shield는 GroundOnly/base 3/+1·10%다. 이 authored
  DefenseFocus/Rest 수치는 위 Attack/Defense 타일 효과 계약으로 전투 중 적용된다.
- Fragment의 `block_capacity_min`과 certified combat power는 타일 효과가 더해진 값이 아니라 위
  `weapon_profile.block_capacity`만 사용한다. 방어구는 생존 stat과 utility를, 악세서리는 보조 stat과
  utility를 제공할 수 있다.
- 비무장 직원은 safe-zone roster와 equipment target으로 유지된다. condition상 이미 출전 불가능하면
  그 condition reason이 우선하고, condition상 가능하지만 무기가 없으면 battle command/capability는
  `weapon_required`로 거부한다. 배치, 재배치, 배치 범위 preview와 그 이후 battle gameplay는 fallback
  profile을 만들지 않는다.
- **합의 변경:** 일반 강화는 무기에만 적용하고 그 무기가 제공하는 공격력·주문력을 높인다.
  방어구·액세서리는 자체 특수 효과를 가질 수 있으며 일반 강화는 없다.
- 무기 강화 상태는 definition 공통값이 아니라 보유 instance를 따른다. 기존 방어구 등 강화 상태의
  세이브 전환 방법과 새 snapshot shape는 미정이다.
- 기존 장비 가루와 파편 가루는 별도 자원이며 파편 가루는 개화 재료로 유지한다. 장비 가루와
  무기 강화 필수 재료의 정확한 연결은 위 재료 경제의 미해결 항목이다.
- 장비 제작은 현재 Maintenance action이 아니다. 이 정리로 제작이나 강화 자동 이전을 추가하지 않는다.

#### 장비 전투 트리거

장비 runtime instance는 소유 유닛의 `OnBattleStart`, `OnAttack`, `OnHit`, `OnKill`, `OnDeath`,
`OnAllyDeath` 발생을 하나의 typed occurrence 경로로 처리한다.

- 같은 발생에서는 장비 instance UUID 오름차순으로 정의를 모으며 일반 effect를 ability activation보다
  먼저 실행한다. 여러 유닛이 참가하는 전투 시작과 아군 사망 처리는 runtime unit UUID 오름차순이다.
- `OnBattleStart`는 effect와 activation을 모두 실행한다. 따라서 `parasite_seed`의 `AllAllies` Attack
  `+6%`처럼 이미 출판된 시작 effect도 살아 있는 아군 각각에게 정확히 한 번 적용된다.
- `OnAttack`과 `OnHit`는 현재 기본 공격의 확정 hit에만 적용한다. 추가 피해와 피해 보정은 해당 hit의
  명시 입력으로 유지하며 일반 skill, buff tick, area damage가 이 두 장비 trigger를 새로 발생시키지 않는다.
- `OnDeath`는 사망한 장비 소유자의 activation에만 dead-caster 실행을 허용한다. `OnKill`은 killer,
  `OnAllyDeath`는 같은 편의 살아 있는 나머지 유닛에게 발생하며 각 effect는 자기 occurrence의 activation보다
  먼저 실행된다.
- `OnHit + CounterpartUnit + BonusDamage`의 counterpart는 공격자다. 이 조합은 피격자의 수신 피해를
  증폭하지 않고, 확정된 원 hit snapshot에서 authored 추가 피해를 한 번 계산해 effect 소유자 attribution으로
  공격자에게 반격한다. authored effect마다 한 번만 적용하며 이 반격 피해는 `OnAttack`/`OnHit` 장비 trigger를
  재수집하지 않아 상호 반사 loop를 만들지 않는다. 공격자가 없거나 이미 죽었으면 재지정하거나 원 hit를
  증폭하지 않는다.

### 스킬 파편

직원은 기본 고유 active skill 대신 장착한 skill fragment의 대표 active skill을 사용한다.

- 한 환상체는 두 개 이상의 서로 다른 skill fragment를 가질 수 있고 각 fragment는 정확히 하나의
  원본 환상체를 가리킨다. 따라서 schema와 UI는 환상체당 단일 fragment를 전제로 만들지 않는다.
- 기본 active fragment slot은 1개다. 환상체 원본 skill과 직원용 모방 skill은 별도 data다.
- Fragment는 passive 장비가 아니며 평타 강화, 일시 stat, buff와 공명 변화도 active skill 결과로
  표현한다.
- 위 두 항목은 정식 fragment 정책이다. 현재 abnormality-derived 28개와 concept starter 1개의
  직접 `BasicAttackModifier`는 dedicated active skill 제작 전의 temporary placeholder이며 정식
  정책이 구현 완료됐다는 근거가 아니다. 이 correction에서는 해당 live data를 변환하지 않는다.
- **합의 변경:** 파편 일반 강화는 제거한다. Maintenance가 개화·형태 선택·분쇄를 소유한다.
  다만 BossOmen처럼 RON이
  명시한 encounter progression은 결과 commit에서 같은 fragment identity의 authored 형태를 전환할
  수 있다. 이 경우 정적 fragment data를 덮어쓰거나 새 파편을 추가 지급하지 않고 run-local 현재
  형태·단계만 갱신한다. 정확한 최초 지급 source와 step별 형태는 각 chain RON이 명시한다.
- 장착 조건은 직원 직업명이 아니라 장착 무기의 combat profile을 사용한다. Melee/ranged 또는
  특정 archetype 제한을 허용하되 모든 fragment를 모든 무기에 강제 호환하지 않는다.
- 최종 장착 가능성은 core validation이 소유한다. 콘텐츠 설계와 live 목록은 `docs/skills/`를
  함께 확인한다.

### 개화 형태와 재료

**합의 변경 — 유물 시스템 노트.**

- 개화는 새로운 형태를 해금한다. 개화 전 형태도 유지하며 Maintenance에서 사용할 형태를 선택한다.
- 파편 가루는 개화 재료로 유지한다. 환상체 파츠 같은 추가 재료를 요구할 수 있다.
- 원본 환상체 이외의 환상체에서 얻는 재료도 요구할 수 있고, 이를 선행 콘텐츠와 밸런스 조절에
  활용한다. 대상 환상체·재료·요구 수량은 각 콘텐츠에서 정한다.
- 일반적인 모든 개화에 같은 보스 핵을 요구하거나 특정 층에 첫 개화를 보장하는 정책은 미정이다.
- 기존 파편 강화 진행·보상·세이브 전환, 해금 형태와 선택 형태의 저장·조건 판정은 후속 검토 대상이다.

## 유물

이 장은 [유물 시스템 노트](../tmp/run_relic_system_design_notes_2026-08-21.ko.md)의 사용자 결정을
통합한다. 별도 미정 표시가 없는 공통 규칙은 **합의 변경**이며 신규 시스템의 목표다. 과거 제거한
Artifact의 타입·획득·효과 규칙을 복원하는 근거로 사용하지 않는다.

### 목적과 효과 범위

- 런마다 서로 다른 무기·파편 조합과 전투 빌드를 만든다.
- 범용 유물은 효과 태그와 세부 속성으로 무기·파편의 효과를 강화하거나 새 효과를 부여한다.
- 전용 유물은 명시한 파편·장비의 고유 구조를 확장한다. 대상과 기본형·개화형 등 적용 형태를
  각 유물에 명시한다.
- 유물의 효과 범위는 전투 안이다. 상점 가격·보상량·맵 생성에 직접 영향을 주는 효과는 제외한다.
  획득·보관·장착·분쇄는 별도의 비전투 관리 규칙이다.
- 무기에 적용할 유물의 첫 설계 범위는 단순 수치 강화까지다. 무기마다 공격 방식·변신 규칙을
  바꾸는 전용 유물 제작을 현재 필수 과제로 두지 않는다. 기존 파편 변형·연결 설계와 장비 효과는 유지한다.

### 성장과 희귀도

- 유물 자체의 레벨·반복 강화 단계는 없다. 같은 유물 재획득은 독립 사본 추가다.
- 일반·희귀·특급의 세 가지 획득 희귀도를 사용한다.

| 희귀도 | 설계 방향 |
| --- | --- |
| 일반 | 자주 만나며 조합의 기초가 되는 선택 |
| 희귀 | 특정 운용·조합에서 높은 효과를 내는 선택 |
| 특급 | 런에 미치는 영향이 커 드물게 제공할 선택 |

희귀도는 강화 단계나 강화형·전문화형·연결형·전환형 역할 분류와 독립적이다. 효과 복잡성만으로
등급을 정하지 않으며 일반 유물도 끝까지 사용할 가치가 있어야 한다. 등급별 확률·획득처·후보
구성은 미정이다. 첫 유물 `깊은 쐐기`는 희귀로 합의했으며 `때린 곳 또 때리기`는 대체 이름이다.

### 슬롯·사본·소유

- 직원마다 장비 장착 여부와 무관한 공용 유물 슬롯 3개를 제공한다. 공격·방어·보조나 장비 부위로
  슬롯 용도를 나누지 않는다.
- 장비 해제로 슬롯이 닫히거나 장착 유물이 자동 회수되지 않는다.
- 직원당 3개 이외에 roster 전체의 장착 합계 상한은 없다. 중복 사본을 포함한 런 보유량도 무제한이다.
- 유물은 런 공용 자산이며 직원·장비에 귀속하지 않는다. 사본을 직원 사이에서 옮길 수 있다.
- 각 사본은 슬롯 하나를 차지한다. 하나의 사본을 여러 직원·슬롯이 동시에 공유할 수 없다.
- 직원 사망 시 장착 유물 전부를 런 공용 보관함으로 회수한다.

| 중복 장착 태그 | 같은 직원에게 같은 종류를 장착할 때 |
| --- | --- |
| 중복 가능 | 보유한 독립 사본들을 여러 슬롯에 장착 가능 |
| 중복 불가 | 같은 종류는 최대 1개 |

중복 불가 유물도 별도 사본을 서로 다른 직원에게 장착할 수 있다.

### 장착 변경과 비활성

- Safezone과 Maintenance에서 장착·해제·교체·직원 간 이전을 한다. Safezone에는 지도 보기와
  전투 진입 전 준비가 포함된다. 전투 중에는 변경할 수 없다.
- 허용 시점의 변경에는 재화 비용·횟수 제한이 없고 슬롯·중복·비호환 제한을 적용한다.
- 무기·파편 교체로 요구 태그가 사라져도 유물은 슬롯에 남고 효과만 비활성화한다. 다시 조건을
  충족하면 자동 활성화한다.
- 최초 장착 때부터 조건이 없어도 장착할 수 있다. 다른 유물이 부여한 효과의 태그로 조건을
  충족할 수도 있다.
- 전용 유물은 지정된 파편·장비·형태 조건도 충족해야 한다. 일반 태그를 부여한 것만으로 전용
  대상 조건을 대체할 수 없다.
- 비활성 상태도 중복·비호환 제한 위반을 허용하지 않는다.

### 태그·새 효과·증폭

- 태그는 적용 대상을 판별하고 실제 변화는 유물 효과 정의가 소유한다. 예를 들어 스택의 획득량·
  최대치·스택당 효과, 관통 횟수·피해 감소 완화, 폭발 반경·피해 등을 강화할 수 있다.
- 유물이 부여한 새 효과에도 태그와 출처를 붙인다. 다른 유물이 그 효과를 강화할 수 있다.
- 같은 효과에 대한 동일 종류의 피해 증폭은 합산한다. 예시로 폭발 피해 +20%와 +30%는 +50%다.
  예시 수치는 실제 유물 수치를 확정하지 않는다.
- 유물 피해 증폭에 공통 상한을 두지 않는다. 개별 수치와 직원당 3슬롯으로 밸런스를 조절한다.
- 이 합산 규칙을 모든 종류의 수치나 기존 피해 계산 전체에 일괄 적용하지 않는다.
- 기초 태그 초안을 스킬·유물에 적용하며 확장하고 대표 설계로 정식 목록을 다듬는다. 예시 단어나
  임시 분류를 정식 schema enum으로 확정하지 않는다.

### 연쇄 발동과 비호환

- 유물이 만든 효과가 다른 유물의 발동 조건을 만족하면 연쇄 발동할 수 있다.
- 한 연쇄 경로가 동일 유물 사본의 같은 효과로 되돌아오는 순환 재발동은 금지한다.
- 새 기본 공격 등 별도 원인에서 시작한 연쇄는 다시 발동할 수 있다. 전투 전체에서 유물당 한 번만
  허용하는 제한으로 해석하지 않는다.
- 함께 성립할 수 없는 효과는 비호환 태그로 표현하고 장착 순서와 무관하게 함께 장착할 수 없다.
  예시는 같은 스킬의 스택 전부 소모와 소모 금지 효과다.
- 비호환 검사는 같은 직원의 유물 조합에 적용한다. 다른 직원 간의 장착을 이 태그로 제한하지 않는다.
- 비호환과 동일 종류의 중복 장착은 별개의 제한이다. 연쇄 경로와 태그 활성화를 판정하는 내부
  알고리즘·순서는 후속 구현 검토 대상이다.

### 판매·분쇄

- 유물은 상점에서 판매할 수 없다.
- Maintenance에서 분쇄해 기존 장비 가루를 얻는다. 장착 중이면 확인 후 자동 해제할 수 있다.
- 유물 강화용 재료나 별도의 유물 강화 경제는 만들지 않는다. 유물별 분쇄 획득량은 미정이다.
- 분쇄 가루와 무기 강화 필수 재료의 연결은 [보상과 무기 강화 재료](#보상과-무기-강화-재료)의
  미해결 정책을 따른다. 일반 전투 전용 획득 목표를 임의로 완화하지 않는다.

### 체크포인트·전투 상태 수명

- 유물 보유 수량과 장착 상태를 체크포인트에 포함하며 불러오면 저장 시점대로 복원한다.
- 효과의 스택·발동 횟수 등 누적 상태는 해당 전투 안에서만 유지하고 새 전투 시작 시 초기화한다.
- 같은 전투에서 재배치하면 일시 스택은 초기화하지만 `전투당 N회` 사용 기록은 유지한다.
- 후퇴·같은 process 재접속과 연결할 실제 상태 구성·복원 경로, 침식된 유물 변형의 보존과 기존
  저장 데이터 전환은 후속 설계 대상이다.

### 침식된 유물

- 침식 아이템을 기존 유물에 적용해 침식된 유물을 얻는다. 침식된 유물 자체를 직접 드롭하지 않는다.
- 효과를 대폭 강화하는 대신 유닛을 죽음에 이르게 할 수 있는 패널티를 짝지어 설계한다.
- 패널티 후보는 집중 중 자신의 HP 희생, 침식체의 최우선 공격 목표, 받는 피해 100% 증가 등이다.
  모든 유물에 동시에 적용하거나 공통 확정 패널티로 취급하지 않는다.
- 기존 희귀도와 구별되는 변형 상태로 정리하는 방향이다. 새 희귀도·반복 레벨 강화나 원래
  희귀도 처리·재적용 규칙을 확정한 것은 아니다.
- 아이템 획득처·소모량, 적용 장소·시점·대상 사본, 해제·재적용·보존, 중복·비호환은 미정이다.
- HP 희생의 비용·피해 구분과 회복·보호막·생존 보장·HP 0 판정, 공격 우선순위의 사거리·저지·
  경로 규칙, 개별 패널티의 시작·종료는 후속 설계한다. 매초 최대 HP 6%는 설명용 후보다.

### 파편 침식형과의 경계

여기서 파편 침식형은 현재 전장의 적을 즉시 kill 판정하고 연출하는 행동이다. 침식된 유물의
변형 상태와 구분한다. 상세 발동·직원 대가·연출은
[침식 시스템 메모](../tmp/ego_corrosion_system_design_notes_2026-08-19.ko.md)가 소유하며 이 통합에서
그 문서 전체를 검토·동결한 것은 아니다.

- 파편 침식형에 유물 증폭·추가 효과·연쇄를 적용하지 않는다. 범용·전용 유물 모두 같다.
- 침식 처치로 유물·장비·파편의 처치 시 효과와 적의 사망 시 효과를 발생시키지 않는다.
- 죽은 적은 웨이브 정리·다음 웨이브·승리 조건 검사에서 정상 처치로 인정한다.
- 아직 남은 전투 진행을 무시한 무조건 즉시 승리는 아니다. 원본 환상체 등장 연출을 일반
  소환·스킬 경로로 해석하지 않는다.
- 침식에 유물 미적용이라는 규칙이 침식된 유물 자체를 비활성화하는 뜻은 아니다.
- 초기 AI안의 보스 최대 HP 20% 피해와 직원 신체 부상 추가는 미승인이다.

### 획득 정책

**보류:** 일반 유물의 획득처·빈도·수량·후보 구성·호환성 기준·후보 부족 처리·선택 개수·
건너뛰기·재추첨을 아직 정하지 않았다. 희귀도 도입이나 판매 금지를 획득처 승인으로 해석하지 않는다.

첫 파편과 호환 유물 3개 중 1개, Reward의 RelicCache, Unknown Shop 구매, 위험 Event,
NormalBoss 반복 보상, FinalBoss·숨겨진 보스 보상, 일반 Combat 무작위 드롭은 모두 검토 후보다.
일반 보스 재조우는 장비·재화 파밍 목적의 재설계 대상이며 유물 지급 채택은 계속 보류한다.
시작 1개·층별 2개·총 13개/17개·특정 층 희귀도 보장은 확정 공급량이 아니다.

### 전투력과 표시

- 활성 유물 효과를 Certified 전투력과 이후 Endless 난이도에 반영한다. 조건부 효과·태그 변형의
  평가·가중치는 콘텐츠와 실제 코드를 검토해 정한다.
- 무기·파편 설명에서 추가·강화된 효과, 적용 조건과 원인이 된 유물을 확인할 수 있게 한다.
- UI 배치·tooltip·계산 결과 전달 구조는 미정이다.

### 개별 콘텐츠의 합의 범위

공통 룰북은 개별 콘텐츠의 소유 문서를 연결한다. 아래는 입력 노트가 기록한 합의 범위이며,
정식 catalog·전체 태그 목록·개별 runtime 구현 완료를 뜻하지 않는다.

| 콘텐츠 | 기록된 합의 | 미정 또는 상세 소유 문서 |
| --- | --- | --- |
| 깊은 쐐기 | 타격별 누적 피해 증가 구조, 희귀, 대체 이름 `때린 곳 또 때리기`, 태그 명칭 `다중타격` | 타격당 10%p는 예시. [유물 콘텐츠 노트](../skills/run_relic_content_design_notes.ko.md#깊은-쐐기--가칭) |
| 붉은 실타래 | 래티씨아 기본·개화 파편의 작은·큰 친구에 적용하는 전용 유물 | 일반 태그만 같은 다른 파편에는 적용하지 않음. [유물 콘텐츠 노트](../skills/run_relic_content_design_notes.ko.md) |
| 소환물 범용 유물 | `소환물` 태그, 소환물 피해를 받은 대상에게 5초간 받는 피해 증가 | 이름·증가율·희귀도·재부여·중첩 미정 |
| 가하는 회복량 증가 | 범용 회복량 증가 25~40% 방향 | 최종 증가율·이름·희귀도 미정 |
| 이로운 스택당 공격력 증가 | 가장 많이 보유한 이로운 스택 한 종류의 수량만 적용. 5·3·2이면 5스택분 | 스택당 수치·이름·희귀도·개별 스택 범위 미정 |
| 눈의 여왕 전용 유물 | 착용자의 빙설의 파도·서리 마검 냉기로 둔화된 적 사망 시 빙결 잔영. 착용자 막타 불필요 | 침식 제외·순환 금지 적용. [상세 노트](../skills/run_relic_content_design_notes.ko.md#눈의-여왕-전용-유물--이름-미정) |
| 단일 대상 전환·효과 증폭 | 여러 아군 지원을 한 명에게 집중하는 범용 유물. 포큐버스 공용 보호막도 적용 | 포큐버스 보호량 2배 채택, 침식 4배는 패널티와 함께 확정할 비교값. 대상 선정·기본 식 미정 |
| 태아 적용 수치 | 기본/단일 전환/침식된 전환 순으로 기본 공격 피해 20%/40%/80%, 공속 가산량 15/20/30. 상태별 교체 | 모든 범위 효과 공통 증폭률 아님. 집중 6~8초 방향의 정확한 값·단일 선정·침식 패널티 미정 |
| 태아 전용 유물 | 2026-09-08에 전용 유물 자체를 미정으로 되돌림 | 이름·효과·수치 미정. 위 범용 전환 효과와 구분 |
| 무기 수치 유물 9종 | 첫 무기 유물 설계 범위를 단순 수치 강화로 정함 | 9종 이름·공격력/공속/평타 수치·중복·적용 세부는 AI 초안 |

`탐욕의 팔찌`는 유물과 별도의 액세서리 콘텐츠다. 이로운 효과 한 종류의 스택을 전부 소비하고
고정 시간 동안 소모량에 따른 단계별 공격력 버프를 얻는 액티브 구조를 기록했다. 상세는
[파편·장비 설계 노트](../skills/abnormality_skill_fragments_design_notes.md#액세서리--탐욕의-팔찌-가칭)가 소유한다.

이전 노트의 등급별 파편 성능 서열, 두건·흉탄 유물 아이디어와 과거 태그 후보는 참고 자료다.
원본 메모의 합의·미정 표시를 보존하며 개별 콘텐츠 승인으로 올리지 않는다.

## 비전투 노드

이 장은 SavePoint, Maintenance, HeadquartersContact에서 플레이어가 얻는 결과를 소유한다.
세션 identity, capability, atomic commit과 snapshot projection은 runtime/wire 경계의 요약이며 세부
구현은 [core_runtime_contract.md](../core_runtime_contract.md)와 외부 Unity canonical을 따른다.

### SavePoint 노드

> **기존 기준:** SavePoint·run당 load 최대 3회를 유지 기준으로 둔다. 자동 저장 전환과 과거 load
> 제거는 미승인 제안이다. 유물 보유·장착의 체크포인트 포함은 합의 변경이지만, 아래 기존 V4가
> 새 데이터를 지원한다는 뜻은 아니며 차기 저장 형식과 기존 데이터 전환은 미정이다.

- 완료 시 run당 단일 checkpoint를 덮어쓰고 Safezone으로 복귀한다.
- Load는 최대 3회이며 Config에서 core가 `LoadRunCheckpoint` capability를 enabled로 투영할 때만
  실행한다. Checkpoint metadata 자체는 실행 가능성을 뜻하지 않는다.
- 생존 직원 trauma를 RunPolicy `save_point_trauma_recovery`에 따라 줄인다. Live 값은
  `CurrentPercent { percent: 15 }`이며 현재 trauma의 15%를 올림한다. Critical과 Breakdown도
  생존 직원으로 포함하고 Dead는 제외한다. Physical damage는 바꾸지 않는다.
- Recovery, node completion과 checkpoint capture는 승인된 최종 상태를 기준으로
  원자적으로 한 번만 적용한다. Host의 durable active-run publish까지 실패하면 이 transition을
  성공으로 보고하지 않는다. SavePoint는 psychology/trust/memory/event를 생산하지 않는다.
- 마지막 완료 SavePoint는 versioned local-install active-run current file과 rolling backup의
  continue source다. 이어하기는 load 횟수를 소비하지 않으며 transport player identity와 영구
  조우 도감은 payload 밖에서 주입·보존한다.
- SavePoint는 active battle, battle clock, 배치, Wave/summon state와 attempt-local 도감 후보를 저장하지
  않는다. Process restart 뒤 이어하기는 마지막 완료 SavePoint 상태에서 시작하며 중단된 전투를 resync하지
  않는다.
- **기존 저장 구현 기준:** 현재 V4 payload의 seed, map/progression, roster, inventory/equipment ownership, skill fragment와 encounter
  progression graph는
  이어하기나 명시적 load가 runtime을 바꾸기 전에 엄격히 검증한다. 불일치 문서는 state, load 횟수와
  active-run export를 바꾸지 않고 거부하며 repair, rebinding 또는 구 schema migration을 수행하지 않는다.
  V2/V3와 unknown/missing/extra/research/pending shape는 strict reject하며, 명시적 load는 payload 내부 seed
  일치에 더해 현재 live run seed와도 일치해야 한다.
- `loads_used`는 checkpoint payload와 분리된 run-scoped monotonic metadata다. 성공한 명시적 load만
  정확히 1 증가시키고 SavePoint 덮어쓰기, checkpoint 복원, process restart와 backup 복구로
  감소시키지 않는다.

### Maintenance 노드

Maintenance는 허용된 분쇄·무기 강화·파편 개화와 형태 선택을 반복한 뒤 완료하는 독립 Safe Node다.

**합의 변경을 반영한 작업 범위:**

| 대상 | 작업 |
| --- | --- |
| 무기 | 개별 recipe에 따른 강화·분쇄 |
| 방어구·액세서리 | 개별 recipe에 따른 분쇄. 일반 강화 없음 |
| 스킬 파편 | 개화·기본/개화 형태 선택·분쇄. 일반 강화 없음 |
| 유물 | 장착 변경·직원 간 이전·분쇄. 일반 강화 없음 |

- 파편 가루는 개화용으로 유지한다. 유물은 분쇄하면 기존 장비 가루를 준다. 무기 강화의 필수 재료와
  장비 가루의 관계는 재료 경제에서 결정하며, 기존 recipe를 그대로 적용할 수 있다고 가정하지 않는다.
- 장착한 대상을 분쇄할 때는 확인 후 자동 해제할 수 있다. Maintenance에서 loadout 교체와 유물
  장착 변경을 할 수 있다.
- Core가 대상별 가능 여부, 불가 사유, 비용·획득량·변경 전후 preview와 확인·자동 해제를 계산한다.
  실제 실행에서는 최신 보유·장착·recipe·재료 상태를 다시 검증하고 실패 시 부분 변경을 남기지 않는다.
- 반복된 같은 material ID는 첫 등장 순서를 유지한 합계로 검증·소비한다. 부족·overflow를 먼저
  거부하고 정확한 합계를 한 번 소비해 한 단계 강화한다.
- 무기·장비의 recipe 없는 작업에 일반 비용·획득량을 임의로 적용하지 않는다. 기존 live의
  `standard_armor` 강화 recipe는 차기 방어구 강화 정책으로 이월하지 않는다.
- 완료 전에는 여러 작업을 반복할 수 있고 완료하면 node를 소비한다. Safezone은 장착 변경을
  제공하지만 이 정비 작업을 자동 제공하지 않는다.

기존 `UpgradeSkillFragment`·`EnhanceEquipment` 등의 command/capability는 현재 구현 계약의 이름이다.
파편 일반 강화 제거, 무기 전용 강화, 형태 선택·유물 분쇄를 반영할 차기 command·DTO·recipe는
후속 설계한다. 이 초안에서는 schema나 migration 방식을 확정하지 않는다.

### 본사 연락 노드

본사 연락 노드는 런 중 본부와 연결되는 별도 Safe Node다.

- `SavePoint`, `Maintenance`와 역할이 다르므로 별도 노드 계열로 다룬다.
- 본사 연락은 스킬 파편 연구도, pending 지급과 파편 수령 경로가 아니다.
- 본사 연락은 추후 정보 확인, 채용, 보급, 특수 지시 수령 같은 별도 선택지로 확장할 수 있다.

## 결과와 실패

이 장은 node 소비, 전투 결과 확정, 후퇴, 직원 상태와 run failure가 플레이어에게 갖는 의미를
소유한다. Runtime instance ledger, rollback snapshot, capability preflight와 commit 순서는
[core_runtime_contract.md](../core_runtime_contract.md)가 소유한다.

### 노드 결과 처리

CombatResult는 성공/실패, 직원 condition 변화, 통계, 자동·bonus 성장 보상, 파편 지급 trigger와
숙달 변화를 표시한 뒤 플레이어 확인으로 commit한다.

- Encounter의 named terminal outcome 하나가 stable outcome ID와 Victory/Defeat verdict를 commit한다. System
  time cap/implicit Draw는 없고 시간 종료가 필요하면 encounter가 `TimeReached` outcome을 작성한다. 같은
  boundary에서 Defeat와 Victory가 함께 성립하면 Defeat가 우선한다.
- `CompleteCombatResult`는 condition, consumable duration, reward/XP, 파편/격파 ledger, node
  진행과 session 정리를 하나의 원자적 transition으로 적용한다. 실패하면 user-visible state를
  부분 commit하지 않는다.
- Attempt 중 condition은 working state에 누적하고 결과 확인 때 한 번 roster에 반영한다. 여러
  runtime instance의 통계는 직원별 한 행으로 합치며 마지막 instance가 최종 HP/생존을 대표한다.
- 결과 완료 capability와 preflight는 Core가 소유하고 Unity는 context에서 추론하지 않는다.
- 패배도 살아남은 직원에게 생존 XP를 지급하지만 성공 보상, bonus objective 보상, 파편 지급과
  승리 격파 횟수는 지급·증가시키지 않는다.

**기존 시도 기준 / 필수 관문 재검토:** 전투 node는 첫 진입을 포함해 기본 최대 3회 도전한다.

- 승리/패배 result commit이 한 회를 소비하고 retreat는 진입 상태를 복원해 소비하지 않는다.
- **현행 비교:** Normal/Elite/NormalBoss는 기존 규칙에서 3회를 모두 소진하면 보상 없이
  Completed transit node가 된다. 필수 Elite·NormalBoss를 도입한 차기 규칙에서는 이 통과 처리를
  그대로 적용할지, 관문 재도전·실패를 어떻게 정의할지 미정이다. 일반 전투의 도전 횟수도
  재설계 검토 항목이며 초기 AI안의 2회 패배 후 런 실패를 자동 채택하지 않는다.
- 모노리스 강화 재현 node는 예외적으로 최대 2회이며 두 번째 패배에 보상 없이 Completed가 된다.
- FinalBoss 패배는 run failure 후보이며 결과 확정 전 retreat는 허용한다. 유효하고 횟수가 남은
  checkpoint가 있으면 다른 gameplay failure와 같은 사전 선택을 거치고, 없으면 즉시 확정된다.

### 후퇴

`RetreatBattle`은 개별 유닛 철수와 달리 BattleEnd/패배 판정 전 전투 전체를 포기하는 rollback이다.

- Pause, cast와 wave 진행 중에도 가능하지만 명시된 강제 encounter는 금지할 수 있다.
- Core는 NodeConfirm 진입 직전의 attempt, RNG/UUID continuation, roster condition, consumable,
  inventory/resource, node/session/VN/checkpoint 상태를 정확히 복원한다. 차기 유물 상태가 이 후퇴
  경계와 같은 process 재접속에 연결되는 구체적인 저장·복원 경로는 후속 검토한다.
- CombatResult, 보상·패널티, node 완료와 시도 소진을 만들지 않는다.

### 직원 condition, 전투 HP, 전투불능, 런 실패

Persistent condition의 authority는 `physical_damage: 0..6`와 `trauma: 0..100`뿐이다.

- Physical damage 0/1~3/4/5/6은 Normal/Minor/Major/Critical/Dead이고 trauma 100은 살아 있는
  Breakdown이다.
- Combat/Event/high-risk 참여는 physical damage 0~4와 trauma 0~99에서만 가능하다. Critical과
  Breakdown은 loadout/치료가 가능하지만 사망 직원은 모든 action에서 제외한다.
- Battle HP는 runtime 값이며 persistent condition으로 되쓰지 않는다. 전투불능 instance마다
  attempt-local physical damage 1과 authored trauma를 정확히 한 번 더한다.
- Physical damage는 명시적 Physical Treatment만 줄인다. 살아 있는 직원은 현재 physical damage가
  0이어도 구조적으로 유효한 target이며, 선택한 Treatment는 실효 회복량과 무관하게 소비된다.
  SavePoint/Gate는 trauma만 회복하고 사망을 되돌리지 않는다.
- 일반 사망 시 roster record는 남기되 장비와 shared active fragment, 장착 유물을 원자적으로 회수하고
  consumable modifier를 제거한다. Death-prevent가 발동하면 회수하지 않는다.

Breakdown은 안정 상태가 아니라 시한부다. Trauma 100은 그 자체로 사망이 아니지만, 방치하면 사망으로
확정된다.

- Trauma가 100에 도달한 시점부터 RunPolicy가 소유한 `breakdown_collapse_node_limit`만큼의 node
  완료를 센다. 그 안에 trauma를 99 이하로 낮추는 조치가 없으면 해당 직원은 사망한다.
- Counter는 node category와 무관하게 완료한 node마다 정확히 1 증가한다. 전투 attempt 소진, retreat와
  같은 미완료 결과는 증가시키지 않는다.
- Trauma가 99 이하로 내려가면 counter를 즉시 폐기한다. 이후 다시 100에 도달하면 새 counter를 처음부터
  시작하며 이전 진행도를 이월하지 않는다.
- Breakdown 사망은 일반 사망과 같은 authority를 사용한다. Roster record를 남기고 장비와 shared active
  fragment, 장착 유물을 원자적으로 회수하며 consumable modifier를 제거한다. 전투 밖 node transition이므로
  battle death-prevent의 대상이 아니다.
- 남은 node 수는 snapshot으로 투영한다. Unity는 node 진행이나 trauma 수치에서 재계산하지 않는다.
- Counter는 run/checkpoint 상태다. Checkpoint load는 저장 시점의 남은 node 수를 그대로 복원한다.
- `breakdown_collapse_node_limit`의 live 값은 아직 authoring하지 않았다. 현재 정책 의도값은 `2`다.

살아 있는 직원이 없으면 run failure 후보이다. 전투 결과 뒤 deployable 직원이 없으면 현재 보유한
Physical Treatment, 현재 progression에서 즉시 선택 가능한 SavePoint/Gate, 실제 후보를 제공하는
HeadquartersContact recruitment를 하나의 recovery-path evaluator로 계산한다. 제한 중에는 이 행동과
checkpoint load만 허용하고 snapshot의 `combat_recovery_restriction` reason/허용 node 집합을 Unity가
표시한다. 허용 행동 하나가 끝날 때마다 전체 deployability와 경로를 다시 계산하며 더 이상 실제
경로가 없으면 failure 후보가 된다.

모든 gameplay failure 후보(`NoLivingEmployees`, `NoDeployableEmployees`, `BossDefeated`)는 유효하고
횟수가 남은 checkpoint가 있으면 `AwaitingRunFailureDecision`에 먼저 들어간다. 플레이어는 checkpoint
load 또는 명시적 failure 확정을 선택하며, checkpoint가 없거나 loadable하지 않으면 즉시
`RunFailed`로 확정된다. `RunComplete`, 확정된 `RunFailed`, `AbandonRun`은 durable 이어하기를
무효화한다. 유효한 이어하기를 새 run으로 교체할 때도 한 번의 명시적 확인이 필요하다.

### 직원 Psychology

Authored personality는 변하지 않는 risk/social 두 축이고 Psychology는 별도의 run-local 신뢰와
기억 상태다.

- `trust_score`는 -100~100이며 band는 Distrust(-100~-51), Uneasy(-50~-11), Neutral(-10~25),
  Trusting(26~70), Devoted(71~100)로 파생한다.
- 현재 production Psychology mutation은 없다. SavePoint/Gate trauma recovery는 신뢰, 기억 또는
  최근 사건을 만들지 않는다.
- Durable memory는 같은 kind/subject를 병합하고 optional duration을 node 완료마다 줄인다. 최근
  반응은 별도 최대 8개 receipt다.
- Psychology는 roster와 checkpoint/retreat에 보존되며 사망해도 남는다.
- 신뢰 기반 명령 거부, combat modifier와 대사·표정 cue는 아직 gameplay 계약이 아니다.

## 후속 콘텐츠·구현 authoring

각 콘텐츠가 정확한 fragment ID·반복 보상, bonus objective 재료·XP 수량, BossOmen 선행 조건·
단계별 형태, Standard script를 작성한다. 합의된 신규 정책에는 무기 공격력·주문력과 파편 계수,
개화 재료, 유물 정의·태그·수치·획득 pool·분쇄량·침식 패널티의 authoring도 필요하다.

이 통합 초안의 gameplay 결정 전체가 완료된 것은 아니다. 특히 보상 수치 authoring 전에 일반
전투 전용 필수 재료의 획득 경로, 필수 관문의 결과와 Gate 처리, 유물·파편 공급 정책을 정리한다.
Schema·DTO·validation·저장·migration은 후속 구현 검토에서 결정한다.

## 고급 정책과 레거시

### FinalBoss pool과 BossOmen Chain

FinalBoss는 일반 Boss node를 직접 pool에서 배정하지 않는다. Endless의 authored BossOmen chain
pool에서 징조 체인을 고르고, 체인을 완료한 뒤 final-boss급 capstone encounter를 강제한다. 한 번에
active chain은 최대 하나다.

FinalBoss pool cycle:

- 개발 bootstrap의 authored 일반 chain membership은 one-step `white_night` 하나다. 이 chain은 generic
  eligibility/cycle/source-failure/capstone과 영구 all-FinalBoss record의 live end-to-end 검증을 위해
  활성화하며 최종 백야 chain 설계로 간주하지 않는다.

- 한 cycle은 모든 authored 일반 FinalBoss chain을 구성원으로 가진다. 각 chain RON은 minimum Floor,
  optional required fragment와 그 fragment의 개화 필요 여부를 직접 명시한다. 모든 FinalBoss에
  “아무 개화 파편 하나”를 요구하는 공통 조건은 두지 않는다.
- Chain identity, step/encounter reference, 선행 조건과 의존 관계는 live data 초기화에서 강하게
  검증한다. 알 수 없는 reference, 순환 의존성과 authored progression에서 달성 불가능한 조건은
  초기화를 거부하며 fallback chain, 조건 무시 또는 자동 해금으로 복구하지 않는다.
- Eligibility는 새 Floor를 생성할 때만 평가한다. 현재 Floor 도중 파편 획득 등으로 조건이 열려도
  이미 확정한 map에 징조를 추가하지 않고 다음 Floor부터 반영한다.
- 조건을 만족하지 못한 chain은 현재 cycle의 미완료 상태로 남되 후보에는 나오지 않는다. 완료된
  chain을 먼저 재사용하지 않으며, 남은 미완료 chain이 모두 잠겨 있으면 조건이 열릴 때까지
  BossOmen이 출현하지 않는다.
- 모든 authored 일반 FinalBoss chain을 한 번씩 완료해야 cycle을 다시 채운다. Cycle reset은
  Endless 완료가 아니며 다음 Floor들에서 같은 chain이 다시 등장할 수 있다.
- 흉탄의 사수 같은 별도 숨겨진 보스는 이 일반 FinalBoss pool cycle의 구성원이 아니다.

Provisional omen과 active chain:

- Eligible chain 하나를 provisional로 고르고 RON step의 Event 또는 Combat source를 현재 Floor의
  같은 category node 위에 overlay한다. 적합한 node가 없으면 다른 category로 fallback하지 않고
  다음 Floor로 미룬다.
- 첫 omen source를 방문하지 않고 Gate로 이동하면 chain을 확정하거나 현재 cycle에서 소진하지 않는다.
  같은 chain은 현재 제안 순서의 뒤로 보내 다른 eligible chain을 한 번씩 먼저 제안한 뒤 다시 후보가
  될 수 있다.
- 첫 omen의 Event source가 시작한 전투에서 패배하거나 Combat source의 모든 node attempt를 소진하면
  chain 활성화에 실패한다. 일반 Event 완료 규칙으로 omen step을 소비하지 않고, chain을 현재
  cycle에서 제거하지 않은 채 제안 순서의 뒤로 보낸다. 이후 Floor에서 다시 첫 omen 후보로 선택될
  수 있다.
- 첫 source를 성공적으로 해결하면 active chain으로 확정한다. 그 뒤 보스 승리 또는 Run 실패 전까지
  다른 FinalBoss chain을 시작하거나 현재 chain을 pool로 되돌릴 수 없다.
- Active chain 후속 step의 Event source가 시작한 전투에서 패배하거나 Combat source의 모든 node
  attempt를 소진해도 Run을 끝내거나 step을 진행하지 않는다. Active 상태와 같은 step을 유지하고
  다음 Floor에 다시 제안한다. Event source 전투는 해당 Event node 안에서 재시도하지 않는다.
- 모든 step을 완료하면 현재 탐사 map을 authored `boss_omen_capstone`으로 원자 교체한다. Capstone은
  Boss room과 미리 배정된 concealed Exit Gate room, `Boss -> Exit Gate` ForwardOnly connection을
  사용한다. Boss가 유일한 진행 node이고 이전 탐사 room으로 돌아갈 수 없으며 진입 전 Safezone
  loadout은 허용한다.

FinalBoss resolution:

- 실제 FinalBoss는 소탕할 수 없다. 패배 결과를 commit하면 남은 일반 node attempt 없이 즉시 Run
  실패 후보가 된다. 결과 확정 전 battle Retreat는 허용하며 유효한 checkpoint가 있으면 공통 실패 전
  load/확정 선택을 제공한다.
- 승리는 RON의 파편 또는 `repeat_reward`, authored 보상, chain 종료와 현재 pool cycle 완료를 함께
  commit한다. Capstone에 미리 배정된 Exit Gate와 connection을 공개하며 새 node/edge를 만들지 않는다.
- Boss 승리 자체는 Floor를 넘기거나 Endless를 완료하지 않는다. 공개된 Gate를 확인할 때 기존
  Gate transition과 trauma 회복을 적용하고 다음 Floor를 계속 생성한다.
- “미방문 FinalBoss를 pool로 반환”하는 규칙은 capstone Boss node가 아니라 provisional 첫 omen
  source에만 해당한다. Active chain 뒤의 capstone Boss는 건너뛸 수 없다.
- Active/completed chain, pool cycle, 파편과 보상은 checkpoint state다. 승리 뒤 과거 checkpoint를
  load하면 함께 rollback해 같은 chain을 다시 진행할 수 있다. 전역 도감, 영구 개인 기록과
  `loads_used`는 되돌리지 않는다.

숨겨진 보스:

- 변질된 모노리스 등 별도 RON 조건으로 발현하는 흉탄의 사수형 숨겨진 보스는 현재 timeline에서
  run당 한 번만 발현한다. 승리 completion ledger가 남아 있으면 다시 조건을 만족해도 재등장하지 않는다.
- 숨겨진 보스 승리도 `RunComplete`를 만들지 않는다. 보상·파편을 commit하고 Exit Gate를 공개해
  Endless의 다음 Floor로 계속 진행한다. 따라서 Exit Gate 없는 구 single-terminal hidden capstone과
  특별 완료 정책은 폐기 대상이다.
- 패배는 일반 FinalBoss와 같은 공통 Run 실패 후보·Retreat·checkpoint 규칙을 따른다.
- 승리 전 checkpoint 또는 승리 뒤의 과거 checkpoint를 load하면 발현·승리·보상·조건 소비가 함께
  rollback되므로 다시 발현해 도전할 수 있다. 이는 새 보상을 중복 보유하는 두 번째 발현이 아니라
  checkpoint가 복원한 timeline이며 `loads_used`는 감소하지 않는다.

### 레거시 제거 원칙

공통 compatibility, fallback, 구 테스트와 사용자 확인 원칙은
[engineering_workflow.md](../engineering_workflow.md#레거시와-문서-정리)가 소유한다. 이 룰북은
gameplay domain에서 다시 만들지 않을 계약만 열거한다.

- 구형 `timeline_delta`/`battle_delta` 전송을 다시 만들지 않는다.
- 제거된 전투 모드와 RON wrapper는 필요하다는 이유만으로 부활시키지 않는다.
- 제거된 `Support`, `Rest`, `ChooseSupport`, support choice mode와 관련 Psychology 사건은
  compatibility alias나 fallback으로 부활시키지 않는다.

## 미정 사항과 미승인 제안

이 목록은 새 선택을 대신 결정하지 않고, 합의된 항목을 일관된 게임으로 연결할 때 남은 논점을
모은 것이다. 합의된 공통 정책 전체를 다시 열 필요는 없으며 각 미정 경계와 수치만 이어서 검토한다.

### 다음 결정이 필요한 경계

| ID | 주제 | 확정된 출발점 | 남은 결정 |
| --- | --- | --- | --- |
| NEXT-01 | 모드·런 길이 | 기존 Standard·Endless를 비교 기준으로 보존 | 6층·8층, FinalBoss 수·배치, 본편 종료·연장, Standard 적용 범위 |
| NEXT-02 | 직원 규모 | 5~8명은 유력 검토 범위 | 보유·등록·동시 배치·예비의 구분, 시작 인원·채용, 보스 전장의 동시 역할 요구 |
| NEXT-03 | 필수 관문·지도 | Elite·NormalBoss 필수, 일반 전투 선택 | 층별 배치·Gate 조건, 일반 전투 수·방문 제한·재도전, 패배 소진 뒤 진행과 실패 |
| NEXT-04 | 강화 재료 경제 | 일반 전투에 참여해야 필수 재료 획득, 유물 분쇄는 장비 가루 지급 | 분쇄·상점·시작 보급·Event·보스 보상의 우회 경로 제거 방법, 비용·재료 종류·수량 |
| NEXT-05 | 파편·반복 조우 | 일반 강화 제거, 개화 유지. 기존 획득·반복 조우 삭제는 미승인 | 첫 획득 시점, 모노리스·숙달·소탕과 필수 관문의 연결, 파편·보스 보상 |
| NEXT-06 | 유물 공급 | 사본별 소유, 3슬롯, 일반·희귀·특급 | 획득처·빈도·수량·선택 방식·후보·재추첨·분쇄량과 희귀도 분포 |
| NEXT-07 | 능력치·개화 | 공격력/주문력 분리, 무기 제공 능력치 강화, 기본·개화 형태 선택 | 계수·증가량·개화 재료·형태 선택과 기존 조건의 연결·기존 강화 상태 처리 |
| NEXT-08 | 유물 효과 평가 | 태그 기반 조합, 비호환, 동일 사본·효과 순환 금지 | 정식 태그·효과 정의, 활성화·연쇄 내부 판정, Certified 기여 평가와 표시 |
| NEXT-09 | 침식된 유물 | 아이템 적용으로 변형, 강화 효과와 생존 위험 | 획득·적용·해제·재적용·사본·중복·보존, 개별 패널티와 HP·보호·타겟 규칙 |
| NEXT-10 | 저장·상태 수명 | 소유·장착 저장, 재배치 일시 상태 초기화·전투당 사용량 유지 | 기존 세이브 전환, 후퇴·재접속 경로, 새 형태·유물 데이터와 versioned 저장 계약 |
| NEXT-11 | 개별 콘텐츠 | 위 개별 합의와 해당 콘텐츠 문서의 소유권 | 미정 이름·효과·수치·등급·태그·개화 recipe, 침식 상세 문서와의 추가 통합 |
| NEXT-12 | 저지 좌표·획득·해제 | 최초 접근은 접근 쪽 앞 타일, 아군 타일 통과 중 저지는 그 타일 유지. 연출 보정과 타일 판정 분리 | 정확한 시작 범위·타일 경계·거리 해제·복수 신규 후보·동시 판정 순서, 기존 같은 logical tile 계약과 runtime·Unity 반영 |

### 승인된 규칙으로 사용하지 않을 초기안

다음은 전면 재설계 v1의 제안 또는 비교 수치다. 후속 사용자 결정이 있을 때 해당 본문을 갱신한다.

| 초기안 | 현재 취급 |
| --- | --- |
| 매 층 필수 일반 진입 전투 | 일반 침식체 선택 전투 합의로 교체 |
| 선택 전투 없이 시작 재료·핵심 보상으로 강화 가능 | 일반 전투 전용 필수 재료 목표에 맞춰 재설계 |
| 6층 본편·선택 연장·튜토리얼 축소 | 미승인. 8층과 비교 중 |
| 보유 5명·등록 4명·시작 3명·특정 층 채용 | 미승인. 직원 5~8명 검토 범위와 전장 역할을 기준으로 재검토 |
| 선택 탐사 최대 2방·준비 확정 후 폐쇄 | 미승인 |
| 시작 파편·유물 보급, Elite 첫 승리 파편 지급 | 미승인 |
| 층별 유물 2개·기본 13개/17개, 특정 층 희귀·특급 보장 | 미승인. 새 전투 구조에 맞춰 다시 계산 |
| 직원 XP·level·tier 제거 | 미승인. 기존 직원 성장 보존 |
| 모노리스·숙달·소탕 제거 | 미승인. 기존 규칙을 관문 정책과 조율할 대상 |
| 일반 보스 파편의 성능 서열 변경·공통 보스 핵 개화 | 미승인 |
| 무기 +3 한도·층별 강화 개방·투자 가루 75% 회수 | 미승인 수치·경제안 |
| 전투력 추종 난이도 제거 | 미승인. 활성 유물의 전투력·Endless 난이도 반영 합의 유지 |
| 층별 정비 거점·무료 치료·상점·채용 통합 | 미승인 |
| SavePoint·과거 load 제거와 자동 저장 | 미승인. 현재 저장 정책과 새 유물 저장 목표를 구분 |
| 필수 전투 패배 2회 뒤 런 실패 | 미승인 |
| BossOmen을 선택형 최종 보스 교체로 전환 | 미승인 |
| 침식 보스 피해 20%·직원 신체 부상 추가 | 미승인. 기존 처치·효과 미발동 경계 유지 |

### 이후 문서 갱신

2026-09-10 사용자 요청에 따라, 앞으로 새로 합의하는 내용은
[후속 정책 합의 임시 기록](../tmp/game_rulebook_followup_policy_notes_2026-09-10.ko.md)에 날짜·주제별로
남긴다. 이 초안 본문은 합의할 때마다 자동 갱신하지 않고, 이후 사용자가 통합을 요청할 때 해당
장과 미정 항목을 함께 갱신한다. 직전 저지 타일 합의는 이미 반영된 기준으로 유지한다.

추가 메모를 통합할 때는 실제 사용자 결정과 제안·예시를 구분하고 출처와 반영 여부를 남긴다.
원본 메모는 결정 경위를 확인할 자료로 보존한다.

기존 canonical의 교체와 소비자 참조 전환은 별도 이행 범위다. 그때 기존 룰북의 이후 변경,
인접 runtime/data/Unity 계약, 구현 상태와 저장 데이터 전환을 확인한다. 이 초안의 작성 자체는
현재 코드 수정·세이브 삭제·자동 migration·기존 문서 참조 변경을 승인하거나 수행하지 않는다.
