# 룰북 아카이빙과 경로 변경 대응표

2026-10-04 · CF-D09 / USER-18

사용자는 과거 룰북 아카이빙, 최신 룰북·상세 규칙의 이름 정리, 현재 정책과 과거 비교 참조의 분리,
동결 계획·감사 원본 보존을 승인했다. Gameplay 의미와 제품 코드는 변경하지 않는다.

## 현재 위치

| 이름 변경 전 | 이름 변경 후 | 해석 |
| --- | --- | --- |
| `core_enhanced/docs/game_rulebook.md` | `core_enhanced/docs/archive/game_rulebook_legacy_2026-10-04.md` | CF-D08에서 혼입을 복원한 과거 룰북. 당시 정책 비교용 |
| `core_enhanced/docs/game_rulebook_next.md` | `core_enhanced/docs/game_rulebook.md` | 최신 게임 정책 입구 |
| `core_enhanced/docs/game_rules_next/` | `core_enhanced/docs/game_rules/` | 최신 상세 규칙 10개. 정책 본문을 재설계하지 않고 경로·비교 링크를 정리 |

[최신 룰북](../../../../game_rulebook.md)과 [과거 룰북](../../../../archive/game_rulebook_legacy_2026-10-04.md)은
서로 다른 정책 시점의 문서다. 이름이 같은 파일로 옛 기록의 의미를 바꾸지 않는다.

## 참조와 증거 인수

- 현재 문서·설계·소개·로드맵의 최신 정책 링크는 새 룰북과 상세 규칙으로 연결했다.
- 옛 모드·모노리스·숙달·전조 비교와 과거 결정/메모의 기존 룰북 참조는 아카이브로 연결했다.
  일반 과거 결정·메모는 참조 경로만 갱신하며 당시 결정의 내용과 승인 상태는 유지한다.
- `skills/skill_fragment_system.md`와 과거 프로젝트 조사는 역사적 설계/조사임을 명시했다.
- `goals/`, 기존 `archive/`, `reviews/`, 로드맵의 동결 execution/records/evidence/reviews/handoffs와
  과거 의존성 검토는 원문을 보존했다. 계속 사용하는 로드맵 결정 장부에는 CF-D09만 추가했다.
- 위 동결 기록의 예전 경로는 당시 archive와 hash가 우선한다. 대응표의 최신 위치는 탐색을 위한 것이며
  그 파일이 과거 동결 bytes와 같다는 주장이 아니다. 이번 이동 직전 bytes는 아래 BEFORE에 별도 보관했다.
  옛 next 경로에 중복 문서나 자동 대체 링크를 남기지 않는다.
- 현재 실행자는 경로 변경을 인수해 실행 문서/감사 입력의 locator와 현재 source identity를 새로 결속한다.
  기존 Goal/Freeze/감사 원문을 소급 수정하거나 이전 검증의 현재성을 자동 승계하지 않는다.

`architecture/runtime_feature_manifest.v1.json`은 과거 룰북 heading 기반의 보존 검증 색인이다.
이 색인의 룰북 locator도 아카이브로 옮겼고, 문서 분류 확인 문자열을 Historical로 정정했다.
기존 hash·feature mapping·실행 증거는 그대로 보존했다. 최신 정책 inventory로 승격하거나 hash를
현재 값으로 덮어써 검증된 것처럼 만들지 않았다. 기존의 오래된 hash/내용 불일치는 이 이동으로 해소됐다고 주장하지 않는다.

## 보관과 검증

- [이동·수정 직전 원문](BEFORE.tar.gz)
- [경로·수정 전후 hash·보호 대상·검증 결과](EVIDENCE.json)
- [선행 CF-D08 혼입 교정](../rulebook_authority_correction_20261004_001/RECORD.ko.md)

검증 완료: 정책 본문과 상세 규칙 10개는 경로·과거 비교 표기 외에 동일하다. 참조가 있는 보호 대상
247개 원문 hash가 유지됐고, 확인한 링크 365개는 대상·앵커가 유효했다. 현재 문서의 이전 경로 잔여와
이번 이동으로 새로 끊어진 링크는 0개다. 공백 검사와 `git diff --check`를 통과했다.

아카이브에는 이동 전부터 삭제된 `engineering_workflow.md`를 가리키던 링크 3개가 남는다.
역사적 원문에서 이미 없던 대상을 임의의 새 계약으로 대체하지 않았으며 EVIDENCE에 기존 문제로 분류했다.
과거 feature manifest의 기록 hash도 이동 전부터 당시 작업본과 불일치했으며, 이를 새 PASS로 재작성하지 않았다.
제품 코드 변경이 없어 제품 전체 시험은 실행하지 않았다.
