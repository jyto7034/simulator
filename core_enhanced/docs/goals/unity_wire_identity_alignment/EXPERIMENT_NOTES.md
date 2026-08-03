# Unity Wire Identity Alignment Notes

Goal: G02

Status: complete

## Fixed Decisions

- Rust typed requests/DTOs are the authority unless the accepted PD-04 coordinated Event cutover
  explicitly replaces the inner Event shape in G03.
- Do not accept both old and new names.
- game_server forwards typed payloads and must not repair identities with JSON rewriting.
- Unity readers and senders consume exact fields; no Raw, display-name, or result-name fallback.

## G03 Boundary

G02 may make root and command-result Event envelopes identical and correct their documentation.
It must leave panels, check choices, d20 persistence, success/failure actions, Event presentation,
and Event/VN removal to G03/G19.

## Current Questions For Discovery

- Which of DCA-H-09/H-10 remain reproducible after recent command/read-model work?
- Are the skill/result/root documentation findings code defects or canonical-only drift?
- Does `selected_event.event` already reuse the command-result inner DTO exactly?
- Which Unity tests currently pin stale aliases or mock-only field names?

## Resolved Findings

- DCA-H-09 reproduced and corrected: Unity now sends `target_employee_uuid` only.
- DCA-H-10 reproduced and corrected: HQ candidates read `id`; reward options read instance `uuid`;
  stale snapshot aliases are rejected.
- DCA-M-14/M-15 were canonical drift: skill examples now use `range_policy`, and Floor completion
  uses `FloorAdvanced`.
- DCA-M-16 is explicit in code and docs: `CombatResultModel.PrimaryAbnormalityId` reads
  `primary_abnormality_id`; mock data matches.
- DCA-L-02 root example includes `visual_novel` and `skill_catalog`.
- DCA-L-03 reproduced and corrected: active consumable display reads `name` only.
- Root selected Event and `EventState` already shared the same typed inner DTO; a Core regression
  now pins this G03 predecessor seam.

## Conclusion

Complete / High / no immediate correction. G03 may replace the shared Event inner DTO through one
coordinated cutover without revisiting these generic identities.
