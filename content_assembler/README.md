# 콘텐츠 조립기

게임 콘텐츠 제작을 위한 독립적인 데스크톱 프런트엔드입니다.
Tauri 2 + React + TypeScript + Vite를 사용하며, Tailwind CSS 4와
Radix 기반 shadcn/ui로 UI를 구성합니다.

프로젝트가 해결하려는 문제, 전체 대상과 첫 작업 범위, 게임 코어와의 관계는
[프로젝트 목적과 범위](docs/project-purpose.md)에 정리되어 있습니다.
일반 보스·엘리트 파편의 작업 순서, 완료 조건과 현재 위치는
[스킬 조립기 임시 파이프라인](docs/skill-authoring-pipeline.tmp.md)에서 관리합니다.

스킬은 **노드 그래프 작성 → Core 검증·컴파일 → 실행용 IR → RON 저장 → 게임 로드·실행**으로 연결합니다.
편집 원본과 실행 산출물은 별도입니다. 일반 보스 10계열·엘리트 11계열의 포함 형태를 공통
연산으로 연결했으며, 대상과 계열별 실행 근거는 [작업 파이프라인](docs/skill-authoring-pipeline.tmp.md)에 기록합니다.
미정 수치·범위·미지원 구성은 원본 위치를 가리키는 진단으로 컴파일을 거절합니다.
검증용 수치를 넣은 사본의 실행 성공이 원본 밸런스 확정을 뜻하지 않습니다.
[공통 계약](docs/common-skill-graph-contract.md)과 각 파편 원문의 후속 정책이 실행 의미의 기준입니다.

**컴파일**은 현재 편집본을 검증하고 실행 RON을 만듭니다. 결과는 요청한 문서·수정 번호와
결속하며 오래된 응답으로 새 문서를 덮지 않습니다. RON 재로드 검증과 실제 전투 검사는
서로 구분합니다. 브라우저 개발 서버와 Tauri 브리지가 같은 Rust 컴파일 경계를 사용합니다.
공명은 정수 전용이며 비율 비용은 항상 올림합니다. 소수 입력과 이전 `rounding` 필드는 거절합니다.
시간 입력은 0.001초 단위까지 허용하며 더 세밀한 값은 자동 반올림 없이 거절합니다.
시간에 따른 위력은 정수 매개값으로 작성하고 최종 위력을 한 번 올림하는 실행 계약을 따릅니다.

