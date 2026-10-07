# Phase 05 Test Reaudit

## Current Results

| Runner/family | Passed | Failed | Ignored/skipped | Meaning |
| --- | ---: | ---: | ---: | --- |
| combat preview unit family | 27 | 0 | 0 | generation/difficulty/route/template/serde behavior |
| combat executor setup family | 15 | 0 | 0 | aggregate plan, Boss/no-binding parity and scenarios |
| typed Boss setup family | 3 | 0 | 0 | opt-out, immutable binding, route/spawn rejection |
| live RON integration | 25 | 0 | 0 | official live publication/reference/setup inputs |
| manifested-only + invalid Boss setup negatives | 2 | 0 | 0 | C02 publication closure |
| world typed setup/ordinary opt-out/late failure exact tests | 3 | 0 | 0 | C04 world handoff and atomicity |
| **Total physical** | **75** | **0** | **0** | nonmatching filters excluded |

## Dispositions

| Test family | Disposition |
| --- | --- |
| C02 enumeration and structured setup negatives | `retain` — full publication closure |
| aggregate typed setup, bound/no-binding ordinary parity | `retain` — C04 closure and Boss opt-in boundary |
| preview generation/serde | `retain + strengthen` — add exact durable Continue C01 sequence |
| resolver/conversion goldens | `strengthen before C03` — normal/Boss/Event/Monolith field/cache parity |
| mission policy shape-only tests | `merge/remove only after assertion transfer` — C05 backlog |

No focused test is ignored. Green preview serde only proves the skipped value defaults; it does not make that default
safe when a persisted cache later feeds setup.
