# Docs Index

docs에는 현재 게임 명세, 클라이언트 계약, 유지해야 할 작업 원칙만 남긴다. 완료된 goal 문서, 과거 감사 로그, 구현 실험 기록, 단기 핸드오프 문서는 현재 source-of-truth 문서로 필요한 내용을 옮긴 뒤 삭제한다.

## Canonical Unity Docs

Unity 구현/계약 문서의 canonical 위치는 이 저장소의 `docs/`가 아니라 아래 외부 Unity 프로젝트 문서 디렉토리다.

```text
F:\unity projects\ark\docs
/mnt/f/unity projects/ark/docs
```

특히 아래 두 파일은 외부 위치를 기준으로 관리한다.

- `F:\unity projects\ark\docs\unity_core_contract.md`
- `F:\unity projects\ark\docs\unity_client_implementation_goal.md`

이 저장소 안에 같은 이름의 문서가 보이면 stale copy일 수 있다. Unity-facing 계약이나 구현 목표를 갱신해야 할 때는 반드시 외부 canonical 문서를 확인하고 갱신한다.

## 먼저 읽을 문서

- `game_rulebook.md`: 게임 규칙과 노드/전투/보상 정책.
- `F:\unity projects\ark\docs\unity_core_contract.md`: Unity 클라이언트와 `/game` WebSocket 계약.
- `refactor_preparation_plan.md`: 리팩토링 판단 기준과 레거시 제거 원칙.

## 도메인 문서

- `gameplay_flow_example.md`: 플레이 기록 형태의 게임 흐름 예시.
- `skill_target_contract.md`: 스킬 타겟/범위 데이터 계약.
- `skills/lobotomy_content_catalog.md`: 환상체/스킬 파편/장비 콘텐츠 카탈로그.

## 작업 원칙

- `codex_goal_command.md`: 새 goal을 Codex에게 맡길 때 붙여 넣는 표준 goal 명령어.
- 레거시는 과감하게 제거한다. 현재 공식 흐름에 없는 compatibility layer, adapter, legacy test는 되살리지 않는다.
- 문서를 무조건 따르지 않는다. 실제 코드와 live RON/API를 읽고 더 나은 개선안이 보이면 근거를 설명하고 사용자 확인 후 진행한다.
- 정책이 모호하면 구현 전에 사용자와 의논한다. 특히 게임 룰, 노드 흐름, 보상, 직원 성장, 전투 성공/실패 판정은 임의 확정하지 않는다.
- 새 추상화는 실제 변형과 호출부가 충분히 확인된 뒤 도입한다. 단순 함수나 테이블로 충분하면 그쪽을 우선한다.
- 테스트는 내부 추상화 모양이 아니라 실제 플레이 흐름과 Unity-facing 계약을 고정해야 한다.

## 정리 원칙

- 새 goal 문서는 작업 중에만 둔다.
- goal이 완료되면 현재 정책/계약만 source-of-truth 문서로 옮기고 goal 문서는 삭제한다.
- "나중에 할 수도 있는 기능"은 현재 구현 지시처럼 보이지 않게 TODO 또는 보류 정책으로만 남긴다.
- 문서와 코드가 충돌하면 코드를 읽고, 정책 판단이 필요한 부분은 사용자와 의논한다.
