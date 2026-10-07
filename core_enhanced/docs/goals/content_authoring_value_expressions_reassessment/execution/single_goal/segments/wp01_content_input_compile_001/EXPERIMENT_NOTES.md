# R06 WP01 콘텐츠 입력·검증·컴파일 현재 기술 메모

Current state: `부분 구현·검증 종료 / 단계검토 요청 대기`

- 승인 의미는 `CODE_REVIEW_003.ko.md` D01, D07/D08, D09/D10과 Goal Set이 소유한다. 이 메모는 새 policy authority가 아니다.
- 이번 기술 선택은 RON 0.5.1에서 expression span을 acquisition 시 먼저 제한하고, AST node 진입 전 bounded visitor meter를 적용한 뒤 기존 iterative compiler에서 다시 예산을 승인하는 이중 경계다.
- RON 0.5.1의 실제 숫자 visitor는 i128 entry를 지원하지 않는다. authored 상수는 parser가 안전하게 제공할 수 있는 i64/u64 범위이며 내부 값만 i128로 확장한다. 계산/overflow 계약의 signed i128은 유지된다.
- immutable catalog는 final winning `EffectSite`별 plan만 소유하며 evaluation result나 runtime capability를 소유하지 않는다.
- expression publication 차단은 WP02 미구현을 숨기는 임시 상수/fallback이 아니라 명시적 support boundary다. literal runtime behavior는 이번 구간에서 전환하지 않는다.
- old-build V7/literal fixtures와 기존 correction/review artifacts는 불변이다.
- known remaining: actual battle frame/capture/resource/result/execution, quantitative consumers, stat/save 전환, public runtime capability와 전체 Goal validation.
- external pack acquisition/publication source relocation과 invalid moment/syntax/literal/content/save 회귀는 checkpoint successor에서 구현·검증했다.
- 단계검토에서 compiler 전 AST clone과 실패 진단의 전체 RON 재parse를 찾아 각각 borrow와 lexical locator로 교정했다. compact 익명 tuple source 위치 회귀도 final pipeline 전에 교정했다.
- 새 target은 parse/compile 가능한 authored expression을 immutable catalog까지 보존하지만 production publication은 `content_capability_unsupported`로 차단한다.
- known remaining은 WP02 actual battle frame/capture/resource/result/execution, quantitative consumers, stat/save 전환, public runtime capability와 전체 Goal validation이다. 이번 요청 권한으로 시작하지 않는다.
