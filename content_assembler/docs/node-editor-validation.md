# TypeScript 노드 편집기 검증

## 연결선 강조 추가 (2026-10-20)

선택한 노드에 직접 연결된 입력·출력 선을 같은 강조색과 3px 두께로 표시한다.
다중 선택과 선택 해제를 반영하며 연출의 점선을 유지한다. 강조 여부는 화면 선택에서
계산하고 연결선 자체의 선택, 문서 데이터, 미저장 판정과 이력에는 섞지 않는다.

실제 선 클릭 검사에서 프레임 배경이 연결선을 가로막는 문제를 발견했다.
프레임 → 연결선 → 노드 카드 순서로 표시하여 선을 직접 선택할 수 있게 수정했다.
추가 테스트는 지원하지 않는 연결선 Enter 선택 대신 실제 포인터 클릭을 사용하며,
확대 이동 후 화면 상태가 반영된 것을 확인하고 클릭 좌표를 계산한다.

`npm run check`·`npm run build`와 최종 UI 36개가 통과했다.
960×640·1280×800·1920×1080의 밝은/어두운 테마에서 단일·다중 선택, 해제,
연결선 직접 선택, 점선 보존과 기존 연결·프레임·Undo/Redo를 검사했다.
각 크기·테마의 PNG를 직접 열어 선·노드·프레임의 표시 순서와 강조를 확인했다.
브라우저 오류·경고와 axe 자동 위반은 0건이다. axe 66회 중 수동 확인 대상으로
ARIA 속성 60회, 색 대비 56회, 숨겨진 포커스 6회가 남았다.
Windows WebView2·Tauri·한글 IME와 스크린 리더 전체 검증은 별도이며,
검토된 기준 이미지가 없어 `ui:compare`는 실행하지 않았다.

- [연결선 강조 화면](../artifacts/node-edge-highlight-20261020/default-light-node-connected-edges.png)
- [다중 선택 화면](../artifacts/node-edge-highlight-20261020/default-dark-multiple-node-connected-edges.png)
- [최종 UI 보고서](../artifacts/node-edge-highlight-20261020/final/report.json)
- [검사 집계·114개 소스 일치 확인](../artifacts/node-edge-highlight-20261020/verification.json)

## TypeScript 이관

2026-10-20. 문서·편집·검증·패치 이력·저장 확인을 React와 독립된 TypeScript 코어로 이관했다.
화면 동작과 `node-graph` version 1 파일 형식을 유지하고, 노드 전용 Rust crate·Tauri IPC·
Vite 개발 브리지·전송 응답 검사·통신 복구 UI를 제거했다. Rust는 Tauri 데스크톱 호스트에 남는다.
초기 Rust 구현의 검사 결과는 [이전 기록](node-editor-rust-validation.md)에 보존한다.

## 검사 결과

| 검사                  | 결과           | 확인 범위                                                   |
| --------------------- | -------------- | ----------------------------------------------------------- |
| `npm run check`       | 통과           | TypeScript, ESLint, Prettier                                |
| `npm run build`       | 통과           | 프런트엔드 번들과 의존성 고지 생성                          |
| `npm test`            | 48건 통과      | 노드 코어·저장소 17건, 기존 맵·스킬 모델 31건               |
| 관련 UI 행렬          | 90건 모두 통과 | 노드 30건과 기존 스킬·작업 공간 회귀 60건, 한 번의 실행     |
| 500노드 성능 시나리오 | 통과           | 30단계 드래그, 문서 확정 1회                                |
| 정적 빌드 미리보기    | 통과           | 추가·Undo/Redo·저장·재열기, 노드 전용 통신 요청 0건         |
| Tauri 호스트 포맷     | 통과           | `cargo fmt --manifest-path src-tauri/Cargo.toml -- --check` |

UI 행렬은 960×640, 1280×800, 1920×1080의 밝은/어두운 테마다.
Windows의 `node_modules`와 분리한 WSL 작업 폴더에서 검사했고, 검사한 소스와 원본의
바이트 일치를 확인했다. [소스 해시](../artifacts/node-typescript-20261020/source-sha256.txt),
[결과 집계](../artifacts/node-typescript-20261020/summary.json),
[UI 보고서](../artifacts/node-typescript-20261020/ui/report.json)를 보존했다.

## 계약과 실패 경계

기존 Rust 실행 파일에서 실제 내보낸 문서와 카탈로그를 고정 fixture로 남겼다.
`tests/model/fixtures/node-rust-v1.json`은 프레임이 포함된 기존 문서이며, 읽기·수정·재출력 시
ID·포트·좌표·속성·프레임이 보존된다. `node-catalog-v1.json`은 모든 노드의 기본값과 포트
정의가 이관 전과 일치하는지 검사한다.

모델 검사에는 다중 이동 중 뒤 항목 실패, 잘못된 속성과 비유한 좌표, 호환되지 않는 재연결,
연결을 포함한 노드 추가 실패, 복제·붙여넣기의 내부 참조 재매핑, 프레임 삭제와 복원,
경유 노드, 작성 중 순환 진단, 변경 없는 편집, Redo 분기, 이력 제한을 포함한다.
저장 완료 순서 역전·중복 확인·문서 교체 후 오래된 결과·잘못된 파일이 기존 문서와 이력을
보존하는지 확인했다. 확정 문서는 불변이며 구독자 오류는 확정 결과를 되돌리지 않는다.

UI 검사에서는 드래그·취소·다중 이동·포트 연결과 재연결·프레임·클립보드·속성 적용과 취소·
잘못된 입력 보존·키보드 메뉴·Escape와 포커스 복귀·저장/재열기를 확인했다.
로딩은 지연 로딩 모듈의 실제 경계를 제어했고, 빈 문서·긴 한글 이름·잘못된 파일·미완성
진단도 촬영했다. 문서 관찰은 테스트에서만 로컬 저장소의 공개 구독/스냅샷 API를 사용하며
제품에 테스트 전용 상태나 전역 변수를 넣지 않았다.

