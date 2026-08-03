# Run Persistence, Facility, And Monolith Master Experiment Notes

Status: complete planning notes

Implementation complete: true

Created: 2026-07-27

이 문서는 구조적 판단, source-of-truth 충돌, 정책 질문, dirty worktree 위험과 범위 밖 후보를
기록한다. Gameplay/data/save/wire의 durable source of truth가 아니다.

## 1. Master 구조를 선택한 이유

다섯 decision을 하나의 단일 구현 Goal로 다루면 source-of-truth 경계와 검증 표면이 지나치게 커진다.
반대로 문서를 각각 완전히 닫은 뒤 다음 문서로 이동하려 하면 다음 순환 seam 때문에 앞 문서를
계속 다시 열게 된다.

- SavePoint active-run payload는 이후 facility pool/assignment, power anchor/difficulty와 monolith
  ledger를 보존해야 한다.
- Local compendium storage/interface는 monolith보다 먼저 필요하지만 hidden boss와 세부 defeated
  producer는 monolith phase가 소유한다.
- Facility decision은 generic Shop guarantee/capstone을 소유하지만 monolith eligibility/trigger는
  monolith decision이 소유한다.
- Facility Floor generation transaction은 difficulty profile 입력 자리를 소유하고, power phase가
  실제 anchor/band/target 계산을 소유한다.

따라서 master는 phase 순서와 seam을 소유하고, child는 하나의 밀접한 계약 경계를 수직으로
완료한다. 실제 hidden boss는 현재 master 밖의 후반 콘텐츠 Goal로 분리하고 마지막 P07은 P00-P05
child의 완료 주장을 독립적으로 재검증한다.

## 2. 기존 단일 Goal 지침에서 조정한 점

사용자가 제공한 단일 Goal 지침의 dirty-worktree, baseline, 정책 결정, compatibility, evidence와
완료 원칙은 유지한다. 다음 부분만 master campaign에 맞게 조정했다.

- 여러 독립 경계가 있다는 이유로 `scope_decision_required`에서 멈추지 않는다. 사용자가 명시적으로
  통합 master와 phase 분리를 요청했으므로 분리 구조 자체는 승인된 실행 입력이다.
- Master는 production 구현 범위가 아니라 registry/dependency/review 범위를 가진다.
- 각 phase가 단일 Goal 규칙을 적용하는 child Goal이 된다.
- Master 생성 시 모든 child 문서를 만들지 않고 활성화 시점에만 만든다.
- 새 정책 결정 보고서를 구현 완료로 처리하지 않는다. `policy_decision_required` 상태로 사용자
  결정을 받은 뒤 같은 child를 재개한다.
- 후속 phase가 predecessor seam을 확장하면 predecessor 완료 evidence와의 회귀를 다시 검토한다.

## 3. 확정된 현재 중단 상태

가장 강하게 입증된 중단 작업은 `Support/Rest -> SavePoint` vertical migration이다.

- Core enum/payload/session/runtime 일부는 새 SavePoint로 이동했다.
- live map generation policy는 삭제된 Support를 참조한다.
- server production build는 되지만 test compile은 삭제된 Support API에서 실패한다.
- Unity runtime, contract, mock과 tests는 여전히 SupportChoice/ChooseSupport/Rest를 사용한다.
- Core의 새 SavePoint recovery도 최신 decision의 tagged recovery와 psychology 제거까지 도달하지
  않았다.

이를 일반적인 unrelated baseline failure로 남기면 후속 persistence/failure 작업이 어느 계약에서
시작됐는지 증명할 수 없다. 다만 flat SavePoint heal과 percent Gate recovery가 섞인 현재 중간 상태를
P00의 public/canonical 기준선으로 한 번 완성하면 P01에서 같은 계약을 다시 migration하게 된다.
따라서 P00은 단순 correction baseline이 아니라 최종 tagged `TraumaRecovery`, psychology producer
제거와 Support/Rest vertical cleanup까지 소유한다. P01은 그 final recovery 계약을 교체하지 않고
failure/checkpoint/active-run을 확장한다.

