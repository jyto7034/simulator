# A04 — 효과 합성·활성 closure·연쇄

**2.2-review.1 · 2026-09-09 · 조사·개정 권고 / 저장소 미적용 / 게임 구현·Goal 실행 미승인**

원래 논리 설계와 승인 목표는 유지하고 새 R owner/현재 코드 전환 경계를 반영했다. 새 문법·타입·포트 이름은 실제 구현 존재의 증거가 아니다.

## 1. 관리와 합성

RelicCopy/직원별 3슬롯/독점/비호환/이전·회수·분쇄는 관리 aggregate다. Composer는 지원된 원형 효과와 loadout binding으로 개인 immutable effective plan을 만든다. 요구 효과가 없으면 장착 거부가 아니라 휴면으로 남긴다. 다른 직원의 조합을 직원 내 비호환으로 막지 않는다.

## 2. 최소 numeric slice

R14는 관리와 최소 numeric 실행을 한 작업 단위로 인수하되 도메인 책임은 분리한다. current cast/queued step뿐 아니라 projectile/area impact가 전역 SkillId를 다시 읽는 경계를 전환한다. 정의 catalog 설명은 유지하고 개인 preview/explain/평가 입력은 effective binding을 받아야 한다. 같은 SkillId의 직원/사본/형태/세대가 cache에서 섞이면 안 된다.

## 3. Selector와 단계화

전용 콘텐츠/형태 identity AND 해당 effect의 role/tags/property capability를 검사한다. 파편 전체 태그 합집합으로 매칭하지 않는다. 긍정 grant의 유한 최소 closure → activation/dormancy → 구조/태그/수치 변경의 명시 단계 → 최종 graph 검증 순으로 처리한다. 의미 없는 자기부트스트랩과 negative cycle은 거부하거나 승인된 층화만 허용한다.

## 4. Write/preserve 계약

Add/Scale/Replace의 타입·단위·합성 순서는 별도다. incompatible Replace는 마지막 장착/UUID 순서로 해결하지 않는다. 구조 recipe는 input/output 및 owner/source/lifetime/채널/영향받지 않는 효과의 preserve set을 선언한다. 포큐버스 2S와 태아 상태별 두 property의 교체값은 공통 무조건 ×2가 아니다. generated effect 주소는 provider copy/effect/input/local identity로 안정화한다.

## 5. Compile closure와 runtime chain

R15의 활성 closure는 runtime 반응의 visited 집합이 아니다. R16은 branch-local (copy_id,effect_id) ancestor를 즉시/지연 자식에 보존한다. A1.e→B1.e→A1.e만 차단하고 A2/다른 effect/독립공격은 구별한다. 진짜 join·정기 root 권한은 명시 정책이 없으면 해당 family를 미지원으로 남긴다. timer마다 새 root를 주어 순환 제한을 우회하지 않는다.

## 6. 관계/감염 recipe

R17이 object lifecycle·source-scoped membership·relation graph와 GroupByTile를 결합할 때 composer descriptor/validation/실행도 같이 확장한다. 설치물 연결망과 감염 집합을 같은 이름 때문에 하나의 global graph로 강제하지 않는다.

원문 설계 근거: [S083](../../evidence/SOURCES.ko.md#s083). 현재 코드 근거와 각 단계의 상세 동시 전환은 [DEPENDENCIES](../DEPENDENCIES.ko.md) 및 [조사 결과](../../report/REVIEW.ko.md)를 따른다.
