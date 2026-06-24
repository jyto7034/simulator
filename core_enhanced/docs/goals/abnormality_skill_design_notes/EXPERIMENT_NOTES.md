# Abnormality Skill Design Notes Experiment Notes

## 2026-06-12

- This is a documentation/content-design goal. Rust tests are not required unless code or live data changes, which are out of scope.
- Core `docs/skills/skill_fragment_system.md` is newer than `/mnt/f/unity projects/ark/docs/skill_fragment_system.md`; use core for WhiteNight policy.
- The design note should not flatten `Special` abnormalities into simple stat modifiers merely because placeholder fragments currently use `BasicAttackModifier`.
- Source text must be summarized and transformed into game design notes, not copied.
- The three Black Forest birds are tightly coupled to Apocalypse Bird. Their individual entries should still stand alone, but their dedicated fragment designs should leave room for a later combined boss/fragment system.
- Big Bird's literal lure mechanic conflicts with the current "deployed allies do not move" policy. Record it as an open design question instead of deciding a movement exception inside this documentation goal.
- Several S-grade abnormalities need concepts that are broader than the current generic `ActiveSkill`: phase sequences, summons, corpse resources, infection propagation, forced pull, panic/fear states, or faction changes.
- Magical girl fragments should probably share a family risk vocabulary later, but this goal should not define that policy.
- Little Red Riding Hooded Mercenary and Big and Will be Bad Wolf are a paired content problem. The notes keep them individually usable, but a later implementation goal should decide whether paired rivalry events exist in run generation.
- A-grade entries introduced possession and observation rules through The Red Shoes and The Burrowing Heaven. These should not be implemented as hidden core behavior without a separate policy discussion because they affect player control and Unity presentation.
- B-grade entries confirmed that lower-priority abnormalities still need major systems in a few cases: conditional healing punishments, spore/minion conversion, corpse/death hooks, paired Yin/Yang resonance, and camera/focus observation rules.
- Schadenfreude and The Burrowing Heaven both use observation/gaze concepts, but direct camera visibility is likely a poor core dependency. The design note records focus/attention approximations as the safer long-term direction unless the user explicitly wants camera-aware combat rules.
- Fairy Festival, Singing Machine, Firebird, and Yin/Yang may fit noncombat/special-node rewards better than ordinary active skill fragments. Keep that as a content-architecture decision for later, not a silent assumption in this note.
