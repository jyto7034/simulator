# R06 단일 R 요청 인수 — 2026-09-13

이번 active thread goal은 사용자가 채택한 [AUTORUN single-r-v3](../../roadmaps/content_authoring/AUTORUN_REQUEST.ko.md)의 단일 R 범위다. 종료된 R05 다음의 첫 미완료인 **R06 — ValueExpr·집계·Attack/SpellPower와 무기 성장**으로 선정·고정한다. 자동 재개나 현황 변경에서 다시 선정하지 않는다. R06 카드의 applicable 요구·직접 검증·필수 감사가 모두 완료되면 이번 Goal을 종료하며 R07은 새 사용자 요청에서만 시작한다.

## 선정과 현재 basis

- ROADMAP `2.2-review.1` 순서와 current `ROADMAP_STATUS.md`를 대조했다. R00 인수와 R01–R05의 current successor 감사가 닫혔고 R06은 `proposed_not_implemented`, 정식 Topic 미존재였으므로 첫 미완료다.
- Branch `cleanup/simulator-dirty-worktree-20260803`, HEAD `f318414fb0156a67e04c5479000c2a2683fc0c09`. 인수 시 `git status --porcelain=v1`은 389행(추적 206, 미추적 183)이었다. 기존 dirty를 reset/clean/stash하지 않고 이번 파일과 후속 R06 변경을 구별한다.
- `core_enhanced/docs/goals/content_authoring_value_expressions/WORKING_POLICY.ko.md`는 미점유였고 `WPD-CAVE-20260913` identity 검색 결과가 없었다. P2G-00은 `PIS-CAVE-P2G00-001`과 `WR-CAVE-P2G00-001`만 새로 발행한다.
- 현재 적용 기준은 P2G `2026-09-11-r5`, GEX `2026-09-12-gex-r5`, AUTORUN `single-r-v3`다. workspace 범위에서 `AGENTS.md`를 찾지 못했으며 현재 developer/runner 지침을 함께 적용한다.
- R01·R04·R05 전체를 재실행하지 않는다. R06 P2G-20에서 직접 소비하는 producer output의 currentness와 실제 source 경계만 대조하고, GEX-00에서 exact predecessor target을 다시 확인한다.

## 승인 범위와 종료 조건

R06은 유한 typed `ValueExpr`, 단위와 checked arithmetic, snapshot/live 평가, resource/result/stat/HP/time 참조, 유한 Count/Sum/Max, Attack/SpellPower와 무기 성장의 단일 합산, 공격속도 point↔interval 구분, AT06의 profile/stat/projection/save 검증, 공개 authoring/validation/compiler/executor와 RON negative를 소유한다. T06과 R06에 적용되는 T18/T27, 결정론·예산·fault, 직접 소비자·계약·설명/preview 입력을 함께 닫는다.

R07의 파편 사본화·구형 rank 제거, R19의 Certified 최종 가중 모델, 공식 밸런스/공급 수치, R07–R24의 신규 기능, 서버/Unity/UI/외부 배포, 개인 저장 삭제·자동변환, P2G/GEX/validator/AGENTS/global 설정 변경은 제외한다. 미정 Replace 우선순위나 평가 가중을 UUID·장착 순서·placeholder로 채우지 않는다.

동일 R의 P2G-60까지 연속 진행과 자격을 갖춘 exact Goal의 GEX 구현·검증·fresh 감사가 이번 상위 요청으로 승인됐다. 통상적 정책은 P2G §8.6의 다섯 조건과 exact 결정을 owner artifact에 기록하고 진행한다. 사용자 선택이 필요한 의미가 드러날 때만 질문하고 주 작업을 대기한다. P2G 문서, 코드 작성, test PASS, 감사 완료와 R06 완료를 서로 대체하지 않는다.

## 마지막 유효 결과와 다음 작업

P2G-00은 [초기 artifact](P2G_00_INITIAL_001.json)와 [완료 결과](P2G_00_RESULT_001.json)로 닫혔다. 이는 scope/raw input bootstrap이며 정책 검토·코드 currentness·Goal/Handoff·GEX 자격이나 구현 완료가 아니다. 다음 안전한 owner는 P2G-10이다. R06의 정책 요소를 분해하고 PG01/03/05/08을 기존 합의와 위임 기준에 따라 결정하거나 실제 사용자 질문으로 분류한다.
