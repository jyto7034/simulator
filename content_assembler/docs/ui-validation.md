# AI의 프런트엔드 검증

Playwright로 실제 브라우저 화면과 동작을 검사하고, axe로 자동 접근성 검사를 수행한다.
현재 검사는 작업 공간·테마 메뉴와 전투 맵 편집, 영역/경로 드래그 취소, 조건 폼,
진단 이동, 파일 다운로드/재열기, 실패 보존, 고정 fixture를 대상으로 한다. 이 도구는 Tauri 앱을 실행하지 않으며 네이티브 API를 검증하지 않는다.

## 준비

```sh
npm ci
npm run ui:install
```

`ui:install`은 해당 OS의 Chromium headless shell을 내려받는다. 브라우저나 앱을 실행하지 않는다.
Linux에서 공유 라이브러리가 부족하면 `npx --no-install playwright install-deps chromium`으로
설치한다. 이 단계는 관리자 권한을 요구할 수 있다. Tauri Linux 빌드용 GTK/WebKit 설치와는
별개다. 다른 OS나 별도 작업 폴더로 옮기면 그 환경에서 의존성과 브라우저를 준비한다.

한글 촬영에는 한글 폰트가 필요하다. Ubuntu에서는 `fonts-noto-cjk` 패키지를 설치할 수 있다.
`fc-list :lang=ko family`로 등록 여부를 확인한다. 이 WSL 환경에는 PC에 이미 있던
Noto Sans KR을 사용자 폰트 디렉터리에 연결했다. 이 폰트는 저장소나 배포물에 포함하지 않는다.
다른 머신/CI에도 동일한 폰트 환경을 구성한 뒤 기준 이미지를 만든다.

## 명령

| 명령                  | 역할                                     | 서버/브라우저 실행 |
| --------------------- | ---------------------------------------- | ------------------ |
| `npm run check`       | 앱과 검사 코드의 타입·린트·포맷 검사     | 없음               |
| `npm run ui:list`     | 시나리오와 설정 로딩 확인                | 없음               |
| `npm run ui:inspect`  | PNG 촬영, 동작·레이아웃·접근성·오류 검사 | 있음               |
| `npm run ui:compare`  | 검토한 기준 이미지와 비교                | 있음               |
| `npm run ui:baseline` | 현재 화면으로 기준 이미지 생성/갱신      | 있음               |
| `npm run ui:report`   | HTML 보고서를 로컬 서버/브라우저로 열기  | 있음               |

촬영/비교 명령은 `127.0.0.1:1422`에 전용 Vite 개발 서버를 시작하고 검사 종료 시 정리한다.
점유 중인 서버를 재사용하지 않는다. 검사 중에는 소스를 변경하지 않는다.
`build`/`licenses`도 함께 실행하지 않는다. 고지 파일 갱신이 Vite의 페이지 새로고침을
유발하므로 빌드를 끝낸 뒤 UI 검사를 실행한다.
각 테스트는 독립된 브라우저 컨텍스트로 시작하므로 저장된 테마가 서로 영향을 주지 않는다.
명령 실행 시 출력되는 안내가 아니라 JSON 보고서의 테스트 결과로 성공/실패를 확인한다.
`ui:list`는 실제 테스트를 실행하지 않으므로 화면 정상 동작의 증거가 아니다.

빠른 반복 작업에서는 프로젝트/테스트를 선택할 수 있다. 완료 전에는 변경에 관련된
나머지 크기·테마도 확인한다.

```sh
npm run ui:inspect -- --project=chromium-default-light
npm run ui:inspect -- --grep "workspace and open theme menu"
```

## 촬영과 자동 검사 범위

- 최소 960×640, 기본 1280×800, 큰 화면 1920×1080 × 밝은/어두운 테마.
- 한국어 로케일, 서울 시간대, 배율 1, 동작 감소 설정으로 환경을 고정한다.
- 기본 작업 공간과 열린 테마 메뉴를 PNG로 촬영한다. 전체 페이지를 늘려 찍지 않고
  실제 뷰포트를 촬영하여 화면 경계의 잘림이 보이도록 한다.
- 문서 전체의 가로/세로 넘침과 주요 영역·메뉴가 화면 안에 있는지 검사한다.
  패널 내부의 의도된 스크롤은 허용한다.
- 콘솔 오류, 처리되지 않은 JS 예외, 실패한 요청, HTTP 400 이상 응답을 수집하고 실패로 처리한다.
  콘솔 경고도 진단에 남긴다. 이후 의도한 실패 응답을 테스트할 때는 해당 시나리오에만
  좁게 예외를 정의하고 실제 응답과 UI 오류 상태를 검증한다.
