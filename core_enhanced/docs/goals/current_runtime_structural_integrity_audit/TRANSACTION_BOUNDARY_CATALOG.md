# Fallible Mutation and Transaction Boundary Catalog

- 기준선: `B-001`
- 상태: 최종

## T-001. Shop commands

| path | preflight/staging | authoritative mutation | post-mutation fallible work | current evidence |
| --- | --- | --- | --- | --- |
| sale | `sale_evaluation`; checked quantity/quote/currency; clone inventory and remove on clone | assign inventory, assign currency | result construction only | overflow candidate rejected; E-007 |
| ordinary purchase | lookup/eligibility; clone `EconomyGrantState`; checked subtraction; grant effects on clone; clone session | commit grant state, commit session | `commit_shop_session(AllowActiveOnlyWithoutRun)` has no error branch after staging | static atomic path; E-007 |
| monolith purchase | lookup/eligibility; clone monolith; checked purchase/subtraction; clone session | assign monolith, currency, session | commit helper mode has no fallible branch | static atomic path; E-007 |
| refresh | eligibility; lookup; checked next index/cost/uses; generate page and clone session | assign currency, commit session | commit helper mode has no fallible branch | static atomic path; E-007 |

`commit_shop_session`의 `RequireRun` mode는 `run_state_mut()?`를 mutation 전에 실행한 뒤 map/active session을
연속 assign한다. `AllowActiveOnlyWithoutRun` mode는 error를 생성하지 않는다. 현재 관측 범위에서는 shop
command의 `Err` 뒤 partial authority mutation을 찾지 못했다.

## T-002. Live battle command families

| family | cursor/precondition order | mutation strategy | `Err` equality |
| --- | --- | --- | --- |
| elapsed/next/server-tick advance | revision+cursor preflight before step | staged active/VN/triggers/UUID/game-state/content rollback | statically preserved for staged roots |
| deploy/withdraw/stance/relocate/swap | revision preflight; DTO cursor check on staged clone | clone `ActiveBattleSession`, replace after result/DTO, restore on VN attachment failure | preserved |
| pause/resume/set speed | state preflight, live mutation+revision, then DTO cursor check | no staging/rollback | violated for future cursor; RUNTIME-AUDIT-005 |
| begin/cancel targeting | state/skill preflight, live focus/playback mutation+revision, then DTO cursor | no staging/rollback | violated; RUNTIME-AUDIT-005 |
| activate skill | skill validation, live BattleCore command/signals/focus/revision, then DTO cursor | no staging/rollback | violated with broad effects; RUNTIME-AUDIT-005 |

The public root validates `BattleStateAuthorityDto` before dispatch but delegates cursor validation to each handler, making
the transaction meaning inconsistent across command families.

## T-003. Map/node/checkpoint/reward/event families

| family | strategy | fallible-after-commit assessment |
| --- | --- | --- |
| confirm/convert/complete node | `with_state_transaction` plus staged map/progression/content plans | full non-battle rollback; monotonic player profile intentionally merged |
| explicit checkpoint load | strict payload/graph validation then `with_state_transaction` restore | error restores non-battle state; E-011 |
| reward/economy grants | `EconomyGrantState::capture`, apply all effects on clone, then `commit_into` | no fallible work after grant commit in inspected paths |
| event employee check grant | validate/roll/plan grant or battle before session mutation | grant branch commits planned economy then infallible session assignment under established run invariant |
| event-started combat | plan complete battle before committing check/session, then commit battle start | committed event check is intentionally retained for retreat/re-entry; rare post-plan engine failure lacks an outer transaction, but no concrete official-data trigger was established |
| StartNewGame | destructive root initialization before trigger evaluation | violated on public invalid data; RUNTIME-AUDIT-006 |

The inspected positive families demonstrate staged/transaction patterns exist, strengthening the conclusion that
RUNTIME-AUDIT-005/006 are inconsistent adoption rather than absence of transaction machinery.
