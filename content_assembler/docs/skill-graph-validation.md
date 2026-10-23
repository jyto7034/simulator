# 스킬 노드 그래프 검증

2026-10-19. React Flow 12.12.0과 ELK.js 0.12.0을 사용해 스킬 화면을
시작 사건별 노드 그래프로 바꿨다. [사용법과 구현 경계](skill-assembler-prototype.md).

## 검사 범위

- `npm run check`: TypeScript·ESLint·Prettier.
- `npm run build`: 지연 로딩한 스킬 화면과 ELK Web Worker를 포함한 프런트엔드 빌드.
- `npm test`: 31개 모델 검사. 접힌/펼친 분기와 합류, 동시 실행의 독립 경로,
  숨겨진 자식 진단 집계, 실제 ELK 배치의 유한 좌표·겹침 방지를 포함한다.
- `npm run ui:inspect -- tests/ui/skill.spec.ts tests/ui/workspace.spec.ts`:
  960×640, 1280×800, 1920×1080 × 밝은/어두운 테마의 36개 검사.
  두 예제, 확대·미니맵 전환, 중첩 추가·순서·복제·실행 취소, 맵 문서/이력 보존,
  저장·재열기·잘못된 파일 보존, 접힌 노드 진단 이동, 빈 문서·긴 이름·60개 블록,
  자동 배치 Worker 실패·재시도와 문서/이력 보존을 확인한다.

위 검사는 모두 통과했다. 마지막 UI 실행은 36개 통과, 실패·건너뜀·불안정 재시도
0개이며 브라우저 오류·경고도 0개다. [원본 보고서](../artifacts/ui-graph-20261019/final/report.json)와
[집계](../artifacts/ui-graph-20261019/summary.json)를 확인했다.

고정 fixture의 브라우저 오류 주입은 테스트에서만 수행한다.
제품 코드는 작업 실패를 표시하고 사용자가 다시 배치할 수 있게 한다.

## 화면 검토

자동 검사 후 PNG를 직접 열어 한글, 제목·요약 생략, 선택/포커스, 패널 경계,
분기 선과 합류, 블록 선택 창의 스크롤을 확인했다. 작은 창에서 글자를 가리던
확대 버튼은 도구줄로 옮기고 미니맵은 기본으로 숨기며 필요할 때 켜도록 했다.
전체 흐름에서 긴 내용은 요약하며, 선택 후 오른쪽 속성에서 원문을 읽고 편집한다.

| 화면             | 이미지                                                                     |
| ---------------- | -------------------------------------------------------------------------- |
| 960×640 밝음     | [두건 전체](../artifacts/ui-graph-20261019/hood-minimum-light.png)         |
| 960×640 어두움   | [죽나장 전체](../artifacts/ui-graph-20261019/butterfly-minimum-dark.png)   |
| 1280×800 밝음    | [두건 전체](../artifacts/ui-graph-20261019/hood-default-light.png)         |
| 1280×800 어두움  | [분기 펼침](../artifacts/ui-graph-20261019/hood-expanded-default-dark.png) |
| 1920×1080 밝음   | [분기 펼침](../artifacts/ui-graph-20261019/hood-expanded-large-light.png)  |
| 1920×1080 어두움 | [분기 펼침](../artifacts/ui-graph-20261019/hood-expanded-large-dark.png)   |

검사 산출물은 `artifacts/ui-graph-20261019/`에 보존한다. 이 폴더는 Git 제외 대상이다.
검토된 시각 비교 기준 이미지가 없어 `ui:compare`와 기준 이미지 갱신은 수행하지 않았다.

자동 접근성 위반은 0개다. 확대 버튼을 감싸는 일반 요소의 불필요한 ARIA 이름을
제거하고 개별 버튼의 한국어 접근성 이름을 유지했다. 남은 `incomplete`는 대비 판단
24회와 기존 테마 메뉴의 `aria-valid-attr-value` 6회다. 그래프의 변환된 글씨와 맵의
SVG·도구 글리프, 보조 글씨는 실제 화면에서 읽기와 배경을 확인했으며 자동 통과로
간주하지 않았다. 메뉴 ID 참조와 키보드·포커스 동작은 기존 검사를 통과했다.
접근성 전체 검증 완료를 뜻하지 않는다.

## 환경과 한계

WSL Node 24.21.0, npm 11.19.0, Playwright Chromium을 사용했다.
Windows 설치와 분리한 `/tmp/skill-assembler-20261019`에서 검사했다.
이번 의존성 갱신으로 잠금 파일의 선택 의존성 누락을 복구했으며
`npm ci --offline`의 깨끗한 재설치가 통과했다. 초기 화면 검증 기록의 설치 제한은
그 당시 상태이며 이번 갱신 후에는 해당하지 않는다.

Windows WebView2·Tauri IPC·파일 대화상자·설치 프로그램·실제 한글 IME는 미검증이다.
기존 맵 전체 시나리오를 이번에 재실행한 것은 아니며, 모델 전체와 공통 작업 공간 및
스킬/맵 전환을 검사했다. 그래프의 선과 합류는 편집 문서의 표시이며 Core 실행 의미를
검증하지 않는다. 임의의 선 연결, 노드 위치 저장, RON 내보내기와 네이티브 저장은 미구현이다.

## 노드 우클릭 메뉴 추가 검증

2026-10-19 후속 변경으로 노드에 Radix 우클릭 메뉴를 연결했다. 블록 추가·복제·순서
변경·삭제, 내부 절차 펼침/접힘과 진단 이동을 지원한다. 분기에는 아니면 절차 추가,
시작 사건에는 마지막 블록 추가와 동작 삭제를 제공한다. 표시용 합류 노드는 원본 블록으로
이동한다. 메뉴와 속성 패널은 같은 편집 연산과 실행 취소 이력을 사용한다.

