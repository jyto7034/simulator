# 보존 정책·조건부 관계·미정 질문

**2.2-review.1 · 2026-09-09 · 조사·개정 권고 / 저장소 미적용 / 게임 구현·Goal 실행 미승인**

승인된 사본별 개화, 직원별 공용 3슬롯, 외부 팩/새 런 고정, 경로별 재방문, 최신 침식 방향은 다시 미정으로 돌리지 않는다. 원래 PG 식별자를 유지하고 미정 family만 제한한다. 새 가격·수치·우선순위·손실 의미를 이 문서가 승인하지 않는다.

## PG00 — 승인 구조·원문 공백·prototype 수치의 구별

Owner: [R00](goals/R00.ko.md), [R01](goals/R01.ko.md), [R21](goals/R21.ko.md), [R24](goals/R24.ko.md).

**유지:** CA00의 354 COV/127 BOSS-COV 기준선과 읽기 기록을 인수한다. 서사·원작·초안·미정·기술 제안을 승인 gameplay로 바꾸지 않는다.

**조건/미정:** 전체 개별행 fresh 재검증/실제 실행은 미완료이며 R21 인수에서 별도 상태를 유지한다.

## PG01 — typed property·Add/Scale/Replace·비호환

Owner: [R06](goals/R06.ko.md), [R14](goals/R14.ko.md), [R15](goals/R15.ko.md).

**유지:** 다른 의미의 Percent를 동일 연산으로 합치지 않는다. 같은 effect의 태그와 전용 콘텐츠 조건은 별도 AND다.

**조건/미정:** 미정 수식/우선순위는 해당 property/recipe만 중단. 서로 다른 Replace를 UUID/장착순서로 해결하지 않는다.

## PG02 — 활성 closure·연쇄 root/fork/join

Owner: [R04](goals/R04.ko.md), [R15](goals/R15.ko.md), [R16](goals/R16.ko.md), [R17](goals/R17.ko.md).

**유지:** compile-time 유한 활성과 runtime 경로별 (copy,effect) 재방문은 별도다. 형제 경로는 전역 visited가 아니다.

**조건/미정:** 진짜 join·정기 event root의 exact 정책이 없으면 그 family를 미지원/정책대기로 남긴다.

## PG03 — 타격 인정·snapshot·동시 시점·cast 의미

Owner: [R04](goals/R04.ko.md), [R06](goals/R06.ko.md), [R08](goals/R08.ko.md), [R09](goals/R09.ko.md), [R10](goals/R10.ko.md), [R11](goals/R11.ko.md), [R18](goals/R18.ko.md).

**유지:** 현재 HP-loss gate/basic OnAttack/OnHit는 보존한다. 새로운 실제 타격 인정·같은시각 우선순위는 해당 규칙으로 닫는다.

**조건/미정:** 침식의 실제 cast 시작 vs Pause 중 admission/준비/release, reinput와 timeout, pre-death 관찰의 미정 경계를 명시한다.

## PG04 — 감쇄·pool·HP비용·패링·생존·특수 kill

Owner: [R12](goals/R12.ko.md), [R13](goals/R13.ko.md), [R17](goals/R17.ko.md), [R18](goals/R18.ko.md).

**유지:** 공유량을 수혜자 수로 복제하지 않는다. HP비용은 Damage로 위장하지 않는다. 침식 특수 kill과 ordinary shield는 구별한다.

**조건/미정:** 개인/공유 보호막 순서·HP1·처형/패링 중재·예외 수치는 임의로 확정하지 않는다.

## PG05 — 사본/형태·자원 수명·성장·저장

Owner: [R05](goals/R05.ko.md), [R06](goals/R06.ko.md), [R07](goals/R07.ko.md), [R09](goals/R09.ko.md), [R14](goals/R14.ko.md), [R20](goals/R20.ko.md).

**유지:** 개화 해금은 파편 사본별. 소유 직원 이전·form 변화·재배치·attempt 종료는 다른 사건이다.

**조건/미정:** 형태 변경/이전의 자원 보존 세부, 신규 저장 손실/호환 정책은 해당 formal owner가 기존 결정과 대조한다.

## PG06 — 공식 콘텐츠 수치·recipe·획득/선택

