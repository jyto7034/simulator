# Elite Monolith Remanifestation Experiment Notes

Status: complete notes

Implementation complete: true

Created: 2026-07-27

## 1. Source-of-truth boundary

- Approved decision fixes progression, transaction, eligibility, attempts, exchange, compendium and completion
  semantics.
- It does not provide an actual hidden-boss actor/skill/encounter design. The older concept note explicitly leaves
  the real encounter undecided.
- Initial numeric shop/remanifestation balance may be authored conservatively under decision section 12, but a
  new boss's attack mechanics, wave composition and counterplay are player-visible content meaning rather than a
  mere coefficient choice.

## 2. Historical pause and approved split

- P05-A/B는 처음에는 hidden-boss P05-C와 한 phase여서 큰 partial migration 위험 때문에 중단됐다.
- 2026-07-27 사용자는 `2번`, 즉 모노리스 시스템 전체를 먼저 구현하고 실제 보스를 뒤로 미루는 경계를
  명시적으로 승인했다.
- 따라서 residual exchange와 corrupted-monolith acquisition까지가 현재 P05의 완결된 vertical slice다.
  Corrupted key는 P06 전까지 consumer가 없는 run-local state이며 기존 Endless completion을 바꾸지 않는다.

## 3. Rejected substitutes

- Do not reuse or rename `t-02-43_freischutz` / `suppress_freischutz`; policy requires a distinct canonical actor.
- Do not create a stat-only copy, empty wave, placeholder reward, blank VN or test-only encounter and call P05
  complete.
- Do not silently omit the corrupted-monolith path while shipping P05-A/B as the whole phase.

## 4. Inputs that are already sufficient

- General final boss identities and the two required reclassifications are explicit.
- Shop page cardinality, eligibility, refresh behavior, purchase lifetime and Floor restriction are explicit.
- Node conversion, stage progression, two attempts, fragment timing and residual conversion are explicit.
- Exchange count, guarantee timing, VN choice cardinality, completion routing and persistence boundaries are explicit.
- P01~P04 technical seams are complete and available.

## 5. Future P06 unblock evidence required

Either an authored hidden-boss content package or explicit authorization for this child to design it must define at
least:

- distinct canonical abnormality/actor identity and display metadata;
- base combat stats, attack/skill behavior and readable telegraph/counterplay;
- encounter battlefield/objective/waves and one-attempt FinalBoss resolution;
- any non-empty ordinary combat reward profile intended at launch, or explicit confirmation that the policy's
  stated compendium/completion record is the complete launch reward;
- exchange and administrator VN local scripts/cues, speaker/background needs and exact player-facing choice text.

After approval, these inputs become live RON/Unity-local authored resources with strict reference tests before
P06 runtime migration begins. They do not block the active P05 monolith implementation.

## 6. Recommended content direction

The recommended distinct mechanic is a telegraphed fan/line shotgun that can pass through shooter-side allied
adds without treating them as the terminal target. Each such allied pass increases the following hostile hit and
impact explosion, so the player counterplay is to remove/reposition around fuel adds, split the employee formation
and avoid stacking behind one line. This preserves the concept seed while differing from Der Freischutz's single
long-range magic bullet and seventh-shot spike.

Implementation would add a typed, bounded, deterministic projectile power curve keyed by caster-side allied
collisions. It must project the charged shot/footprint before resolution and include collision order and applied
multiplier in battle evidence. Exact mechanic, adds, curve and VN prose remain unapproved until user input.

2026-07-27 사용자 결정에 따라 이 절은 승인 정책이나 구현 authority가 아닌 과거 검토 제안으로만
보존한다. 작업자는 이를 live content로 승격하거나 세부 구현하지 않는다. 완성된 콘텐츠 정책과 명시적
재개 요청이 들어오기 전까지 이 제안과 실제 hidden-boss production은 P06에 보류한다. 승인된 P05
모노리스 시스템 구현에는 사용하지 않는다.

## 7. P05 dormant-key safety boundary

- P05 accept exchange는 residual 네 개를 소비하고 corrupted-monolith key 하나를 주는 것으로 끝난다.
- Key는 inventory merchandise나 Elite conversion selector가 아니며 P05에서 사용할 command를 만들지 않는다.
- P05는 final-boss classification, final completion routing, hidden capstone manifestation을 수정하지 않는다.
- 이 경계 덕분에 모노리스 progression을 완결하면서 미정 보스 콘텐츠를 placeholder로 노출하지 않는다.
- P06이 활성화될 때 key consumer와 completion interception을 같은 transaction/vertical review로 추가한다.