P4의 절망의 기사 기본/개화·붉은 두건·탐욕의 왕도 실제 그래프 목록에서 열어 수정할 수 있습니다.
관계·단계 소비·입력 계획·동시 방출·착탄 대기·전투 원장 작성은 [콘텐츠 안내](content/README.md#절망의-기사붉은-두건탐욕의-왕--p4)를 따릅니다.
미정 값은 진단으로 남기며, 확정한 검증 사본의 Core 실행은 P7/P8 기록에서 확인합니다.

상단 **맵 / 스킬 / 노드**에서 작업 공간을 전환합니다. 스킬 첫 화면에서는 죽나장·두건 설계 예제로
상태·사건별 동작·중첩 블록을 편집하고, 진단·Undo/Redo·JSON 저장/열기를 사용할 수 있습니다.
[스킬 조립기 안내](docs/skill-assembler-prototype.md)를 참고하세요. 수식과 조건은 설계 문자열이며
아직 Core 실행 문법으로 검증·변환하지 않습니다.

스킬 작성 명세는 [일반보스 콘텐츠 요구사항](docs/normal-boss-skill-requirements.md)과
[Hello / Goodbye 상세 사례](docs/hello-goodbye-spec.md)에서 시작합니다.
여러 스킬이 공유할 입력·비용·값·출처·수명·완료와 노드 작성 기준은
[공통 스킬 그래프 계약](docs/common-skill-graph-contract.md)에 정리했습니다.
합의한 콘텐츠를 기준으로 게임 판정·연출·조정값을 구분한 설계 문서입니다.
외부 설계 참고로 [Forge Statescript 적용 기준](docs/forge-statescript-reference.md)을 채택했으며,
수명·상태·취소 처리 원리와 단계별 검증 항목을 정리했습니다.
채택 원리와 우리 게임의 차이, 실제 수정·검증 사례를 같은 기준 문서에 기록합니다.

**노드** 작업 공간은 UI와 독립된 TypeScript 코어가 문서·편집 명령·이력·검증을 관리합니다.
포트 연결·재연결, 다중 이동, 프레임, 속성, 복사/붙여넣기와 JSON 저장을 지원합니다.
이동한 위치도 저장과 Undo에 포함합니다. [노드 편집기 안내](docs/node-editor-guide.md)와
[기존 프로젝트 분석](docs/node-editor-reference-analysis.md)을 참고하세요.
[Hello / Goodbye 스킬 파편](docs/hello-goodbye-node-guide.md)을 노드 예제로 열어 수정할 수 있습니다.
공명 조건·비용·접수·후속 지불 노드와 9가지 소비 예제도 그래프 목록에서 제공합니다.
작성 방법과 실행 지원의 경계는 [공명 그래프 계약](docs/common-skill-graph-contract.md#공명-노드와-연결-타입의-확장)을 따릅니다.
장판을 포함한 스킬 효과 범위는 [타일 패턴](docs/common-skill-graph-contract.md#스킬-범위와-장판의-타일-패턴-계약)으로 작성합니다.
Goodbye 범위는 격자에서 직접 선택하며 이전 도형 파일은 재지정 진단과 함께 보존합니다.
새 그래프에서는 **스킬 범위 → 효과 범위 배치**를 사용합니다. **스킬 대상 선택**의 선택 범위는 별도 연결입니다.
그래프 목록의 ‘range-front-strike’와 ‘range-selected-ground’는 수치 미정의 구조 예제이며,
지정 타일 장판은 패턴을 회전하지 않습니다. 장판 생성에 피해 효과가 자동으로 붙지는 않습니다.
`ground-damage`, `ground-heal`, `ground-status` 예제에서 장판 사건·대상별 효과·상태 수명을 연결합니다.
첫 주기 발동, 상태 중첩과 이탈 후 수명은 직접 선택하며 미지정 값은 진단합니다.
생성 당시 대상도 진입으로 처리하고 종료 시각의 주기는 발생하지 않는 작성 계약입니다.
자세한 의미는 [장판 사건·효과 계약](docs/common-skill-graph-contract.md#장판-사건대상-효과상태-수명)을 따릅니다.
게임 동작과 연출을 구분하며 상태·시간·위력·참조를 검사합니다. 지원하는 실행 구성과 검증 한계는 파이프라인을 따릅니다.

노드는 빈 그래프로 시작하며 **새 그래프**는 새 빈 문서를 만듭니다.
**그래프 목록 / 저장**은 프로젝트의 [content 폴더](content/README.md)에 연결됩니다.
그래프 목록에는 실제 저장 경로와 파일들이 표시되며 Hello / Goodbye도 여기서 선택합니다.
그룹 제목으로 목록을 접고 펼치며, **그룹 편집**에서 그룹 추가·이름 변경·그래프 이동·그룹 해제 후
**그룹 적용**으로 저장합니다. 공명 소비 예제 9개는 한 그룹으로 묶여 있습니다.
다른 위치의 파일은 **다른 위치에서 가져오기**, 다운로드 사본은 **JSON 내보내기**를 사용합니다.

첫 전투 맵 편집기 작업을 다른 AI에게 전달할 때는
[프로토타입 구현 프롬프트](docs/map-assembler-prototype-prompt.md)를 사용합니다.

전투 맵 조립기 프로토타입을 구현했습니다. 비정형 유효 영역·지형·구역·경로·이름 있는
지점·임시 개체 슬롯·다섯 조건의 웨이브를 작성하고, 진단·실행 취소·다시 실행과
JSON 다운로드/재열기로 작업을 이어갈 수 있습니다. 밝게/어둡게/시스템 테마를 지원합니다.
게임 코어 연동, 전투 실행, 길 찾기, 최종 코어 내보내기는 포함하지 않습니다.

사용 방법과 저장 형식·좌표·삭제 정책·제한은
[프로토타입 안내](docs/map-assembler-prototype.md)에 정리했습니다.
검사 결과와 직접 검토한 PNG는 [검증 기록](docs/map-assembler-validation.md)을 확인하세요.
맵·기존 스킬 화면의 `저장`은 브라우저 다운로드 요청입니다. 파일을 확인한 뒤 `다운로드 확인`을 눌러 저장 기준을
갱신하며, 앱이 디스크 저장 완료를 자동 확인했다고 표시하지 않습니다.

## 설치

이 디렉터리에서 명령을 실행합니다. 권장 Node 버전은 `.nvmrc`에 기록되어 있습니다.

```sh
cd content_assembler
npm ci
```

`package-lock.json`, `src-tauri/Cargo.lock`을 함께 관리합니다.
Windows와 WSL 사이에서 개발 환경을 바꿀 때는 해당 환경에서 `npm ci`를 다시 실행합니다.
Vite/Tauri의 네이티브 npm 패키지는 운영체제별로 다릅니다.

## 앱을 실행하지 않는 검사

```sh
npm run check
npm run build
npm run test
cargo fmt --manifest-path src-tauri/Cargo.toml -- --check
```

- `check`: TypeScript, ESLint, Prettier 검사.
- `test`: 서버/브라우저를 시작하지 않는 편집 규칙·이력·직렬화·파일 실패/순서 검사.
- `build`: 타입 검사와 프런트엔드 정적 번들 생성. 서버나 앱 창을 열지 않습니다.
- `npm run check:rust`: Rust 포맷 및 네이티브 컴파일 검사. 아래 운영체제별 의존성이 필요합니다.
- `npm run licenses`: 설치된 프런트엔드 런타임 의존성의 고지 파일을 갱신합니다.

## 운영체제별 네이티브 의존성

### Ubuntu / WSL

Tauri의 Linux 빌드에는 GTK와 WebKit 개발 패키지가 필요합니다.
현재 환경 구축 과정에서는 관리자 암호가 필요해 이 시스템 패키지들을 설치하지 못했습니다.
사용자가 아래 명령으로 설치한 뒤 `npm run check:rust`를 실행할 수 있습니다.

```sh
sudo apt-get update
sudo apt-get install --no-install-recommends \
  build-essential pkg-config curl wget file \
  libwebkit2gtk-4.1-dev libgtk-3-dev libxdo-dev \
  libayatana-appindicator3-dev librsvg2-dev patchelf
```

WSL에서 기본 Tauri 명령은 Linux 앱을 대상으로 합니다.
WSLg 등 그래픽 환경은 앱을 실제 실행할 때 필요합니다.

### Windows

Windows 앱은 Windows의 Node/npm 및 Rust MSVC 도구 체인으로 빌드합니다.
Visual Studio C++ Build Tools, Windows SDK, WebView2 Runtime이 필요합니다.
구체적인 설치 항목은 [Tauri 공식 준비 문서](https://v2.tauri.app/start/prerequisites/)를 따릅니다.

Tailwind CSS 4의 최소 브라우저 기능 기준을 적용합니다.
Windows는 Chrome 111에 상응하는 WebView2 이상, macOS는 Safari 16.4에 상응하는
WKWebView 이상이 필요하며, Linux의 WebKitGTK도 해당 CSS 기능을 지원해야 합니다.
실제 지원 운영체제 범위는 배포 전에 네이티브 환경에서 확인합니다.

## 사용자가 실행을 원할 때

브라우저 편집기 또는 Tauri 호스트를 실행합니다. 이 프로토타입의 실제 UI 검증은 WSL Chromium을 대상으로 합니다.

```sh
npm run dev          # Vite 개발 서버만 시작
npm run desktop:dev  # Vite 서버와 Tauri 데스크톱 앱 시작
```

브라우저의 노드 편집은 TypeScript에서 실행하므로 Cargo나 별도 편집 서버가 필요하지 않습니다.
개발 서버와 정적 빌드 모두 같은 편집 코어를 사용합니다. Rust는 Tauri 데스크톱 호스트 빌드에 필요합니다.

설치 파일 생성은 `npm run desktop:build`를 사용합니다. 네이티브 의존성이 필요합니다.

## 구조와 확장

AI 작업 규칙은 [AGENTS.md](AGENTS.md), 코드 작성·설계·검토 기준은
[개발 지침](docs/development-guidelines.md), 스크린샷·동작·접근성 검사 방법은
[UI 검증 안내](docs/ui-validation.md)에 정리되어 있습니다.
`npm run ui:list`는 앱을 실행하지 않고 검사 설정을 확인합니다.
`npm run ui:inspect`는 전용 개발 서버와 브라우저를 실행해 화면을 촬영하고 검사합니다.
프로토타입 작업의 화면·동작 검사 결과는 `artifacts/ui/report.json`과 스크린샷에서 확인합니다.
검토한 비교 기준 이미지는 아직 생성하지 않았습니다.

- `src/skill/`: 스킬 설계 문서·블록 편집·상태 참조·진단·이력·파일 경계.
- `src/nodes/core/`: React·Tauri와 독립된 문서·카탈로그·편집·검증·패치 이력·저장 코어.
- `src/nodes/store.ts`: 확정된 편집 상태와 화면 알림.
- `src/nodes/`: 노드 화면, 미확정 드래그, 속성 입력·파일 입출력.
- `src/editor/`: 전투 맵 문서·순수 편집 규칙·이력·파일 경계·캔버스와 속성 패널.
- `tests/model/`: 순수 규칙과 비동기 입출력 검사, 고정 fixture.
- `tests/ui/`: 실제 파일 업로드/다운로드를 이용하는 편집 시나리오와 화면 검사.
- `src/app/`: 전역 UI 설정과 외부 환경 동기화.
- `src/components/ui/`: 공식 shadcn/ui에서 가져온 Radix 기반 컴포넌트.
- `src/components/`: 제작기에서 사용하는 공통 UI.
- `src/stores/`: Zustand UI 상태. 게임 문서 원본이나 실행 취소 이력을 테마 저장소에 넣지 않습니다.
- `src/index.css`: Tailwind, 공통 테마 토큰과 기본 스타일.
- `src-tauri/`: 데스크톱 호스트. 루트 게임 Cargo workspace와 독립적인 workspace입니다.

색상·간격·모서리 규칙은 공통 테마를 사용하고, 반복되는 속성 행과 입력 위젯은
공통 컴포넌트로 추출합니다. 게임 데이터·검증·편집 명령은 UI 라이브러리와 분리합니다.

현재 속성 폼은 작은 입력 버퍼 컴포넌트를 사용합니다. React Hook Form은 기존 의존성으로 유지합니다.
스킬 화면은 React Flow와 ELK 자동 배치로 시작 사건별 노드 그래프를 표시합니다.
Konva, Dockview, AG Grid, Monaco는 해당 기능을 구현할 때 별도로 도입합니다.

shadcn 컴포넌트는 `components.json`의 Radix 설정을 사용해 추가합니다.
초기 생성에는 shadcn CLI 4.21.4를 사용했습니다. 생성 CLI는 상시 의존성에서 제외했으며,
필요한 공통 CSS는 `src/styles/shadcn.css`에 MIT 고지와 함께 보존했습니다.
CLI의 fast-glob/braces 의존성에는 구축 시점의 npm audit 경고가 있어,
후속 생성 작업에서는 수정된 CLI 버전을 먼저 확인합니다.

```sh
# 패치된 CLI 버전을 확인한 뒤 명시적으로 지정합니다.
npm exec --yes --package=shadcn@VERSION -- shadcn add COMPONENT
```

이는 소스 파일과 의존성을 변경하는 명령입니다. 생성된 파일의 클래스 병합은
`@/lib/utils`를 사용하도록 맞추고, 공통 CSS 변경도 함께 확인합니다.
추가 후 `npm run licenses`와 `npm run check`를 실행하고 변경 내용을 검토합니다.

## 라이선스

프로젝트 자체는 비공개 소스로 취급하며 npm 배포를 막기 위해 `private: true`,
`license: UNLICENSED`로 설정되어 있습니다. 의존성의 라이선스와는 별개입니다.

- React, Zustand, React Hook Form, Radix, shadcn/ui, Tailwind CSS는 MIT 기반입니다.
- React Flow는 MIT이며, ELK.js는 EPL-2.0을 선택해 사용합니다.
  원본은 [ELK.js 저장소](https://github.com/kieler/elkjs)와
  [0.12.0 패키지](https://registry.npmjs.org/elkjs/-/elkjs-0.12.0.tgz)에서 확인할 수 있습니다.
- Lucide는 ISC 및 포함된 아이콘의 고지가 있으므로 배포물에 고지 파일을 함께 포함합니다.
- Tauri는 MIT 또는 Apache-2.0입니다.
- 복사한 shadcn/ui 코드의 원문 고지는 `licenses/shadcn-ui.LICENSE.txt`에 보존합니다.
- `public/third-party-notices.txt`는 설치된 npm 런타임 의존성과 복사한 UI 코드의 고지를
  수집하며, Vite 빌드 결과에도 포함됩니다.
- 이 npm 고지 파일은 Rust crate와 운영체제 라이브러리 전체의 라이선스 검토를 대체하지 않습니다.
  네이티브 배포 시에는 `src-tauri/Cargo.lock`과 실제 번들에 포함되는 항목을 기준으로 확인합니다.

Enterprise 패키지나 유료 템플릿은 이 환경에 포함되어 있지 않습니다.