Owner: [R07](goals/R07.ko.md), [R13](goals/R13.ko.md), [R14](goals/R14.ko.md), [R15](goals/R15.ko.md), [R17](goals/R17.ko.md), [R18](goals/R18.ko.md), [R19](goals/R19.ko.md), [R21](goals/R21.ko.md).

**유지:** 기존 승인 구조·동일 사본 독점·직원 3슬롯은 유지한다. 일반/퍼펙트 보상·미정 획득/분쇄량을 발명하지 않는다.

**조건/미정:** 공식 S/지속시간/확률/재료량 미정은 test-only 입력과 분리한다. 정책 대기를 완료로 치환하지 않는다.

## PG07 — 외부 팩 합성·새 런 세대·fingerprint

Owner: [R02](goals/R02.ko.md), [R03](goals/R03.ko.md), [R19](goals/R19.ko.md), [R20](goals/R20.ko.md).

**유지:** 명시 replace/충돌/최종참조 검증, 새 런 pin, 실패 candidate 미게시. 사용자 live_data 자동 덮어쓰기 없음.

**조건/미정:** 정확한 manifest/fingerprint 표현·symlink 정책은 기술 계약으로 닫되 저장 손실/새 호환 정책을 대신 승인하지 않는다.

## PG08 — 실행 예산·성능 측정·Certified 가중

Owner: [R02](goals/R02.ko.md), [R03](goals/R03.ko.md), [R04](goals/R04.ko.md), [R06](goals/R06.ko.md), [R10](goals/R10.ko.md), [R13](goals/R13.ko.md), [R16](goals/R16.ko.md), [R19](goals/R19.ko.md), [R23](goals/R23.ko.md).

**유지:** 기능 추가부터 유한성/예산/fault를 소유하고 최종 현실 부하에서 측정한다. 현재 skill 존재 정액을 새 효과 평가로 포장하지 않는다.

**조건/미정:** 안전 상한과 gameplay cap 구별. 목표 장비·측정 기준·새 평가 가중은 근거/정책 없이 채우지 않는다.

## PG09 — 침식된 유물 상태 vs 파편 침식

Owner: [R09](goals/R09.ko.md), [R14](goals/R14.ko.md), [R15](goals/R15.ko.md), [R21](goals/R21.ko.md).

**유지:** 둘은 다른 기능이다. 파편 침식은 세트4종/별도버튼 없이 실제 시전 조건 대체, 최종 보스 파편 제외다.

**조건/미정:** 공통 trauma threshold·개별 authored 연출 지속시간·침식 유물 세부는 관련 가족만 gate한다.

## 침식의 구체적인 남은 질문

1. `world/combat`의 Pause 중 수동 접수와 `sim`의 pending start_seq/focus 종료 사이에서 **한 번의 실제 cast-start 결정**을 어디에 두어야 기존 일반 접수/자원/취소 의미를 유지하는가? 기술 후보는 resume 후 첫 적격 battle-advance의 release 이전 지점이다. 이를 이미 승인된 hook이라고 단정하지 않는다.
2. 준비 도중 자격/trauma가 변하는 경우 어느 snapshot을 인정하는가? 기존 접수 권한을 보존하면서 침식 조건의 “실제 시작” 의미가 바뀌면 P2G-10의 최소 정책 질문으로 반환한다.
3. node 완료가 Breakdown 한계와 SavePoint/Gate 회복을 동시에 가져올 때 만료와 회복의 순서를 어떻게 적용하는가? 일반 node 완료당 1, retreat 미증가, trauma<100 즉시 폐기, checkpoint 잔여수 복원은 이미 승인돼 있다.

이 질문들은 R09의 해당 계약을 제한한다. R01에 전체 침식 선구현을 요구하는 근거는 아니다. 근거: [S018](../evidence/SOURCES.ko.md#s018) · [S040](../evidence/SOURCES.ko.md#s040) · [S044](../evidence/SOURCES.ko.md#s044) · [S046](../evidence/SOURCES.ko.md#s046) · [S059](../evidence/SOURCES.ko.md#s059) · [S060](../evidence/SOURCES.ko.md#s060) · [S061](../evidence/SOURCES.ko.md#s061) · [S077](../evidence/SOURCES.ko.md#s077).