`npm run check`, `npm run build`, 모델 검사 31개가 통과했다. 다음 UI 검사 54개도
세 창 크기와 두 테마에서 모두 통과했다. 실패·건너뜀·불안정 재시도와 브라우저 오류·경고는
각각 0개다.

```sh
npm run ui:inspect -- tests/ui/skill-context-menu.spec.ts tests/ui/skill.spec.ts tests/ui/hood-review.spec.ts tests/ui/workspace.spec.ts
```

우클릭한 대상 선택, 메뉴의 화면 경계, Shift+F10 열기, Escape 포커스 복귀,
추가 창 취소, 중첩 복제의 새 ID, 이동과 실행 취소, 삭제 취소·확정·복원 및 저장 문서의
동일성을 확인했다. 기존 스킬 편집과 두건 검토 예제의 저장 왕복, 공통 작업 공간도 재검사했다.

PNG를 직접 열어 한글, 메뉴 위치·잘림, 테마, 비활성 항목을 확인했다.

- [960×640 밝음](../artifacts/ui-node-menu-20261019/node-context-menu-minimum-light.png),
  [어두움](../artifacts/ui-node-menu-20261019/branch-context-menu-minimum-dark.png)
- [1280×800 밝음](../artifacts/ui-node-menu-20261019/branch-context-menu-default-light.png),
  [어두움](../artifacts/ui-node-menu-20261019/node-context-menu-default-dark.png)
- [1920×1080 밝음](../artifacts/ui-node-menu-20261019/node-context-menu-large-light.png),
  [어두움](../artifacts/ui-node-menu-20261019/node-context-menu-large-dark.png)

[원본 보고서](../artifacts/ui-node-menu-20261019/final/report.json)와
[집계](../artifacts/ui-node-menu-20261019/summary.json)를 보존했다. 자동 접근성 위반은 0개이며,
`incomplete`는 대비 판단 36회, 기존 테마 메뉴의 ARIA 값 6회다. 변환된 노드 글씨,
중첩 배경, 짧은 글씨와 SVG를 포함한 수동 확인 항목으로 유지한다. 새 우클릭 메뉴 항목
자체에는 미완료 대비 판정이 없었다. Windows 네이티브 동작과 접근성 전체 검증은 미완료이며,
검토된 기준 이미지가 없어 자동 시각 비교와 기준 이미지 갱신은 수행하지 않았다.

## 노드 자유 배치 추가

2026-10-19 후속 변경으로 노드의 제목·요약을 드래그해 화면 위치를 옮길 수 있게 했다.
연결선은 이동한 노드를 따라간다. ‘전체 보기’는 현재 배치를 화면에 맞추고,
‘자동 정렬’은 수동 위치를 초기화하여 자동 배치로 되돌린다.

위치는 편집 문서와 분리한 화면 상태다. 속성 편집·실행 취소·접기/펼치기·작업 공간 전환에서
유지하며 실행 순서·JSON·미저장 표시·문서 이력에는 반영하지 않는다. 같은 ID의 파일을
다시 여는 경우를 포함하여 문서 교체 시에는 초기화한다.

구현은 [React Flow의 노드 변경 이벤트](https://reactflow.dev/api-reference/react-flow#node-events)에서
위치 변경만 받아 처리한다. 드래그 시작 시 해당 노드를 선택하며, 추가·접기·진단 버튼은
드래그 대상에서 제외한다.

`npm run check`와 `npm run build`가 통과했다. `skill-drag.spec.ts`를 위의 4개 UI 검사
파일에 더해 실행한 60개 검사가 모두 통과했다. 실패·건너뜀·불안정 재시도 및 브라우저
오류·경고는 각각 0개다. 다른 블록을 선택한 상태에서 한 노드만 이동하는 동작,
편집·Undo·접기/펼치기·맵 전환 후 위치 유지, 자동 정렬, 동일 파일 재열기 시 위치 초기화,
드래그 전후 저장 문서의 동일성을 검사했다. 문서 모델은 변경하지 않아 모델 검사는 재실행하지 않았다.

[원본 보고서](../artifacts/ui-node-drag-20261019/final/report.json)와
[집계](../artifacts/ui-node-drag-20261019/summary.json), 드래그·정렬 PNG를 보존했다.
다음 화면을 직접 열어 도구줄·한글·연결선·선택 표시를 확인했다.

- [960×640 밝음: 이동](../artifacts/ui-node-drag-20261019/skill-node-dragged-minimum-light.png),
  [어두움: 정렬](../artifacts/ui-node-drag-20261019/skill-node-realigned-minimum-dark.png)
- [1280×800 밝음: 이동](../artifacts/ui-node-drag-20261019/skill-node-dragged-default-light.png),
  [어두움: 정렬](../artifacts/ui-node-drag-20261019/skill-node-realigned-default-dark.png)
- [1920×1080 밝음: 이동](../artifacts/ui-node-drag-20261019/skill-node-dragged-large-light.png),
  [어두움: 정렬](../artifacts/ui-node-drag-20261019/skill-node-realigned-large-dark.png)

자유 배치에서는 노드를 겹칠 수 있으며 자동 정렬로 겹침을 해소한다. 접근성 자동 위반은
0개이며 미완료 판정은 대비 42회, 기존 테마 메뉴 ARIA 6회다. 겹친 노드와 변환된 글씨의
배경 판단, 짧은 글씨·SVG를 수동 확인 항목으로 유지한다. 접근성 전체 검증이나
Windows WebView2·Tauri 검증을 완료한 것은 아니다. 검토된 기준 이미지가 없어 자동
시각 비교와 기준 이미지 갱신은 수행하지 않았다.
