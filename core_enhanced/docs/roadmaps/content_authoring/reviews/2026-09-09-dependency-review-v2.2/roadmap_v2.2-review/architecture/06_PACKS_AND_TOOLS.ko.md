# A06 — 외부 팩·세대·작성 포트

**2.2-review.1 · 2026-09-09 · 조사·개정 권고 / 저장소 미적용 / 게임 구현·Goal 실행 미승인**

원래 논리 설계와 승인 목표는 유지하고 새 R owner/현재 코드 전환 경계를 반영했다. 새 문법·타입·포트 이름은 실제 구현 존재의 증거가 아니다.

## 1. Source와 실행 입력

`game_resources/data`는 공식 authoring 원본, `live_data`는 선택된 runtime pack 입력이다. 자동 상호 덮어쓰기를 하지 않는다. 파일 감시기는 reload 요청을 만들 수 있을 뿐 진행 중 전투의 정의를 바꾸지 않는다. effect마다 파일을 다시 읽지 않는다.

## 2. Manifest와 명시 합성

pack ID/version·engine authoring/capability·dependencies·file inventory·명시 add/replace/remove를 검증한다. pack dependency 순서가 충돌 winner를 정하지 않는다. replacement chain 또는 명시 선택이 필요하다. 기본 full-definition Replace와 좁은 typed patch를 구별하고 최종 resolved catalog의 참조를 검사한다.

## 3. 입력 일관성과 안전

source bytes/path/hash를 확보하고 파일 수/크기·parser 깊이·경로 정규화/루트 탈출·절대경로·symlink·case 충돌 정책을 검증한다. 느슨한 디렉터리 읽기 여러 번이 원자적 파일시스템 snapshot은 아니다. 동시 변경은 bounded retry/명시 실패, 필요하면 immutable directory+manifest 교체를 택한다.

## 4. Publication/new-run/test reload

R02는 validate→compile→generated setup 뒤 불변 세대만 게시한다. 새 런은 briefing/starter부터 pin한다. 현재 world commit은 state/seed만 옮기므로 generation-aware factory/commit을 명시해야 한다. B실패 때 기존 A런은 유지하되 B요청을 A로 몰래 성공시키지 않는다. 시험 reload는 B검증과 새 시험 session 생성이 모두 성공한 뒤 교체한다.

## 5. Fingerprint와 저장

source bytes hash와 최종 의미 fingerprint를 구별한다. save는 최신 schema/contract/resolved content fingerprint를 기록하고 같은ID다른내용을 거부한다. 기존 세대의 메모리 강한 참조 유지와 구형 parser 이중 지원은 다른 문제다. 오래된 저장을 이어갈 수 있는 내용 구성을 사용자가 복원하거나 새 런을 시작할 수 있지만 파일을 임의 삭제/변환하지 않는다.

## 6. Describe/Validate/Explain/Simulate

R03의 library API를 단일 의미 구현으로 두고 CLI는 얇은 adapter다. Cargo의 autobins=false를 고려해 실제 public binary를 명시 등록한다. schema/API에 없는 fixture-only builder를 제작 SDK로 포장하지 않는다. source/expected/actual/관련정의/지원 capability를 진단하고 Explain/Preview는 state/RNG/revision을 바꾸지 않는다.

## 7. Trace와 비용

Trace는 source map, semantic occurrence, action milestone, resource before/after, composition decision/차단 chain을 설명한다. diagnostic history의 retention과 실제 ChainContext를 분리한다. retained generation/cache와 runtime execution budget을 각 도입 owner가 계측하고 R23에서 최종 측정한다.

원문 설계 근거: [S085](../../evidence/SOURCES.ko.md#s085). 현재 코드 근거와 각 단계의 상세 동시 전환은 [DEPENDENCIES](../DEPENDENCIES.ko.md) 및 [조사 결과](../../report/REVIEW.ko.md)를 따른다.
