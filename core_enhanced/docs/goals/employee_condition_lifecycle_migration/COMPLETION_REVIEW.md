# Employee Condition Lifecycle Migration Completion Review

Status: complete

Reviewed: 2026-07-18

## 완료 판단

Phase 3은 complete다. 직원의 장기 상태와 모든 purpose eligibility는 `physical_damage`와 `trauma`에서 파생하며, Run HP, generic injury, mutable life/availability authority는 production code와 public consumer에서 제거됐다.

## 구현 증거

- physical damage 0~6, trauma 0~100과 Normal/Minor/Major/Critical/Dead/Breakdown 파생 상태
- combat/Event/high-risk, consumable, loadout, physical treatment와 trauma recovery의 중앙 typed eligibility
- RuntimeUnit instance별 전투불능 one-shot, same-attempt redeploy, result one-shot commit과 exact retreat rollback
- final max HP 뒤 trauma floor/minimum-one 공식과 BattleHpSetup 합성
- death-prevent 단일 소비, 일반 사망 roster record 유지와 equipment/shared fragment 원자 회수
- 정수 physical Treatment와 Rest/SavePoint/Gate trauma-only 분리
- checkpoint와 `admin_set_employee_condition`, legacy RON/admin rejection
- Core DTO, server transport, Unity strict reader/presenter complete cutover

## 검증 증거

- Core: 775 passed, 0 failed (`731` unit + `44` integration).
- game_server: 30 passed, 0 failed.
- Unity EditMode: job `332889c81c374b8cae2d0a7c950fc039`, 240 passed.
- Unity ordinary PlayMode: job `a398e95e9a8f4213986860005d933fab`, 57 passed.
- 전체 PlayMode의 live-only 4개는 서버 미기동 connection failure였으며 ordinary suite와 분리했다.
- Rust employee profile fixture와 action capability fixture exact match가 통과했다.
- `cargo fmt --all` 및 `git diff --check`가 통과했다.

## Legacy 감사

production Core/server/live RON/Unity 검색에서 `EmployeeLifeState`, `EmployeeAvailability`, `EmployeeHealthState`, `EmployeeInjury`, `RunHpLossMitigation`, `available_for_combat`, 구형 employee admin command가 남지 않았다. 검색 결과의 구형 문자열은 strict rejection/absence를 증명하는 negative test뿐이다.

## 알려진 비차단 항목

- repository 전체 `cargo clippy --all-targets -- -D warnings`는 Phase 밖 기존 lint debt 119개 때문에 실패한다.
- production Treatment amount는 승인되지 않았으므로 새 live item을 만들지 않았다.
- 외부 durable save/database는 발견되거나 검증되지 않았다.

## 다음 Phase 게이트

Phase 1 definition/Profile, Phase 2 recruitment ownership과 Phase 3 condition lifecycle이 모두 complete이므로 Phase 4 `employee_psychology_state_contract`에 진입할 수 있다.
