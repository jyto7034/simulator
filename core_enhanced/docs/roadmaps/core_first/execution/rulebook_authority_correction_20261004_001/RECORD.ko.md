# 과거 룰북 혼입과 최신 정책 기준 교정

2026-10-04 · CF-D08 / USER-17 · 문서 교정

## 사용자 결정과 발견

사용자가 `game_rulebook.md`는 과거 문서이고 `game_rulebook_next.md`가 최신 문서임을 확인한 뒤,
오늘 변경 사실을 로드맵에 기록하고 교정하도록 요청했다. 수정 전 안내는 과거 룰북을 최상위
Gameplay canonical로, next를 현행 기준을 대체하지 않는 차기 설계로 소개해 이 결정과 충돌했다.

조사 때 과거 룰북의 최종 수정 시각은 2026-10-04 20:56:30 +09:00, next는
2026-10-03 18:58:01 +09:00이었다. 시각만 바뀐 것이 아니라 아래 본문 변경이 있었다.
Git diff에는 8월 이후 미커밋 변경이 함께 포함되므로 오늘 변경분 판정에 사용하지 않았다.

## 오늘 확인된 변경

| 구간 | 보관본 비교로 확인한 내용 |
| --- | --- |
| U04 변경 전 → U04 완료 target (본문 mtime 17:38:31 +09:00) | ChooseOne 허용·후보 고정, 모노리스 첫 승패 소비, SavePoint와 최신 Continue 분리·V10/저장 실패 설명, 승패 즉시 정산, 패배 재도전 제거, 후퇴 2회, 이어하기 절 추가, Breakdown 완료 카운터 문구, terminal 필수 선택 중 Continue 유지 |
| U05 baseline → 교정 전 작업본 (mtime 20:56:30 +09:00) | Hard CC의 미실행 단계 취소, Platform Rest 공명 보너스 제거와 장비 설명, 접수 시 공명 소비·필수 대상 상실 취소·RetargetOnStep 거절, 직원 공명과 사용 수명 절 추가 |

U04 완료 문서와 U05 baseline의 과거 룰북 bytes가 일치한다. 이 비교 범위에서 맵 생성·방 배치·연결
구조 본문은 바뀌지 않았으나 노드 소비·후퇴·복구 정책은 변경됐다. 이는 보관본 사이의 확인 결과이며
오늘 자정 이후의 모든 편집 이력을 복원했다는 뜻은 아니다.

## 적용한 교정

1. 과거 룰북 본문을 U04 변경 전 target의 SOURCE_BEFORE 보관본으로 복원했다. 그보다 오래된
   미커밋 변경은 보존했다. 서두·문서 등급·권위 안내만 과거 비교 자료임을 명시하도록 정정했다.
2. next 서두와 docs/README의 역전된 권위 안내를 정정했다. 최신 상세 정책 본문은 변경하지 않았다.
3. 활성 로드맵 §0.1.1과 P12, CF-D08, 단일 현황에 사실·근거·후속 인수 기준을 연결했다.

정책 기준은 최신 명시적 사용자 합의 → next와 연결 상세 규칙이다. 과거 문서는 최신 정책이
명시적으로 계승한 공통 규칙만 참조하며 새 정책·구현 결과의 동기화 대상이 아니다. 정확한 schema와
API는 해당 최신 runtime/data/host 계약에, 구현 진행·검증은 실행 문서에 기록한다.

이번 교정은 U04/U05 제품 구현을 되돌리거나 새 정책을 취소하지 않는다. 기존 동결 계획·독립 감사·
보관본을 소급 편집하지 않는다. 기존 감사는 당시 target의 판정으로 보존하고 이후 실행·감사는 이
교정을 인수해 현재 문서 hash와 적용성을 새로 결속한다. 문서 변경만으로 새 제품 PASS를 주장하지 않는다.

## 근거와 확인

- U04 복원 기준: `core_enhanced/docs/goals/core_result_persistence/execution/single_goal/target/u04_001/SOURCE_BEFORE.tar.gz`
  안의 `core_enhanced/docs/game_rulebook.md`.
- U04 이후/U05 이전: 같은 target의 `SOURCE_AFTER.tar.gz`와
  `core_enhanced/docs/goals/core_resonance_lifecycle/execution/single_goal/baseline/SOURCE_BEFORE.tar.gz`.
- [교정 직전 문서 보관본](BEFORE.tar.gz): 이번에 수정한 여섯 문서의 실제 직전 bytes.
- [내용 식별·검증 기록](EVIDENCE.json): archive/member hash, 직전 mtime, 수정 전후 hash와 확인 항목.

검증 완료: 과거 룰북의 `게임 개요` 이하 본문은 U04 변경 전 보관본과 정확히 일치하고, next의
`어떤 게임인가` 이하 정책 본문은 교정 전과 같다. 보관본·수정 전후 hash, 추가 상대 링크와 추가 행의
공백 검사를 통과했다. 대상 문서의 `git diff --check`도 통과했다. 제품 소스 변경이 없어 제품 테스트는 실행하지 않았다.
