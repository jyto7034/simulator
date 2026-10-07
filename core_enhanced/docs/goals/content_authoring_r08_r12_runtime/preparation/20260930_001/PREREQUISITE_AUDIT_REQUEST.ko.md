# 재개에 필요한 R06·R07 독립 감사

이 문서는 감사 요청용 준비 자료다. 이번 작업에서 감사는 수행하지 않았고 감사 verdict를 발행하지 않는다.
현재 사용자 요청 및 R08~R12 상세 지시서 §2가 선행 감사 인수를 요구한다. 자체 검증 완료 인계는 이를 대신하지 않는다.

## exact 대상

저장소 root는 `/mnt/f/work/simulator`다. 아래 경로는 root 상대 경로다.

- 최종 구현 인계: `core_enhanced/docs/goals/content_authoring_fragment_copies/execution/single_goal/REVIEW_HANDOFF.md`
- 최종 target: `core_enhanced/docs/goals/content_authoring_fragment_copies/execution/single_goal/target/r06_r07_integrated_001/MANIFEST.json`
- 최종 manifest SHA256: `22b61c4eb0518fcc11209eaf00e415a4f8b6f283c73037d9a04e8e5c168df59a`
- 최종 source digest: `714195327ec0df7facc60d1aafe721e6e26c1296494b39cff593232c45365e6f`
- R06 경계: `core_enhanced/docs/goals/content_authoring_value_expressions_reassessment/execution/single_goal/continuations/r06_completion_001/target/r06_boundary_001/MANIFEST.json`
- 경계 SHA256: `0b6938d77cc6cded598b2b01e51d41bf7e651dd66e8acf2f695a1cf3b5fd44f1`

2026-09-30 준비 검사에서 최종367개 source bytes와 membership이 전부 일치했다. [CURRENTNESS.json](CURRENTNESS.json)을 참조하되 감사 시작·종료에 currentness를 다시 확인한다.
R06 경계는69개 파일이 current source와 다르다. 보존 경계 비교와 최종 R07 영향 수용표로 해당 차이를 추적하고 의미적 영향은 감사자가 판정해야 한다.

## 필요한 범위

1. **R06 전체**: `content_authoring_value_expressions_reassessment/successors/r06_scope_20260927_001/`의 정식 Goal/Handoff와 continuation 인계를 인수한다. GS-CAVER-SCOPE50-001 / GOAL-CAVER-01 / HA-CAVER-SCOPE60-001. 기존 C07 미검토 및 C07-OBS01 delivery settlement/C03-OBS01 MIN 공개 RON 교정, 31 acceptance·21 IVA, 수식·Snapshot/Live·실제 amount/trace·stat/SpellPower·fingerprint·저장을 포함한다. 부분 D04/D07 리뷰의 PASS를 전체로 승격하지 않는다.
2. **R07 전체**: `content_authoring_fragment_copies/PLAN_HANDOFF.ko.md`, GS-CAFCP-001 / GOAL-CAFCP-01 / HA-CAFCP-001. exact 사본 지급·장착/이전·개화/선택·분쇄/사망회수, 실제 battle 출처/자원, V8 restore·명시 V7 변환과 원본 보존. execution `COVERAGE.json` F01~F12/T11·T18·T20·T27을 실제 입력/코드/로그로 확인한다.
3. **최종 통합 영향**: R07 execution `R06_IMPACT_COVERAGE.json`과 `evidence/VALIDATION_CURRENTNESS.json`, 두 세 기록·실패/교정 원자료, 실제 공개 RON/CLI/GameCore/새 process restore, PC36/R05, budget·clone·rollback·source membership·archive·정역 patch·격리 복원을 현재 통합 target에서 확인한다. 1581+1581은 구현자 보고이므로 숫자만 신뢰하지 않는다.
4. R01/R04/R05 필요한 선행 verdict/target의 current 적용성을 조건별로 확인한다. 감사에서 새 변경이나 결함이 발견되면 수정 없이 owner와 영향 조건을 보고한다. 자체 회귀나 유효한 과거 결과 재사용은 감사의 의미 판단과 구별한다.

두 기존 Single Goal의 scope/verdict/condition mapping을 각각 보존한다. 통합 source가 하나라는 이유로 가상의 Master나 GEX-30을 만들지 않는다.
GEX-20 fresh read-only pass로 수행하고 필요한 reviewer 독립성을 확인한다. source를 수정하거나 R08~R12 구현을 감사와 동시에 시작하지 않는다.
동결 R06/R07 디렉터리에 새 로그를 쓰지 않고 새 review 경로를 사용한다. R05 cap9,030,188 단독 초과만 합의된 경고이며 다른 hard guard는 유지한다.20분 시험은 제외한다.

## 재개에 반환할 결과

각 Goal의 exact 감사 문서 경로, verdict, 감사 target identity, 조건별 근거, 미해결 finding/owner, 종료 시 currentness와 R06→R07 영향 판정이 필요하다.
필수 선행 조건의 current 긍정 감사가 충족되면 R08~R12 정식 P2G 작성·검증→GEX-00/10을 재개할 수 있다.
finding이 있으면 그 correction/재감사 조건을 먼저 닫는다. 이번 준비자는 완료 또는 PASS를 추정하지 않는다.
