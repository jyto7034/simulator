# Policy Decision Log

이 ledger는 current runtime 사실이나 이미 승인된 gameplay 의미를 다시 묻지 않는다. Production 구현을
시작하면 사용자 progression/content scope가 실제로 달라지는 미결정만 분리한다. 최초 감사에서 만든
7개 최초 후보와 최종 정책 검토에서 생긴 3개 추가 결정을 durable canonical과 live identity에 대조해 `closed_by_canonical`,
`closed_by_audit`, `closed_by_user` 또는 `content_input_required`로 분류한다. 권장안만으로 open content
input을 승인으로 간주하지 않으며, 2026-08-12 사용자 답변과 최종 검토로 모든 input을 닫았다.

## Content Authority Boundary

`GRB-PD-004/005/006`의 `closed_by_user`는 최종 콘텐츠 승인 상태가 아니다. 이 decision들은 runtime/schema를
구현하고 검증할 때 필요한 **minimum bootstrap fixture**를 선택했음을 뜻한다. 다음은 별도 content
Goal의 authority로 남는다.

- 최종 encounter roster, source→fragment pairing과 source별 repeat balance
- 최종 Standard topology/slot/encounter/tuning/reward/script/narrative/assets
- dedicated fragment skills와 최종 BossOmen live chains. Current one-step WhiteNight는 개발 검증용
  bootstrap 예외다.

후속 content Goal은 explicit source authoring, first-or-repeat 단일 transaction, ordinary Reward 배제,
fixed three-Floor Standard/no adaptive difficulty, F3 completion+profile atomicity 같은 policy invariant를
바꾸지 않는 한 bootstrap 값을 교체할 수 있다. 따라서 bootstrap 교체만으로 이 policy decision을 다시
열거나 rulebook correction을 요구하지 않는다.

## Decision Summary

| Decision | Status | Connected findings | Blocks | Recommended option |
| --- | --- | --- | --- | --- |
| `GRB-PD-001` | closed_by_canonical | M002, M005, M008, M010–M014 | none | A — strict new-run cutover |
| `GRB-PD-002` | closed_by_canonical | M001, M004 | none | A — preserve compendium V1 in unified successor |
| `GRB-PD-003` | closed_by_audit | M005, M007, M008, M010, M012 | none | A — explicit authored source identity |
| `GRB-PD-004` | closed_by_user | M002, M007–M010, M012 | none | A/R1-bootstrap — runnable 29-row fixture + explicit 5/10/20 dust |
| `GRB-PD-005` | closed_by_user | M011, M012 | none | A-bootstrap — one-step WhiteNight live fixture; final chain content deferred |
| `GRB-PD-006` | closed_by_user | M001 | none | A/ST1-NT60-bootstrap — runnable fixed tutorial fixture |
| `GRB-PD-007` | closed_by_canonical | M015 | none; future campaign only | A — separate deferred content Master |
| `GRB-PD-008` | closed_by_user | M005 | none | unmastered-first, mastered fills otherwise-empty Elite slots |
| `GRB-PD-009` | closed_by_user | M006 | none | initial 150%/200%, development-tunable RON values |
| `GRB-PD-010` | closed_by_user | M007, M008 | none | mastered-sweep dismantle/reacquire cycle |

## GRB-PD-001 — Existing Active-Run V3 Cutover

- **Question/blocker:** Encounter/research authority를 V4로 바꿀 때 local-install의 미완료 V3 run을
  어떻게 처리할 것인가. `GRB-WP-001`의 save schema, loader, rollout과 migration tests 전체를 막는다.
- **Current/canonical:** V3 payload는 strict `deny_unknown_fields`이며 research/pending, old Monolith state를
  저장한다. `data_loading_contract.md`는 active-run envelope/Core payload를 exact V3로 제한하고
  `core_runtime_contract.md`는 V2 header 즉시 거부, V2/V3 dual reader·migration 부재를 명시한다. 룰북도
  불일치 문서를 state/export 변경 없이 거부하고 repair/rebinding/구 schema migration을 금지한다.
- **Already decided:** research는 제거하고 두 authority를 병행하지 않는다. New run/checkpoint는 하나의
  encounter ledger를 사용하고 rollback은 전체 ledger/economy에 원자 적용한다.
