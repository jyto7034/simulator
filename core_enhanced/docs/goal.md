docs/post_policy_code_repair_master_goal.md를 기준으로, 최근 두 audit goal인
post_policy_test_verification과 post_policy_technical_debt_review의 결과를 순서대로 실제 코드 수정으
로 연결하라.

먼저 docs/post_policy_code_repair_master_goal.md를 읽고, goal 시작 시 docs/goals/
post_policy_code_repair_master/PLAN.md, EXPERIMENTS.md, EXPERIMENT_NOTES.md, REPORT.md를 만들고 계속
갱신하라.

반드시 Phase 1을 먼저 수행하라. Phase 1이 green이 되기 전에는 Phase 2의 큰 구조 리팩토링으로 넘어가지
마라.

Phase 1 목표:

1. cargo test -p game_core 실패를 해결하라.
   - 실패 테스트:
     game::events::combat::tests::authored_protect_unit_tactical_plan_overrides_default_defense_contrac
     t
   - strict battlefield route validation을 약화하지 마라.
   - route id fixture/data authoring contract를 장기적으로 맞게 고쳐라.
2. cargo test -p game_server -- --list test cfg compile 실패를 해결하라.
   - AbnormalityMetadata.mobility_kind
   - AbnormalityMetadata.target_traits
   - LiveBattleDeploymentDto.unit_deploy_costs
   - 단순히 빈 필드만 채우지 말고 Unity-facing payload 직렬화 assert를 보강하라.
3. 0-test filter 문제를 정리하라.
   - integration target은 cargo test -p game_core --test <target> 형태로 실행/문서화하라.
   - tests/unit_test.rs가 0-test target으로 남지 않게 제거하거나 의미 있는 테스트로 채워라.
4. focused coverage gap을 보강하라.
   - attempt/retreat edge cases
   - server Unity DTO coverage
   - threat warning false-rumor lifecycle

Phase 2 목표:

1. server BehaviorResult payload mapping을 typed DTO 방향으로 개선하라.
2. CombatPreview generated warning validation을 GameDataBase::new 밖의 명시 validation/audit path로
   분리하라.
3. projected effective profile helper 중복을 runtime/snapshot helper와 통합하라.
4. dead anti-overlap steering policy branch를 제거하거나 live path 밖으로 분리하라.
5. false threat rumor 정책을 구현/정리하라.
   - 20% 확률
   - 최대 1개
   - 실제 resolved spawn wave warning에 없는 후보에서만 선택
   - retreat/re-entry 시 false rumor는 Disproved
   - Observed 관측 시스템은 구현하지 말고 active contract에서 제거하라.
6. rumor candidate는 현재 실제 판정 가능한 warning으로 제한하라.
7. generated wave enemy briefing count는 resolved spawn wave 기준으로 계산하라.
8. stale resource file은 외부 참조 audit 후 가능한 삭제 방향으로 정리하라. 삭제 직전에는 사용자 허락
   을 받아라.
9. snapshot effective profile 실패는 숨기지 말고 effective_profile_error 형태로 surface하라. 추천
   shape는 { code: string, message: string }다.

Source of truth 우선순위:

1. 실제 runtime code
2. live RON/data
3. Unity-facing snapshot/command 계약
4. 최신 정책 문서

Unity-facing canonical 문서는 저장소 내부 copy가 아니라 다음 외부 문서를 우선한다.

- /mnt/f/unity projects/ark/docs/unity_core_contract.md
- /mnt/f/unity projects/ark/docs/unity_client_implementation_goal.md

코드 수정은 장기적인 방향으로 하라. 임시방편, 최소 수정, 특정 테스트만 통과시키는 패치를 피하라.
문서를 무조건 신뢰하지 말고 실제 코드를 읽으면서 더 나은 개선안이 있으면 EXPERIMENT_NOTES.md에 근거를
기록하고 적용하라.
레거시는 과감하게 제거하라. compatibility layer, fallback path, dual schema는 기본적으로 만들지 마라.
테스트가 예전 정책을 고정하고 있으면 기대값만 바꾸지 말고 삭제하거나 최신 정책 테스트로 교체하라.

git restore, git reset 등 git file modifier 명령은 수행하지 마라.
git 과거 내역의 코드를 현재 파일에 cp하지 마라.
파일을 과거로 돌려야 한다면 이유를 설명하고 사용자 허락을 먼저 받아라.
worktree가 dirty여도 사용자 변경으로 간주하고 되돌리지 마라.

사용자 결정이 필요한 정책이 발견되면 임의로 결정하지 말고 goal을 종료하고 질문 목록을 보고하라.
특히 외부 resource file 삭제는 사용자 허락 없이 진행하지 마라.

검증 시 모든 cargo test 명령의 실제 test count를 기록하라. 0개 테스트 실행 filter를 성공으로 보지 마
라.

최종 완료 전 반드시 가능한 범위에서 다음을 실행하고 결과/test count를 REPORT.md에 기록하라:

- cargo fmt
- cargo check -p game_core
- cargo check -p game_server
- cargo test -p game_core --test ron_loading
- cargo test -p game_core --test skill_refactor_validation
- cargo test -p game_core --test skill_test_suite
- cargo test -p game_core --test live_item_skill_activation
- cargo test -p game_core --test live_skill_catalog_audit
- cargo test -p game_core
- cargo test -p game_server -- --list
- 가능한 경우 focused game_server tests

완료 시 변경 요약, 제거한 레거시, 새로 고정한 계약, 갱신한 테스트, 남은 위험, 실행한 검증 명령과 test
count를 한국어로 보고하라.