## 4. 부분 기반과 중단 작업의 구분

모든 미완성 기능을 같은 종류의 중단 작업으로 취급하지 않는다.

- `map_template_id`, `slot_id`와 Unity placeholder catalog는 과거 facility DTO foundation goal의
  결과다. 이는 P03의 재사용 가능한 부분 기반이지만 authored facility implementation 완료 증거는
  아니다.
- 현재 global encounter compendium과 server in-memory merge는 과거 goal의 부분 기반이다. P02의
  영속 profile target policy는 별도 신규 범위다.
- 현재 checkpoint clone은 기존 SavePoint goal의 기능이지만 durable continue와 새 failure policy는
  P01의 신규 범위다.
- 현재 floor multiplier/research/repeat weighting/BossOmen은 당시 정책의 구현이다. P04/P05에서
  새 승인 정책과 충돌하는 범위를 migration하되 과거 goal 문서를 현재 policy처럼 고치지 않는다.

## 5. Hard Dependency와 실행상 순서

### Hard dependency

- P00 final SavePoint/Gate recovery contract -> P01 failure/checkpoint/active-run extension
- P01 checkpoint/host persistence -> P02 local profile persistence primitive reuse
- P01 checkpoint/active-run envelope -> P03 facility state round-trip
- P03 facility Floor generation transaction -> P04 anchor/band/difficulty integration
- P01 checkpoint/active-run + P02 compendium + P03 facility + P04 difficulty -> P05 monolith
- P00-P05 completion -> P07 final review
- P05가 만든 dormant corrupted key -> 현재 master 밖의 미래 hidden-boss 콘텐츠 Goal이 명시적 재개
  뒤 소비

### 실행상 선택

- P02는 P01 뒤에 둔다. Storage low-level 지식을 재사용하되 active-run/profile transaction 분리를
  즉시 검증할 수 있다.
- P03은 기본적으로 P02 뒤에 직렬로 둔다. 다만 P02와 P03은 서로의 hard dependency가 아니며 둘 다
  P01만 닫히면 시작할 수 있다. 현재 worktree가 Core/server/Unity 전반에서 매우 dirty하므로 동시에
  public contract를 바꾸지 않는 선호 실행 순서다. P02가 외부 persistence/UI 문제로 막혀도 P03의
  필수 template/asset 입력이 준비됐다면 P03까지 자동 hard-block하지 않는다.
- Unity migration은 별도 마지막 phase로 미루지 않는다. 각 producer 변경과 같은 phase에서 reader,
  store, command, presentation, mock/test와 external canonical을 함께 완료한다.

P03은 P04가 아직 저작하지 않은 `difficulty_profile_id`를 live assignment RON에 먼저 남기지 않는다.
P03은 Floor generation의 typed extension seam까지만 제공하고, difficulty reference field와 strict
reference validation은 실제 profile data와 함께 P04가 추가한다. P04 completion review는 이 변경으로
materially 확장된 P03 determinism/assignment/save contract를 함께 재실행한다.

## 6. Source-Of-Truth 충돌

현재 다음 세 층을 구분해야 한다.

1. 실제 동작: 부분 전환된 Core와 아직 남은 live RON/server/Unity legacy
2. 과거 canonical/goal: Support 선택, procedural map, in-memory compendium, research-point completion
3. 승인된 목표: 다섯 2026-07-25/26 decision record

현재 실제 동작을 목표 정책으로 승격하지 않는다. 반대로 decision을 구현된 사실처럼 표현하지
않는다. 각 child는 migration 전 현재 behavior와 migration 후 expected behavior를 테스트에서 나눠
증명해야 한다.

## 7. Dirty Worktree 주의사항

- `/mnt/f/work/simulator`와 외부 `/mnt/f/unity projects/ark` 모두 대규모 tracked/untracked 변경이
  있다.