- **Option A — strict new-run cutover (recommended):** V3 continue를 명시적 incompatible-version으로
  거부하고 사용자가 reset/new run을 선택한다. Roster/economy를 포함한 미완료 run progression은 잃지만
  research point, past boss assignment와 Elite victory를 임의 의미로 재해석하지 않는다.
- **Option B — one-shot validated transform:** V3의 roster/inventory/equipment/owned fragment는 보존하고
  research/pending은 drop하며, 표현 가능한 natural Elite/Monolith state만 documented mapping으로 V4에
  옮긴다. NormalBoss/FinalBoss cycle은 새 cycle로 시작한다. 일부 old progress가 의미를 잃고 복잡한
  corruption/backup/rollback fixture가 필요하다.
- **Impact/difficulty:** A는 package difficulty 5를 유지하고 migration surface를 제한한다. B도 5이나
  C/F 축의 migration corpus와 data-loss/partial transform risk가 더 크다. 양쪽 모두 Core/store/server
  restart와 Unity incompatible-save UX가 필요하다.
- **Recommendation evidence/trade-off:** Current V3에는 target first/repeat/cycle source를 정확히 복원할
  history가 없어 lossless migration이 불가능하다. Correctness 우선으로 A를 권장하지만 unfinished-run
  loss를 사용자가 명시적으로 수용해야 한다.
- **Resolution/status:** Option A. Existing durable canonical이 strict incompatible-version rejection을 이미
  결정하므로 one-shot semantic transform은 새 정책 변경 없이는 추가하지 않는다. V4 cutover fixture는
  V3 rejection, original preservation과 explicit reset/new-run UX를 검증한다. Status
  `closed_by_canonical`.

## GRB-PD-002 — Global Profile V1 Preservation

- **Question/blocker:** compendium-only V1 store를 tutorial flag/records가 포함된 profile로 바꿀 때 기존
  discovered/defeated sets를 보존할 것인가. `GRB-WP-003` durable schema와 reset/recovery UX를 막는다.
- **Current/canonical:** V1 current/backup envelope는 four compendium sets만 strict 저장한다. Local
  compendium policy는 committed set을 title의 confirmed full reset 외에는 삭제하지 않고 dormant ID를
  보존하며 ID rename은 explicit migration table로 처리한다고 확정한다. 최신 룰북도 두 set과 새 profile
  record를 local-install monotonic lifetime으로 두고 confirmed full profile reset만 삭제 authority로 둔다.
- **Already decided:** global profile은 active run과 분리된 local-install monotonic lifetime이며 title의
  confirmed full reset만 삭제한다. New tutorial/record를 past runs에서 소급 추론하지 않는다.
- **Option A — V1→V2 preservation (recommended):** four sets를 그대로 옮기고 tutorial flag=false,
  numeric records=0, all-boss=false로 초기화한다. Existing discovery는 보존되고 Endless는 Standard를 새로
  완료해야 해금된다.
- **Option B — strict profile reset:** V1 current/backup을 incompatible로 처리하고 full profile reset 뒤
  모든 set/flag/record를 empty로 시작한다. 구현은 약간 단순하지만 기존 도감 진행을 잃는다.
- **Impact/difficulty:** A는 explicit version migration/current+backup fixture가 추가되나 package는 이미
  multi-lifetime public cutover라 difficulty 5 유지다. B도 recovery UX와 data-loss confirmation 때문에 5다.
  Core/server/Unity snapshot 의미는 migration 이후 동일하다.
- **Recommendation evidence/trade-off:** V1 sets는 target compendium과 의미가 동일해 무손실 mapping이
  가능하다. 새 flag/record만 non-retroactive default로 두는 A가 최소 surprise다.
- **Resolution/status:** Option A. Same-semantic discovered/defeated sets의 보존은 canonical lifetime의
  결과이며 reset은 migration convenience가 아니다. Successor publication은 bounded explicit V1→successor
  transform으로 four sets를 보존하고, 소급 근거가 없는 tutorial/records는 false/0으로 시작한다. 이는
  current reader에 implicit fallback이나 permanent dual-read를 추가하지 않는다. Status
  `closed_by_canonical`.

