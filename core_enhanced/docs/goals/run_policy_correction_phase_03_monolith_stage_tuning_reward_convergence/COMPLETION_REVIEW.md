# Monolith Stage Tuning And Reward Convergence Completion Review

Review date: 2026-07-30

Verdict: Complete

Confidence: High

## Scope And Independence

P03 exit gate를 current source, serialized boundaries, tests와 Unity presentation에서 다시 추적했다.
계획 문서의 구현 주장만으로 판정하지 않았으며 focused와 broad suites를 현 작업 상태에서 재실행했다.

## Requirement Evidence

1. Typed authority and persistence — Complete
   - `MapNodeCombatDifficulty`는 required `monolith_stage: Option<Stage2|Stage3>`를 직렬화한다.
   - role/stage/ledger 불일치는 live authority 및 checkpoint continue에서 거부된다.
   - checkpoint export, 정상 continue, checkpoint load, transaction rollback의 Stage2 왕복 회귀가 통과한다.
2. Tuning convergence — Complete
   - 한 resolver가 role+stage를 합성하고 target/HP/attack/defense/reward consumer가 이를 공유한다.
   - Stage2는 `220/165/145/145/200/250`, Stage3은 `275/207/182/182/200/313`이다.
   - preview 회귀가 Stage3 실제 stat 증가와 두 stage의 동일한 200% wave budget을 입증한다.
3. Reward rounding — Complete
   - original quantity에 composite multiplier를 한 번만 nearest/minimum-one 적용한다.
   - quantity 1, 313%가 3인 회귀가 통과한다.
4. Wire and Unity projection — Complete
   - Core DTO는 exact percent 없이 `enhanced`/`final_enhanced` tier만 직렬화한다.
   - Unity reader/presenter는 qualitative label만 표시하며 `%`가 없다.
   - legacy exact percent와 mixed new+legacy payload는 fail closed한다.
5. Regression safety — Complete
   - Rust monolith 17/17, live RON 23/23, workspace all-targets 전부 통과했다.
   - Unity focused 3/3, EditMode 260/260, PlayMode 62 passed/4 explicit live-server skips다.
   - broad run에서 발견한 event-started combat retry preview 회귀는 cached authority 선사용으로 보정했고
     관련 기존 테스트 2/2 및 workspace 재실행이 통과했다.

## Remaining Risk

PlayMode의 4개 explicit test는 외부 live game_server가 필요한 기존 opt-in 항목이라 자동 suite에서 skip됐다.
변경된 reader/presenter 및 offline runtime 경로는 focused/EditMode/PlayMode 전체 비-live suite로 검증됐다.
정책·schema·compatibility fallback에 알려진 미해결 항목은 없다.

## Final Assessment

P03 요구사항과 사용자 가시 exit gate가 모두 충족됐다. 판정은 `Complete / High`다.