- `core_enhanced/docs/decisions/` 자체가 현재 untracked다. 이 campaign은 해당 내용을 승인 입력으로
  읽되 사용자 변경을 재작성하거나 이동하지 않는다.
- P00 대상인 Core map/support, run policy, live map RON, game_server message test와 Unity non-combat
  파일도 이미 modified 상태다.
- Child는 production 수정 전에 대상 파일별 `git diff`를 읽고 기존 변경의 의미를 보존해야 한다.
- 저장소 전체 format, reset, restore, clean, stash나 unrelated snapshot update를 하지 않는다.
- 기존 변경과 안전하게 병합할 수 없으면 해당 child를 완료하지 않고 충돌 파일과 필요한 사용자
  선택을 보고한다.

## 8. Save/Compatibility 미결정 사항

Decision은 active-run/profile 파일을 versioned envelope로 만들도록 승인하지만, campaign의 중간
phase build끼리 deployed file compatibility를 얼마나 유지할지는 승인하지 않았다.

기본 실행 가정:

- campaign 중간 build가 별도로 배포되지 않는다면 각 phase가 최종 target schema로 전진하고 최종
  P07에서 current schema를 검증한다.
- intermediate deployment나 기존 파일 유지 요구가 생기면 migration/compatibility는 새 정책
  질문이다.
- 승인 전에는 missing field default, alias, dual reader, silent reset을 추가하지 않는다.

## 9. 콘텐츠·밸런스 미결정 위험

P03/P04/P05와 미래 hidden-boss Goal은 decision에 명시된 구조와 의미를 구현하되 실제 live authoring에
필요한 값이나 content asset이 모두 존재하는지 phase 시작 전에 다시 확인해야 한다.

다음 값이 decision/canonical/live data 어디에도 확정돼 있지 않고 사용자-visible 결과를 바꾼다면
임의로 만들지 않는다.

- Standard Floor 1~3와 Endless 초기 facility template의 room/connection/tag, 조건부 발현 대표
  template, BossOmen/hidden-boss capstone 및 Unity-local slot/reveal resource
- combat power provisional coefficient와 version별 formula 세부값
- Floor별 minimum anchor, minimum rise, band quota/modifier와 role profile 값
- monolith price/offer/refresh/reward/VN resource와 encounter composition의 미작성 값
- final boss reclassification 또는 hidden boss resource가 요구하는 미제공 asset

필수 값이 없으면 child를 `policy_decision_required` 또는 asset 접근 문제라면
`externally_blocked`로 두고 evidence와 선택지를 보고한다.

P02는 storage/interface만 만들고 실제 `defeated` producer를 P05까지 미루지 않는다. 기존 대표
encounter의 primary victory와 non-primary `UnitDied` identity가 P02에서 실제 profile commit까지
도달해야 한다. P05는 remanifestation, 미래 콘텐츠는 hidden-boss identity만 이 공통 API에 연결하고 producer
의미를 재정의하지 않는다. 또한 local profile과 active-run은 transport `player_id`별 actor map이
아니라 decision이 승인한 local-install scope를 사용해야 한다.

## 10. 접근 가능한 외부 소비자

현재 workspace에서 다음 외부 Unity 저장소를 읽을 수 있다.

- runtime reader/store/router/command/presentation/view
- EditMode/PlayMode tests와 mocks
- `docs/unity_core_contract.md`
- `docs/core_unity_battle_transport_contract.md`
- probe scripts와 contracts

실제 Unity Editor runner와 target OS별 filesystem behavior는 phase 실행 환경에서 다시 확인한다.
파일을 읽을 수 있다는 사실을 EditMode/PlayMode test나 Windows/macOS/Linux atomic replace를 실제로
검증한 것으로 표현하지 않는다.

## 11. 적용하지 않은 대안

### 다섯 문서를 하나의 giant child Goal로 구현