## GRB-PD-003 — Stable Encounter Reward And Cycle Identity

- **Question/blocker:** first/repeat dedupe와 Elite/NormalBoss cycle이 어떤 stable key를 사용할 것인가.
  Encounter variant가 같은 fragment source인지 달라지므로 사용자-visible reward 횟수와 pool 구성이 바뀐다.
- **Current/canonical:** PVE에는 encounter ID와 primary abnormality ID가 있고 BossOmen에는 chain ID가 있지만
  target reward source ID는 없다. 룰북은 지급 source가 exact fragment를 RON으로 가리키고, 같은 source의
  first/repeat ledger와 authored Elite/NormalBoss/FinalBoss identity/cycle을 요구하며 exact schema 이름은
  구현 계획에 맡긴다. Existing Monolith canonical은 abnormality-keyed ledger다.
- **Already decided:** Dismantle은 first grant를 reset하지 않고 source completion은 run-local/checkpoint다.
  FinalBoss cycle 구성원은 normal authored chain이며 hidden boss는 제외한다.
- **Option A — explicit authored source ID (recommended):** PVE/chain entry가 stable `reward_source_id`와
  cycle identity를 명시하고 여러 encounter variant는 의도할 때만 같은 ID를 공유한다. Loader가 class,
  fragment와 membership uniqueness를 검증한다.
- **Option B — primary abnormality ID:** Elite/NormalBoss는 primary abnormality로 자동 group하고 FinalBoss는
  boss abnormality로 group한다. Authoring은 단순하지만 non-abnormality/special source와 한 abnormality의
  독립 reward variants를 표현하기 어렵다.
- **Option C — encounter/chain ID:** 각 PVE encounter와 chain을 곧바로 source로 쓴다. Schema는 가장
  단순하지만 같은 boss의 wave/tuning variant마다 first grant가 반복될 수 있고 content refactor가
  progression identity를 바꾼다.
- **Impact/difficulty:** A는 E2 validation/authoring이지만 long-lived save key가 content layout과 분리된다.
  B/C는 당장 field 수를 줄여도 future migration/duplicate reward risk가 있다. 모든 옵션은 V4 save와
  result source projection, deterministic tests가 필요해 WP difficulty 5를 바꾸지 않는다.
- **Read-only reconciliation:** Live eligible encounter 30개는 encounter ID와 primary abnormality ID가 모두
  30/30 unique하며 `suppress_king_of_greed`도 올바른 `o-01-64_king_of_greed`를 가리킨다. 따라서 current
  collision을 고칠 필요는 없다. 그러나 rulebook은 한 abnormality의 multiple fragments와 future encounter
  variants를 허용하므로 automatic abnormality key는 독립 reward source를 표현할 수 없고, raw
  encounter/chain ID는 content layout rename에 lifetime을 결합한다.
- **Resolution/status:** Option A는 승인된 gameplay를 바꾸는 새 선택이 아니라 strict authoring/validation
  구현 방식이다. Eligible source마다 stable explicit key를 요구하고 variant sharing은 동일 key를 RON에
  의도적으로 반복 authoring한 경우만 허용한다. Bootstrap roster 29개 source는 one-to-one key로 시작하고
  excluded Moonlit Wail과 exact primary/fragment reference를 manifest validation gate로 분리한다.
  Bootstrap fragment/repeat 값은 PD004에 남는다. Status `closed_by_audit`.

## GRB-PD-004 — Live Encounter Fragment/Repeat Content Scope

- **Question/blocker:** Atomic research replacement을 빈 참조 없이 구동·검증하려면 어떤 bootstrap
  Elite/NormalBoss/FinalBoss source의 `fragment_id`와 `repeat_reward`를 publish할 것인가. Mapping 없이는
  loader를 strict하게 만들거나 old acquisition을 제거할 수 없다. Final content selection은 이 질문의
  범위가 아니다.
