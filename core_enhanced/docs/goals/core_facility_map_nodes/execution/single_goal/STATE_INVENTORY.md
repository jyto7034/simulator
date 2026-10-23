# State ownership

| State | Owner/lifetime | Restore |
| --- | --- | --- |
| layout/edges/templates/reservations/display refs | validated data generation | immutable content identity |
| current map and progression | RunState/current room | checkpoint/Continue |
| noncurrent visited map/progression, sampled metadata | facility run state/current expedition | checkpoint/Continue, no reroll |
| node sessions/entry receipts/preview/attempt | current expedition, global node UUID | references resolve across all visited rooms |
| explored room/final knowledge/actual victory compendium | profile/permanent | monotonic merge; no run allocation copied |
| profile+active accepted publication | existing paired host intent | exact accepted recovery/CAS |
