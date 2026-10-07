# A02 — 작성·compiler·지원 descriptor

**2.2-review.1 · 2026-09-09 · 조사·개정 권고 / 저장소 미적용 / 게임 구현·Goal 실행 미승인**

원래 논리 설계와 승인 목표는 유지하고 새 R owner/현재 코드 전환 경계를 반영했다. 새 문법·타입·포트 이름은 실제 구현 존재의 증거가 아니다.

## 1. 의미 단위

EffectSite는 개별 효과, ActionSite는 실제 target/delivery/완료 milestone을 소유하는 행동 주소다. tags/property는 타입과 capability가 있는 지원 표면이며 내부 Rust/JSON field path가 아니다. 같은 effect의 태그 조건과 전용 definition/form identity 조건을 구별한다.

## 2. 최소 전환과 확장

R01의 AG는 literal Damage/Heal/Instant metadata를 현 parser/catalog/runtime에 연결한다. 현재 ModifyDamage는 step-local이며 step 공유 target/delivery·효과 순서·signed Heal을 보존한다. R02에서 static source path를 owned pack/file/source span으로 확장한다. R06의 Expr와 R10 Flow는 실제 지원하는 새 typed vocabulary와 executor/시험을 함께 추가한다.

## 3. Compile 흐름

source bytes와 locator 확보 → raw schema/타입 → pack dependency/명시 합성 → 최종 참조 → effect/Flow/Boss domain compile → bounded graph/작성 지원 검사 → generated setup 검증 → 불변 세대 게시. 최종 loser 정의의 더 이상 쓰이지 않는 참조로 유효한 resolved catalog를 막지 않되 raw syntax/manifest 검증을 건너뛰지 않는다.

## 4. 식·Flow·결과 검증

값의 단위/부호/범위·산술·overflow·empty aggregation·평가 시점을 검증한다. result는 생산된 범위에서만 참조하고 무한 loop/자기부트스트랩을 허용하지 않는다. 자유 스크립트/일반 bytecode/임의 JSON patch로 타입과 authority 경계를 우회하지 않는다. 15개 분해 축을 그대로 15개 schema field로 만들지 않는다.

## 5. SourceMap과 지원 정보

source locator, definition/effect/action 주소와 compile index 매핑을 보존한다. runtime-supported는 parser/compiler/consumer와 음성 시험까지 존재하는 조합에만 붙인다. unsupported를 warning 뒤 무시하거나 빈 success handler로 공개하지 않는다. 현재 예제 RON은 채택된 schema로 실제 fixture가 된 뒤에만 실행 증거가 된다.

원문 설계 근거: [S081](../../evidence/SOURCES.ko.md#s081). 현재 코드 근거와 각 단계의 상세 동시 전환은 [DEPENDENCIES](../DEPENDENCIES.ko.md) 및 [조사 결과](../../report/REVIEW.ko.md)를 따른다.