- **Current/canonical:** Live PVE는 Elite 13, NormalBoss 12, FinalBoss 5이고 target fields는 0건이다. Current
  content roster source는 Moonlit Wail을 boss 후보에서 제외하고 Queen of Hatred/Mountain of Smiling
  Bodies를 NormalBoss로, WhiteNight/Silent Orchestra/Apocalypse Bird만 FinalBoss로 둔다. 따라서 target
  encounter roster candidate는 Elite 13 + NormalBoss 13 + FinalBoss 3 = 29 identities다. Existing abnormality
  response-complete fragment IDs는 old research identity일 뿐 target repeat quantity/identity approval이
  아니다. 룰북은 값을 개별 RON authoring으로 명시적으로 위임했다.
- **Already decided:** eligible source는 first fragment 또는 repeat reward 하나를 result commit에서 직접
  지급하며 ordinary Reward node는 지급하지 않는다.
- **Option A — complete eligible bootstrap mappings before cutover (recommended):** initial runnable pool의 모든
  eligible source가 explicit ID/repeat effect를 갖고 loader가 omission을 reject한다.
- **Option B — defer encounter-economy cutover:** mappings가 준비될 때까지 `GRB-WP-001` 전체를 시작하지
  않는다. Current research semantics는 당분간 남고 다른 dependent package도 정지한다.
- **Impact/difficulty:** A는 bootstrap 29개 explicit rows와 Core data/load/result, Unity reward presentation QA를
  요구하는 one-time strict cutover다. B는 구현 변경이 없지만 audit target 미달이고 dependent packages도
  정지한다. Final content Goal은 활성 row 전부가 strict-valid라는 invariant를 지키면서 roster를 증감·교체할
  수 있다.
- **Available bounded candidate:** Existing abnormality catalog의 response-complete fragment는 target roster
  29개에 대해 one-to-one candidate를 제공한다. Moonlit Wail live source/fragment는 bootstrap pool에서
  제외하고 Queen/Mountain은 NormalBoss로 재분류해야 한다. Current encounter-primary reference는 30/30
  unique/correct하며 이 fact 자체는 새 reward approval이 아니다.
  `CONTENT_INPUT_CANDIDATES.md`가 exact 29-row bootstrap source→fragment table을 기록하므로 initial runnable
  fixture는 더 이상 unbounded input이 아니다. Final identity/roster selection은 의도적으로 다루지 않는다.
- **Remaining repeat options:** `R1-continuity`는 current run policy와 live primitive를 사용해 Elite 5,
  NormalBoss 10, FinalBoss 20 fragment dust를 각 row에 explicit authoring한다. `R2-none`은 모든 row의
  `repeat_reward: None`으로 base encounter reward만 남긴다. R1은 현재 값/반복 성장 연속성, R2는 추가
  economy 없음이라는 trade-off다.
- **Resolution/status:** 2026-08-12 사용자 답변 “나머진 너의 제안대로 진행”으로 Option A와
  `R1-continuity`를 minimum bootstrap으로 승인했다. Bootstrap 29 rows에 `repeat_reward`를 source별 explicit authoring하며 Elite는
  `GrantFragmentDust(5)`, NormalBoss는 `GrantFragmentDust(10)`, FinalBoss는
  `GrantFragmentDust(20)`을 사용한다. Class fallback이나 implicit substitute는 두지 않는다. Status
  `closed_by_user`. 이 값은 final roster/pairing/balance approval이 아니며 별도 content Goal에서 strict
  authoring invariant를 유지한 채 교체할 수 있다.

## GRB-PD-005 — Live BossOmen Chain Authoring

- **Question/blocker:** Current `white_night`를 포함한 live 일반 FinalBoss chain의 minimum Floor, optional
  required fragment/awakening, step/encounter와 encounter-driven fragment form을 무엇으로 승인할 것인가.
- **Current/canonical:** Live chain은 min Floor 3, one Event step, WhiteNight boss이며 runtime은 any-awakened
  gate다. 룰북은 백야 상세 체인을 future policy 후보라고도 명시하고 exact chain values를 후속 RON에
  위임하므로 current values를 target approval로 볼 수 없다.
- **Already decided:** per-chain condition, locked-cycle behavior, provisional/active failure, capstone/exit와
  FinalBoss reward/cycle semantics는 승인됐다. Common any-awakened fallback은 폐기한다.
