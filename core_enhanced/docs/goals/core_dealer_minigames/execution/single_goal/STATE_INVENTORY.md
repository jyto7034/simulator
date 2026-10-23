# U16-3 상태·소유·수명

| 상태 | 단일 소유/변경 경계 | 수명·저장·회수 |
| --- | --- | --- |
| DealerPolicy·상품/VN/원금/가중치 | validated FacilityLayout | immutable content identity; RON loader checks positive values, references and terminal choices |
| dealer_introduced | RunProgression | same expedition carry; new expedition false; whole-node checkpoint restores flag |
| DealerSession | RewardSession optional Box | current node only; original entry checkpoint owns rollback; completion drops session and residual points |
| points/wager/gambit/revision | DealerSession | i64 checked money; gambit fixed once; revision increments once successful transition, outer draft rejects unchanged |
| RNG/FlipShoe/current round | DealerSession | node-seeded deterministic draw; no future/hole/seed in view; one round+last_result, no growing history |
| escrow copies+principal | DealerSession escrow | exact original UUID/variant moved out of inventory; impossible to equip/corrupt/fuse/dismantle locked copies through live inventory |
| live copies/equipment/resources | existing RelicInventory/EconomyGrantState | capture/commit and application draft; redemption returns exact copy, foreclosure drops selected escrow, shop common grant gives Normal copies |
| selected collateral/redemption | DealerSession UUID list | no assets move until confirm; sorted unique, length/qualification/total checked; cancel/insufficient/stale leave roots unchanged |
| offers remaining | DealerSession | authored list copied once; purchase binds price/remaining/grant in one transaction; no re-roll |
| VN | existing normal VisualNovelSession | blocking EventNode source bound to node+configured VN; actual select choice hook executes dealer transition after closure inside application draft |
| public view | owner-produced DealerView/MiniRound view | current actions, amounts/rules/copies/stock/effects; BJ hole and all RNG/shoe state excluded; consumer does not derive domain eligibility |
| durable publication | unchanged paired host | intermediate encounter is non-durable; unfinished freshContinue restores whole entry, finished preserves purchase/redemption/forfeit; SavePoint and actualIO failure keep state+disk paired |
| allocation/work | existing clone/preflight/native operation scopes | fixed64KiB round workspace, serialized state extent+vector projections/selection/grant; default16MiB retained and10M authoring unchanged;9actual cost replays release all requested payload bytes |

No new independent save store, asset engine, event history or parallel command owner. Existing non-dealer Reward uses dealerNone and old run flag false serialization is omitted.