적용하지 않는다. Persistence, map generation, difficulty와 content progression의 source-of-truth와
검증 실패 범위가 달라 completion evidence가 흐려진다.

### 각 decision을 완전히 닫고 다음 문서 시작

적용하지 않는다. Facility의 monolith capstone, compendium defeated producer와 checkpoint의 후속
state 확장 때문에 predecessor를 숨은 방식으로 재개하게 된다. Generic seam과 content consumer를
명시적으로 나눈다.

### Local compendium과 active-run을 하나의 save file/transaction으로 통합

적용하지 않는다. 두 decision이 rollback lifetime과 failure semantics를 명시적으로 분리한다.

### Unity migration을 마지막에 한 번 수행

적용하지 않는다. Phase 사이에 production producer와 accessible consumer가 다른 schema를 사용하게
되고 phase completion을 거짓으로 만든다.

### 현재 모든 dirty failure를 P00에서 정리

적용하지 않는다. P00은 중단된 SavePoint vertical boundary만 소유하며 unrelated battle, employee,
skill, artifact, debug output와 문서 cleanup은 사용자 변경으로 보존한다.

## 12. 범위 밖 후속 후보

현재 없음으로 확정하지 않는다. Child 조사에서 발견한 독립 문제는 각 child notes에 기록한다.
Master 범위에 직접 필요한 dependency correction이 아니라면 새 Goal을 자동 생성하거나 구현하지
않는다.

현재 worktree의 광범위한 unrelated 변경, 삭제된 debug artifacts와 과거 docs 정리는 이 master의
후속 후보로조차 자동 등록하지 않는다. 별도 사용자 요청이나 기존 owner Goal에서 다룬다.

## 13. 남은 위험

- 현재 worktree가 매우 커서 phase 시작 전 target diff가 추가로 바뀔 수 있다.
- P01 저장 substrate가 host 플랫폼별 atomicity를 실제로 제공하는지 구현 조사 전에는 확정할 수 없다.
- External Unity test runner와 실제 persistent-data path를 실행할 수 없으면 해당 phase complete가
  차단될 수 있다.
- P04/P05 content authoring 양이 예상보다 크면 정책 구현과 콘텐츠 제작을 별도 child로 나눠야 할 수
  있다. 분리가 사용자-visible completion을 바꾸면 scope decision을 요청한다.
- P05에서 battle event identity가 defeated semantics를 충분히 표현하지 못하면 battle transport/event
  schema 변경이 필요할 수 있다. 승인된 semantic을 구현하기 위한 typed shape는 child에서 조사하되
  새로운 승패/죽음 의미는 임의로 결정하지 않는다.
- P01과 P05는 단일 patch로 수행하기에 넓다. 각 child는 master PLAN의 내부 milestone대로 focused
  검증 단위를 나누되 phase 전체 결합 review 전에는 complete로 닫지 않는다.

## 14. P01 전달 seam

- P01 completion review는 `Complete / High`다.
- P02는 `ActiveRunStore`의 파일 lifecycle 기법을 참고할 수 있지만 active-run과 profile의 파일,
  envelope, lock, transaction과 failure surface를 결합하지 않는다.
- Core active-run payload에는 local compendium copy가 없고 continue/import는 현재 local profile을
  보존한다.
- load-balance의 기존 process-local per-player compendium map은 P02에서 durable authority로 제거해야
  한다. transport player UUID별 분리는 승인된 local-install profile scope가 아니다.

## 15. P02 전달 seam

- P02 completion review는 `Complete / High`다.
- local compendium은 active-run/checkpoint 밖 local-install singleton이며 P03 map state에 복사하지 않는다.
- dormant-preserving persisted 검증과 current-catalog 신규 commit 검증이 분리돼 있다.
- authored full projection과 generic actual-result commit seam은 P05가 신규 encounter identity를 연결할
  수 있지만 facility/difficulty phase가 profile 저장 의미를 확장할 이유는 없다.