- **Option A — explicit live chain contract now (recommended for current BossOmen scope):** 사용할 일반
  chain마다 required fragment/awakening, min Floor, steps, boss encounter와 form transitions를 승인해 strict
  RON으로 publish한다.
- **Option B — defer live BossOmen content:** generic approved chain/cycle/failure state machine은 WP001에서
  구현하되 production chain catalog는 empty/dormant로 두고 WhiteNight의 상세 content publication을 별도
  승인 Goal로 미룬다. Empty membership이 all-FinalBoss record를 자동 달성시키지 않도록 검증한다.
- **Impact/difficulty:** A는 `GRB-WP-001`에서 generic contract와 live content를 함께 닫고 difficulty 5다.
  B는 state machine을 placeholder로 만들지 않으면서 live FinalBoss content만 dormant 처리한다. WP003은
  typed record reducer를 구현할 수 있지만 non-empty live membership 전에는 all-FinalBoss achievement가
  dormant다.
- **Read-only reconciliation:** `game_rulebook.md`는 WhiteNight 상세 체인을 향후 재승인 후보이자 current
  implementation contract가 아니라고 명시한다. 완료된 placeholder canonicalization도 current one-step
  research-based shape를 temporary placeholder로 남겼다. BossOmen goal/hardening 기록은 new final-boss
  content를 scope 밖으로 두고 Silent Orchestra source encounters/events를 future authoring으로 명시한다.
  Apocalypse Bird도 concept skeleton 외 exact source/event/tuning/reward가 없다. 따라서 current one-step
  WhiteNight를 target으로 승격하거나 나머지 chain table을 audit가 발명할 authority가 없다.
- **Resolution/status:** 2026-08-12 최종 검토에서 Option A의 bootstrap variant로 수정했다. Current one-step
  `white_night`/`suppress_white_night`를 generic eligibility/cycle/source-failure/capstone과 profile publish를
  검증하는 유일 live chain으로 유지한다. 이 하나를 격파하면 그 시점 live membership 전체를 격파했으므로
  all-FinalBoss 영구 기록을 달성한다. 상세 WhiteNight/Silent Orchestra/Apocalypse Bird chain은 별도 content
  Goal이 소유하며 current chain을 최종 WhiteNight 설계로 승격하지 않는다. Status `closed_by_user`.

## GRB-PD-006 — Fixed Standard Tutorial Authoring

- **Question/blocker:** Fixed 3-Floor Standard의 exact templates/nodes/tuning과 optional scripted fragment를
  무엇으로 publish할 것인가. `GRB-WP-003`은 framework-only로 완료될 수 없다.
- **Current/canonical:** Current live Standard profiles are generated/adaptive and F3 FinalBoss다. Rulebook은
  F1 Normal/F2 Elite/F3 simplified NormalBoss와 fixed tuning을 결정했지만 exact layouts/encounters/script
  fragment ID/timing은 RON authoring에 맡겼다.
- **Already decided:** three fixed Floor, no adaptive power/FinalBoss, F3 victory completion/profile flag와
  Standard-only fragment lifetime.
- **Option A — complete fixed tutorial RON (recommended):** exact map/encounter/tuning and scripted fragment
  sequence를 승인한다. Script fragment를 사용하지 않기로 하면 그 부재도 명시적으로 author한다.
- **Option B — defer Standard/profile package:** content가 준비될 때까지 generated Standard와 ungated
  Endless current runtime을 유지하고 WP003을 시작하지 않는다. Target profile/Endless gate를 partial
  publish하지 않는다.
- **Impact/difficulty:** A는 M001 E3 및 WP003 difficulty 5를 유지하고 full map/script QA가 필요하다.
  B는 production change가 없지만 tutorial/profile target이 미달이다.
- **Available bounded candidate:** Existing `standard_intro_a`, `standard_branching_a`, `standard_final_a`는 각
  Standard pool의 유일한 template라 topology 재사용 후보는 명확하다. 그러나 slot content는 weighted
  assignment이고 final terminal은 `RunFinalBoss`; exact F1 Normal/F2 Elite/F3 NormalBoss encounter, fixed
  tuning과 scripted fragment는 target으로 authoring되지 않았다. `CONTENT_INPUT_CANDIDATES.md`의 `ST1`은
  세 topology의 모든 slot/content, neutral 100% fixed tuning, Standard-only reward override, F3
  `suppress_funeral_butterfly` completion과 `scripted_fragment: None`을 initial approval candidate로 묶었다.
  Current active fragment 둘은 Ranged Bow/Gun 호환인데 모든 starter가 Melee Sword이므로 fragment만 지급하는
  후보는 배제했다.
