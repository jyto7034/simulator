# 스킬 화면 검증 기록

2026-10-19. WSL, Node 24.21.0, npm 11.19.0, Playwright Chromium에서 확인했다.
기능 범위와 사용법은 [스킬 조립기 첫 화면](skill-assembler-prototype.md)에 있다.

## 결과

- `npm run check`: 타입·린트·포맷 통과.
- `npm run build`: 프런트엔드 빌드 통과.
- `npm test`: 모델 테스트 28개 통과.
- `npm run ui:inspect`: 기존 맵과 새 스킬 화면의 126개 검사 통과.
- 예제 교체 시 스크롤 초기화와 진단 대상 블록 표시를 보완한 뒤,
  `npm run ui:inspect -- tests/ui/skill.spec.ts`: 스킬 검사 18개 재통과.
- 두 실행 모두 실패·건너뜀·불안정 재시도 0개. 보고서와 첨부 진단을 확인했다.

960×640, 1280×800, 1920×1080 각각 밝은/어두운 테마를 검사했다.
죽나장·두건 예제, 블록 중첩·순서·복제·실행 취소, 맵 문서/이력 보존,
JSON 다운로드·재열기, 잘못된 파일의 현재 문서 보존, 진단 포커스,
긴 한글 이름·빈 문서·60개 블록·삭제 확인의 Escape 취소를 포함한다.

검사 중 발견한 선택 항목의 작은 글씨 대비를 수정했다. 예제를 바꾸면 새 동작의
처음으로 이동하고, 진단을 누르면 속성 입력에 포커스를 주면서 해당 블록도 보이게 했다.
미정 필드와 끊어진 참조는 저장 가능한 상태로 유지한다.

## 화면과 보고서

아래 크기·테마별 PNG 및 빈 문서·긴 이름·60개 블록 화면을 직접 열어
한글, 줄바꿈, 패널 경계, 스크롤, 선택 표시와 포커스를 검토했다.
기존 맵의 최소 크기 화면, 테마 메뉴, 원형 도구 메뉴도 확인했다.

| 크기·테마        | 이미지                                                               |
| ---------------- | -------------------------------------------------------------------- |
| 960×640 밝음     | [두건](../artifacts/ui-skills-20261019/hood-minimum-light.png)       |
| 960×640 어두움   | [죽나장](../artifacts/ui-skills-20261019/butterfly-minimum-dark.png) |
| 1280×800 밝음    | [두건](../artifacts/ui-skills-20261019/hood-default-light.png)       |
| 1280×800 어두움  | [두건](../artifacts/ui-skills-20261019/hood-default-dark.png)        |
| 1920×1080 밝음   | [두건](../artifacts/ui-skills-20261019/hood-large-light.png)         |
| 1920×1080 어두움 | [죽나장](../artifacts/ui-skills-20261019/butterfly-large-dark.png)   |

[전체 UI 보고서](../artifacts/ui-skills-20261019/full/report.json)와
[마지막 스킬 UI 보고서](../artifacts/ui-skills-20261019/final/report.json)를 보존했다.
산출물은 Git 제외 대상인 로컬 `artifacts/`에 있다. 검토된 시각 비교 기준 이미지가
없어 `ui:compare`는 실행하지 않았으며 기준 이미지를 만들거나 바꾸지 않았다.

자동 접근성 위반은 0개다. 마지막 스킬 검사에서는 스크롤 영역 가장자리의 구성 제목·
상태 개수·상태 이름 등에 대비 `incomplete`가 3회 남았다. 해당 글씨와 배경을 화면에서
직접 확인했으며 자동 통과로 간주하지 않았다. 전체 검사에서는 대비 판단 78회와
테마 메뉴 `aria-valid-attr-value` 6회가 남았다. 메뉴의 실제 ID 참조·키보드·포커스
동작은 기존 UI 단언을 통과했다. 접근성 전체 검증을 완료했다는 의미는 아니다.

## 환경과 남은 범위

기존 Windows용 `node_modules`와 분리한 `/tmp/skill-assembler-20261019`에서
Linux 의존성을 준비했다. 현재 잠금 파일은 `npm ci --offline`에서 선택 의존성
`@emnapi/core`, `@emnapi/runtime` 누락으로 거부되어, 별도 폴더에서
`npm install --offline --package-lock=false`로 검증 환경을 만들었다.
프로젝트의 의존성 선언·잠금 파일과 기존 Windows 설치는 바꾸지 않았다.
따라서 깨끗한 환경에서 `npm ci`가 재현된 것으로 보고하지 않는다.

Windows WebView2·Tauri IPC·설치 프로그램·실제 한글 IME는 미검증이다.
Core 실행, RON 내보내기, 수식의 실행 의미, 드래그 연결과 네이티브 파일 저장은
이번 화면의 구현 범위에 포함하지 않는다.
