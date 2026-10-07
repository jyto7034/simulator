# Phase 04 Test Reaudit

## Current Results

| Runner | Passed | Failed | Ignored/skipped | Meaning |
| --- | ---: | ---: | ---: | --- |
| `visual_novel` filter | 24 | 0 | 0 | data/live validation, trigger ordering/checkpoint and battle overlay |
| Event evaluator unit filter | 5 | 0 | 0 | choice/check math and key evaluation |
| node-session filter | 18 | 0 | 0 | HQ/Shop/Reward/Event-adjacent persistence and atomicity |
| facility filter | 11 | 0 | 0 | VN MapNode trigger, queue priority and checkpoint lifetime |
| **Physical executions** | **58** | **0** | **0** | overlapping filters retained; nonmatches excluded |

## Dispositions

| Family | Disposition | Missing/retained invariant |
| --- | --- | --- |
| typed/live trigger validation | `retain + strengthen` | negative matrix for every runtime-rejected repeat/source pair |
| Event/Shop/HQ lifecycle | `retain + strengthen` | persisted/active equality for every entry/update and explicit no-run semantics |
| HQ happy paths | `retain + strengthen` | full-state late-failure rollback for recruit/emergency/common completion |
| VN trigger order/checkpoint | `retain` | unique Tail/non-interruption and consumption timing |
| Monolith exchange VN | `strengthen` | queue-front, exact UUID progression, accept/decline and Shop blocking |
| Boss battle VN overlay | `retain under battle/VN owners` | consumer regression; not a new P04 candidate |

No focused test is ignored. Green tests do not cover the named publication, HQ rollback or Monolith placement gaps.
