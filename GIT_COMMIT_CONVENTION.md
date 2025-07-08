# Git 커밋 메시지 컨벤션

이 문서는 우리 프로젝트에서 사용하는 Git 커밋 메시지 작성 규칙을 정의한다. 일관성 있는 커밋 히스토리는 코드 변경 사항을 이해하고, 특정 기능을 추적하며, 협업을 원활하게 하는 데 매우 중요하다. 우리는 [Conventional Commits](https://www.conventionalcommits.org/) 명세를 기반으로 한다.

---

## 커밋 메시지 구조

모든 커밋 메시지는 다음 구조를 따른다.

```
<type>(<scope>): <subject>
<BLANK LINE>
<body>
<BLANK LINE>
<footer>
```

-   **Header:** `type`, `scope`, `subject`을 포함하는 첫 번째 줄은 필수이다.
-   **Body:** 변경 사항에 대한 상세한 설명. 선택 사항이다.
-   **Footer:** 관련된 이슈 번호(예: `Closes #123`)나 주요 변경점(Breaking Change)을 명시. 선택 사항이다.

---

### 1. 타입 (Type)

커밋의 성격을 나타내는 가장 중요한 부분이다. 아래 목록 중 하나를 사용해야 한다.

-   **feat**: 새로운 기능 추가 (a new feature)
-   **fix**: 버그 수정 (a bug fix)
-   **docs**: 문서만 변경 (documentation only changes)
-   **style**: 코드 의미에 영향을 주지 않는 서식 변경 (예: 공백, 세미콜론 등)
-   **refactor**: 버그를 수정하거나 기능을 추가하지 않는 코드 구조 변경
-   **perf**: 성능을 개선하는 코드 변경 (a code change that improves performance)
-   **test**: 누락된 테스트를 추가하거나 기존 테스트를 수정
-   **build**: 빌드 시스템이나 외부 종속성에 영향을 미치는 변경 (예: `Cargo.toml`, `package.json`)
-   **ci**: CI 구성 파일 및 스크립트 변경 (예: `.github/workflows/*.yml`)
-   **chore**: 소스 코드나 테스트 파일을 수정하지 않는 기타 변경 (예: `.gitignore` 수정)

### 2. 스코프 (Scope) - 선택 사항

커밋이 영향을 미치는 코드의 범위를 괄호 안에 명시한다. 스코프는 변경된 부분의 컨텍스트를 빠르게 파악하는 데 도움을 준다.

-   **예시:** `(matchmaker)`, `(auth_server)`, `(gamelift)`, `(ci)`, `(deps)`

### 3. 제목 (Subject)

변경 사항에 대한 50자 이내의 간결한 요약.

-   **규칙:**
    -   명령문으로 작성한다. (예: "Fix" not "Fixed", "Add" not "Added")
    -   첫 글자는 대문자로 작성한다.
    -   문장 끝에 마침표(.)를 찍지 않는다.

---

## 예시

**좋은 예시:**

```
feat(matchmaker): Add ranked 1v1 game mode
```

```
fix(server): Fix race condition in loading complete handler

Uses a Redis Lua script to ensure the process of checking and
updating player readiness is atomic. This prevents multiple game
sessions from being created for the same group of players.

Closes #42
```

```
refactor(matchmaker): Split monolithic actor file into modules

- Separate the `matchmaker/actor.rs` file into a structured module
  with distinct responsibilities (actor, messages, handlers, scripts).
- This improves code organization, readability, and maintainability.
```

**나쁜 예시:**

```
fixed a bug  // 타입과 스코프가 없고, 과거형이며, 대문자로 시작하지 않음
```

```
Update code // 무엇을, 왜 했는지 알 수 없는 모호한 메시지
```