- profile store와 active-run store는 저수준 filesystem primitive만 공유하며 canonical file, lock,
  transaction, bootstrap와 retry state는 분리돼 있다.

## 16. P05 readiness blocker

- P03/P04까지의 generic technical seam은 Complete / High이며 접근 가능하다.
- P05 audit에서 전용 hidden-boss production actor/skill/encounter/capstone 및 exchange/administrator VN
  content가 없음을 확인했다.
- `abnormality_skill_design_notes.ko.md`의 산탄총·아군 관통 seed는 실제 조우를 `추후 미정`으로 남겨
  player-visible mechanics/counterplay를 승인하지 않는다.
- 따라서 이는 asset 접근 실패가 아니라 content 의미 입력 부재이며 `externally_blocked`가 아닌
  `policy_decision_required`다. 사용자 입력 전 P05 production migration을 시작하지 않는다.
- 2026-07-27 사용자는 미정 콘텐츠를 작업자가 채우지 않고 전체 콘텐츠 정책이 확정될 때까지 P05를
  구현하지 않으며, 특히 hidden-boss 세부 구현을 skip하도록 확정했다.

## 17. 사용자 승인 후 phase split

- 이후 사용자는 `2번`, 즉 모노리스 시스템 전체 구현과 실제 보스 구현의 후속 연기를 선택했다.
- 기존 blocker는 P05 전체 blocker가 아니라 hidden-boss content를 소유하는 P06 activation gate로 이동한다.
- P05는 natural Elite, merchant/typed monolith, node conversion, stage 2/3, residual exchange와 corrupted
  key까지 구현하면 독립적으로 완료 가능하다.
- P05의 corrupted key에는 consumer가 없고 기존 final-boss/Endless completion은 유지한다. P06은 콘텐츠
  정책 확정과 명시적 재개 뒤에만 이를 소비한다.
- 과거 hidden-boss mechanic 제안은 historical note로만 보존하며 P05 구현 authority가 아니다.

## 18. P06을 현재 master에서 분리한 이유

- 실제 hidden boss는 actor/skill/encounter/battlefield/wave/reward/VN/Unity resource처럼 콘텐츠 설계와
  제작이 함께 필요한 수직 범위다. 시스템 substrate를 검증하는 현재 master의 완료 gate로 계속 두면
  이미 닫힌 P00-P05의 통합 감사가 콘텐츠 일정에 종속된다.
- P05 terminal은 dormant corrupted-monolith key이며 consumer가 없는 상태를 의도적으로 검증했다.
  기존 FinalBoss classification과 Endless completion을 가로채지 않으므로 현재 시스템 의미가
  불완전하거나 숨은 placeholder로 남은 것은 아니다.
- 따라서 현재 master는 P00-P05와 그 cross-phase seam을 P07에서 닫는다. 후반 콘텐츠 Goal은 P02의
  compendium commit, P03의 generic capstone/manifestation, P04의 role difficulty와 P05의 key를 소비할
  수 있지만 현재 master의 완료 상태를 소급 변경하지 않는다.
- README 표식은 향후 콘텐츠 개발자가 Core/live RON/server/Unity/persistence/canonical을 한 작업에서
  함께 갱신하도록 하는 일정·범위 기억장치다. 콘텐츠 정책 확정과 명시적 재개 전에는 Goal이나
  production identifier를 만들지 않는다.

## 19. Master 종료 메모

- P07이 P00~P05를 `Complete / High`로 독립 판정했고 발견된 세 계약 drift를 같은 범위에서 수정했다.
- 최종 broad Rust/Unity와 두 live WebSocket probe가 모두 실패 없이 완료됐다.
- dormant corrupted-monolith key와 기존 completion 유지가 현재 master의 의도된 종점이다.
- 다음 콘텐츠 Goal은 이 master를 재개하지 않고 README의 후반 hidden-boss 표식을 입력으로 새로 만든다.
