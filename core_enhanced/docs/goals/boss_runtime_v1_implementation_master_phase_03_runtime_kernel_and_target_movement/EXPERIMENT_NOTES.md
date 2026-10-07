# Boss Runtime v1 Phase 03 — Notes

## Activation

- Boss Runtime remains the main track. P03 implements no additional refactor-audit candidate.
- P02 data/setup is an immutable input; P03 must not create a second compiler, raw lookup path or generic argument map.
- Production Boss data remains empty, so all runtime behavior is exercised with Core-local compiled fixtures until
  P04 owns the authored vertical slice.
- Evidence is written after each bounded discovery/test/implementation unit so context compaction cannot erase it.

## Initial runtime constraints

- Encounter runtime and actor runtimes are distinct lifetimes; actor death/late spawn cannot delete or recreate the
  encounter authority.
- The same Core battle clock and queue own every wake/deadline. No wall clock, Unity update or parallel scheduler exists.
- A Boss Task chooses high-level intent; existing movement/attack/skill/damage/death modules decide whether and how the
  leaf action executes.
- Target lock is separate from `RuntimeUnit.current_target`, and path feasibility is separate from target validity.
- Whole-chain staging is the preferred atomicity shell because it naturally covers runtime, leaf state, queue/log and
  movement intent together. Per-transition plans still make ownership and validation explicit inside that shell.
- P03 implements only the compiler-supported FixedSequence and five Task variants. Known future variants remain
  rejected at publication rather than acquiring a runtime fallback.