- **User modification and bootstrap authoring:** 2026-08-12 사용자는 최소 구동용 F3 boss를 “스탯이 감소한 아무것도
  없는”으로 바꾸고 나머지는 제안대로 진행하도록 승인했다. 이를 `ST1-NT60`으로 기록한다. F3
  `final_chamber`는 `suppress_nothing_there`/`o-06-20_nothing_there`이고 only that boss actor의 max HP,
  attack, defense와 magic resistance를 base 60%로 고정한다. Current base 1200/88/60/55의 integer target은
  720/53/36/33이며, initial Funeral candidate 680/58/30/40과 가까운 combat envelope라 “reduced”를
  과도한 ALEPH tutorial spike 없이 exact하게 만든다. 이 네 정수는 F3 fixed slot의 explicit boss-actor
  override이며 60%는 derivation rationale다. Current generic `BattleUnitStatScale`은 magic resistance가 없고
  encounter-wide이므로 global/runtime multiplier로 사용하지 않는다. 다른 Nothing There 출현과 ST1
  enemies/nodes/rewards는 100% fixed values를 유지하고 adaptive scaling을 사용하지 않는다.
- **Resolution/status:** Option A, modified `ST1-NT60`; scripted fragment `None`, F3 victory
  `StandardCompletion`과 permanent tutorial flag commit을 포함한다. Status `closed_by_user`.
  이는 runnable Standard/bootstrap fixture이며 final tutorial content approval이 아니다. 별도 content Goal은
  three-Floor fixed/no-adaptive와 terminal transaction invariant를 유지하면서 topology, encounter, tuning,
  reward, script와 presentation을 교체할 수 있다.

## GRB-PD-007 — 29 Dedicated Fragment Skill Campaign Scope

- **Question/blocker:** 29 temporary BasicAttackModifier를 이 implementation wave에서 모두 dedicated active
  skills로 author할지, 별도 content Master로 미룰지 결정한다.
- **Current/canonical:** ActiveSkill framework와 two live skills는 존재한다. 29개 individual mechanic,
  balance, compatibility와 Unity asset contract는 없고 룰북은 이번 correction에서 live conversion을 하지
  않는다고 명시한다.
- **Already decided:** fragment는 active player-selected skill이고 generic/no-op/shared buff conversion은
  완료가 아니다. 구체 design/art 제작은 이 audit scope 밖이다.
- **Option A — separate deferred content Master (recommended):** runtime capability audit 후 approved design
  batches를 independent vertical phases로 만들고 마지막 zero-placeholder audit로 닫는다. Current policy
  implementation wave는 WP001–004에 집중한다.
- **Option B — include campaign after core policy packages:** 29 design/implementation/asset batches를 같은
  roadmap의 `GRB-WP-005`로 활성화하되 각 batch 승인 전 production mutation을 시작하지 않는다.
- **Option C — supersede active-skill policy:** placeholders를 정식 passive/basic-attack modifiers로 유지하고
  룰북을 수정한다. 이는 2026-08-12 승인 정책을 뒤집는 새 gameplay 결정이다.
- **Impact/difficulty:** A는 현 wave에서 future/dormant이며 later package provisional 3–5. B는 현 roadmap의
  최종 Master로 3–5, large content/Unity QA를 추가한다. C는 implementation 1 수준의 문서 correction일 수
  있으나 사용자-visible combat policy를 바꾼다.
- **Resolution/status:** Option A. 룰북이 “이 correction에서는 해당 live data를 변환하지 않는다”고
  명시하므로 현 implementation wave에 campaign을 포함시키는 B는 scope 확대이고 C는 정책 변경이다.
  M015/WP005는 current readiness blocker가 아닌 `future_or_dormant`로 유지한다. 사용자가 별도 content
  Master를 명시적으로 시작할 때만 batch manifest/design gate를 새로 연다. Status
  `closed_by_canonical`.