- axe로 WCAG 2/2.1 A·AA 규칙을 검사한다. 위반은 실패로 처리하고, 도구가 판단하지 못한
  `incomplete` 결과도 첨부한다. 자동 검사는 접근성 전체를 보장하지 않는다.
- 키보드 Tab/Enter/Escape로 메뉴 진입·닫기·포커스 복귀를 확인한다.
  키보드로 테마를 바꾼 뒤 새로고침해 선택이 유지되는지 검사한다.

## 결과물 읽기

| 경로                                     | 내용                                            |
| ---------------------------------------- | ----------------------------------------------- |
| `artifacts/ui/results/**/workspace.png`  | 기본 화면                                       |
| `artifacts/ui/results/**/theme-menu.png` | 열린 메뉴                                       |
| `artifacts/ui/report.json`               | 테스트 상태, 오류, 첨부 파일 정보               |
| `artifacts/ui/report/index.html`         | 사람이 살펴볼 HTML 보고서                       |
| `artifacts/ui/results/**/trace.zip`      | 실패한 테스트의 DOM·동작·네트워크 추적          |
| `tests/ui/baselines/`                    | 검토 후 생성한 플랫폼별 기준 PNG, Git 관리 대상 |

진단 JSON은 보고서의 attachments에 포함된다. 실제 PNG 경로는 보고서 또는
`rg --files artifacts/ui -g '*.png'`로 찾는다. AI는 이미지 보기 도구로 PNG를 열고
겹침·잘림·정렬·한글·줄바꿈·메뉴 위치를 직접 확인해야 한다.
문서 넘침 검사로 모든 내부 요소의 겹침/잘림을 발견할 수는 없다.
실패 trace는 `npx --no-install playwright show-trace <trace.zip 경로>`로 확인한다.
이 명령은 뷰어를 실행한다.

`artifacts/`는 Git에서 제외되며 다음 검사에서 덮어쓰일 수 있다.
비교/보고에 남길 결과는 다음 실행 전에 별도 위치로 보존한다.
초기 준비에서는 실제 촬영을 실행하지 않았으므로 검토된 기준 이미지가 없다.

## 기준 이미지 사용

1. `ui:inspect`를 실행하고 생성된 화면이 의도에 맞는지 직접 검토한다.
2. `ui:baseline`으로 기준 후보를 생성하고 해당 PNG를 검토한 뒤 Git에 추가한다.
3. 이후 `ui:compare`로 회귀를 확인한다. 기준이 없으면 실패하며 자동 승인하지 않는다.
4. 실패하면 actual/expected/diff를 확인한다. 의도한 디자인 변경일 때만 기준을 갱신한다.

이미지 비교는 디자인의 품질을 판단하지 않고 기존 화면과의 차이를 찾는다.
OS와 폰트가 다르면 픽셀이 달라지므로 기준 경로는 플랫폼과 크기·테마별로 분리한다.
브라우저/폰트 업그레이드 시에도 기준 이미지를 검토한다.

## 추가하면 유용한 도구

- Storybook: 속성 패널·대화상자 등 재사용 컴포넌트가 늘어날 때 상태별로 독립 확인한다.
- Vitest + Testing Library: 폼 검증, 편집 명령, 실행 취소 등 화면 사진으로 확인하기 어려운
  동작이 생길 때 도입한다.
- 고정 fixture와 네트워크/IPC 모의 계층: 긴 문자열, 빈 값, 대량 데이터, 로딩/실패를
  재현한다. 실제 연동 검사와 구분한다.
- Windows CI: Windows용 빌드 검증과 설치 파일 생성을 개발 초기에 자동화한다.
  실제 WebView2·Tauri 연동·파일 대화상자·한글 IME·Windows 배율은 네이티브 앱에서 확인한다.

순수 편집 모델 테스트는 기존 Playwright 러너로 실행하며 브라우저/서버는 사용하지 않는다.
위의 추가 패키지는 도입하지 않았다. 현재는 Playwright와 axe를 사용한다.
Playwright는 Apache-2.0, axe 관련 패키지는 MPL-2.0이며 제품의 런타임 번들에는 포함하지 않는다.

공식 문서: [Playwright 스크린샷](https://playwright.dev/docs/screenshots),
[이미지 비교와 환경 차이](https://playwright.dev/docs/test-snapshots),
[axe 접근성 검사](https://playwright.dev/docs/accessibility-testing),
[테스트 서버 관리](https://playwright.dev/docs/test-webserver).
