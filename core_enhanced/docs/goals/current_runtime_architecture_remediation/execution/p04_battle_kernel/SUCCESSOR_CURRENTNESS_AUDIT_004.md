# P04 Successor Currentness Audit 004

Verdict: `Complete / High — current / expected P07 application battle caller cutover`

## Identity and Trigger

- Record: `ATR-CRAR-C2-P04-GEX20-004`; completion basis remains `ATR-CRAR-C2-P04-GEX10-001`.
- Triggering successor: P07 migrates the real live battle caller from public `GameCore` to scoped
  `CoreApplication` command/snapshot/reconnect ports.

## Direct Current Evidence

- Comparison against the P06-current P04 basis is 27/31 unchanged. No battle kernel production source changes; only
  the canonical contract and positive caller manifest/lock/source change.
- The positive default-release probe loads actual validated data, starts a run, enters a private live battle, projects
  the run snapshot and obtains an identical repeated reconnect read model. The public result retains the existing
  `BattleAdvanced` payload identity but is explicitly `ReadOnly`.
- Full Core is 1,157/1,157. The negative probe still fails with `E0603` for raw kernel access, and release compilation
  is warning-free.

## Disposition

P04 remains `Complete / High`. Kernel fields, constructors, floats, events and mutation stay private; P07 adds no raw
battle handle or second kernel writer.