## GRB-PD-008 — Elite Pool Mixed Fill

- **Question:** 미숙달 Elite가 하나라도 남아 있으면 모든 숙달 repeat 배정을 막을지, 현재 Floor의 고유
  미숙달 후보가 slot보다 부족할 때 숙달 후보로 남은 slot을 채울지 결정한다.
- **Resolution/status:** 미숙달 후보를 항상 먼저 배정한다. Same-Floor uniqueness를 적용한 서로 다른
  미숙달 후보가 Elite slot보다 적을 때만 남은 slot을 숙달 repeat pool로 채운다. 첫 숙달 identity부터
  repeat pool은 활성 상태를 유지하되 미숙달 우선순위를 침범하지 않는다. 양쪽 후보가 없을 때만 Normal
  Combat으로 바꾼다. Status `closed_by_user`.

## GRB-PD-009 — Monolith Development Tuning

- **Question:** Monolith Stage 2/3 stat multiplier의 current development baseline과 변경 authority를 정한다.
- **Resolution/status:** 초기값은 150%/200%다. 이는 hard-coded gameplay constant가 아니라 RON-owned
  development tuning이므로 playtest 중 조절할 수 있고, 값 변경만으로 policy decision을 reopen하지 않는다.
  Loader는 positive/ordered/safe 범위를 검증하고 현재 exact 숫자를 코드에 고정하지 않는다. Reward
  multiplier는 별도이며 현재 100%/125%를 유지한다. Status `closed_by_user`.

## GRB-PD-010 — Mastered Sweep Fragment Reacquisition

- **Question:** 지급 완료 뒤 마지막 fragment copy를 분쇄한 Elite source를 숙달 소탕했을 때
  `repeat_reward`만 지급할지 fragment를 재획득하게 할지 결정한다.
- **Resolution/status:** Source 지급 완료 ledger와 숙달은 유지한다. 소탕 commit 시 authored fragment의
  current count가 0이면 같은 fragment 한 개를 재지급하고 `repeat_reward`는 지급하지 않는다. Count가
  1 이상이면 fragment를 추가하지 않고 `repeat_reward`를 지급한다. 한 commit에서 둘은 상호 배타적이다.
  마지막 copy 분쇄→소탕 재획득 cycle은 횟수 제한 없이 허용하며, identity별 upgrade/awakening progress는
  copy count와 별도이므로 유지한다. 직접 전투, NormalBoss와 FinalBoss에는 이 예외를 적용하지 않는다.
  Base authored reward, alive-roster XP와 item-duration consumption은 양쪽 branch에서 동일하다. Status
  `closed_by_user`.

## Final Review Confirmations

- 프로젝트는 개발 중이고 공개 계획이 없으므로 bootstrap 전용 release gate를 추가하지 않는다.
- 29 BasicAttackModifier는 의도된 temporary placeholder이며 기반 이후 별도 content batches로 교체한다.
- 소탕의 무위험 full base reward/alive-roster XP, active FinalBoss chain의 비포기성, V3 strict rejection,
  checkpoint load가 영구 무사망 기록을 rollback하지 않는 성질은 의도된 정책이다.
- F3 Nothing There는 쉬운 패턴의 damage-meter 역할이므로 현재 stat-only bootstrap simplification을 유지한다.

## Decision Gate Conclusion

독립 read-only 재검증과 2026-08-12 사용자 결정을 합쳐 PD001~010이 모두 닫혔다. PD004는 minimum
29-row bootstrap과 R1 dust 5/10/20, PD006은 minimum `ST1-NT60` bootstrap이다. 둘 다 final content
authority가 아니다. PD005는 one-step WhiteNight live bootstrap과 future final content를 분리한다. PD008은
Elite mixed fill, PD009는 tunable 150%/200%, PD010은 mastered-sweep reacquisition cycle을 고정한다. Open material gate는
0개이며 investigation verdict는 `ready_for_implementation`이다. 이는 별도 구현 Goal 없이 production
mutation을 승인한다는 뜻은 아니다.