정적 빌드 검사는 모듈 계측 없이 `vite preview`에서 수행했다. 실제 다운로드와 업로드,
추가·Undo/Redo·파일 재열기가 통과했고 브라우저 오류·경고와 `/__node-editor` 요청은 0건이었다.
[검사 스크립트와 결과](../artifacts/node-typescript-20261020/preview/result.json)를 보존했다.

## 직접 검토한 화면

각 크기·테마의 대표 PNG를 직접 열어 한글, 긴 이름, 배치·정렬, 패널 스크롤, 메뉴 위치와
포커스 표시를 확인했다. 최소 화면과 많은 노드의 전체 보기에서는 노드가 축소되며 확대해서
편집한다. 비교 기준 이미지가 없어 `ui:compare`와 새 기준 이미지 승인은 수행하지 않았다.

- [최소 밝은 화면](../artifacts/node-typescript-20261020/nodes-edited-minimum-light.png)
- [최소 어두운 프레임 메뉴](../artifacts/node-typescript-20261020/nodes-frame-menu-minimum-dark.png)
- [기본 밝은 노드 추가 대화상자](../artifacts/node-typescript-20261020/nodes-add-dialog-default-light.png)
- [기본 어두운 화면](../artifacts/node-typescript-20261020/nodes-edited-default-dark.png)
- [큰 밝은 화면](../artifacts/node-typescript-20261020/nodes-edited-large-light.png)
- [큰 어두운 미완성 진단](../artifacts/node-typescript-20261020/nodes-incomplete-large-dark.png)
- [잘못된 파일 보존](../artifacts/node-typescript-20261020/nodes-invalid-file-default-light.png)
- [정적 빌드의 파일 재열기](../artifacts/node-typescript-20261020/preview/nodes-static-preview.png)

## 접근성 진단

90건 전체에서 브라우저 오류·경고, 보고서 오류, axe `violations`는 0건이다.
아래 `incomplete`는 자동 판정이 불가능했던 항목이며 반복 캡처를 포함한 발생 수다.
[대상 목록](../artifacts/node-typescript-20261020/incomplete-targets.json)에 상세 HTML을 보존했다.

| 범위      | 규칙                    | 발생 횟수 | 대상 노드 발생 횟수 | 수동 확인과 제한                                                                                    |
| --------- | ----------------------- | --------: | ------------------: | --------------------------------------------------------------------------------------------------- |
| 노드      | `aria-prohibited-attr`  |        42 |                  42 | React Flow 확대 도구 컨테이너의 `aria-label`. 개별 버튼 이름·조작 확인, 컨테이너 ARIA 판정은 미완료 |
| 노드      | `color-contrast`        |        40 |                 344 | 축소된 그래프·프레임·겹침 영역. 대표 이미지에서 식별 확인, 모든 대상의 대비 수치 측정은 아님        |
| 노드      | `aria-hidden-focus`     |         6 |                  18 | Radix의 숨김 루트·포커스 가드. 대화상자 키보드 조작·Escape·복귀 확인                                |
| 기존 회귀 | `color-contrast`        |        42 |               1,314 | 기존 그래프 대표 화면 검토, 전 대상 대비 수치 측정은 아님                                           |
| 기존 회귀 | `aria-valid-attr-value` |         6 |                   6 | 테마 메뉴의 `aria-controls`. 메뉴 대상과 키보드 포커스 검사 통과                                    |

자동 위반 0건을 접근성 전체 검증 완료로 해석하지 않는다. Windows 보조 기술은 미검증이다.

## 성능

WSL Chromium 1280×800 밝은 테마의 Vite 개발 모드에서 500노드·499연결을 측정했다.
30단계 드래그 중 문서 확정은 놓을 때 1회였다. 176개 프레임 간격 표본의 중앙값은
16.7ms, p95는 16.8ms였고, 단일 로컬 확정 처리는 약 8.8ms였다.
[측정 원자료](../artifacts/node-typescript-20261020/performance-final/measurement.json)를 보존했다.

첫 측정은 테스트 도구가 React StrictMode에서 폐기된 초기 저장소를 관찰해 실패했다.
실제 화면에는 500노드가 열려 있었다. React가 구독하는 인스턴스를 관찰하도록 테스트 도구를
고친 뒤 재검사에서 통과했다. 편집 동작이나 기대 노드 수를 바꾸지 않았다.

프레임 간격과 로컬 확정 시간은 서로 다른 지표다. 이전 Rust 프로세스 왕복 시간과 직접
성능 우열을 비교하지 않는다. 한 환경의 표본이며 Windows WebView2나 2,000노드 UI의
성능을 보장하지 않는다.

## 남은 범위

Windows 빌드·WebView2·한글 IME·실제 데스크톱 다운로드·설치 프로그램은 미검증이다.
Tauri의 노드 의존성을 제거하고 Cargo 잠금 파일을 로컬 캐시에서 다시 생성했지만,
네이티브 호스트 전체 컴파일은 이번 검증 결과에 포함하지 않는다.
기존 환경에는 GTK/WebKit 개발 라이브러리가 준비되어 있지 않다.

계산·분기·동작의 실행 의미, 우리 스킬 모델 적용, 게임 Core 연결은 후속 작업이다.
브라우저 저장은 다운로드 후 사용자가 파일을 확인하는 기존 방식이며,
네이티브 파일 대화상자와 원자적 디스크 쓰기를 구현한 것은 아니다.
