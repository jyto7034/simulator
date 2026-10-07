# Policy-To-Implementation Documentation Migration Map

Cutover rule: this map is prepared while the seven existing documents remain current owners. The manual becomes the sole normative owner only when the manual, README/engineering routing changes, and five compatibility entrypoints are changed in the same scoped cutover.

Cutover outcome: complete. The manual is `current`; README and engineering routing have changed; all five legacy paths are non-normative entrypoints. No old owner remains current.

## File-level owner migration

| Current file | Current role | Post-cutover role | Normative destination |
| --- | --- | --- | --- |
| `docs/README.md` | Index plus duplicated source/pipeline rules | Docs index and brief manual link | `PIP-00`, `PIP-10` |
| `docs/engineering_workflow.md` | Generic engineering rules plus PIP/Goal lifecycle fragments | Generic source selection, canonical-update matrix, refactor classification, test and Git/file safety | PIP lifecycle → manual; generic rules remain local |
| `docs/policy_implementation_readiness_review_guide.md` | Alignment/readiness/dependency/execution lifecycle owner | Non-normative compatibility entrypoint and ATI crosswalk | `PIP-00`, `PIP-10`–`PIP-80` |
| `docs/goal_authoring_and_execution_guide.md` | Goal state, authorization, execution, audit owner | Non-normative compatibility entrypoint and anchor map | `PIP-00`, `PIP-30`–`PIP-90` |
| `docs/goal_completion_review_guide.md` | Completion audit owner | Non-normative compatibility entrypoint and anchor map | `PIP-60`, `PIP-70`, `PIP-80`, `PIP-90` |
| `docs/policy_implementation_readiness_goal_prompt_template.md` | Large reusable prompt with copied lifecycle rules | Thin Campaign/manual invocation prompt | `PIP-10`, `PIP-20`, `PIP-30`, `PIP-90` |
| `docs/goal_execution_resume_prompt_template.md` | Resume prompt with copied Gate rules | Thin exact-Goal execution/resume invocation prompt | `PIP-40`, `PIP-80`, `PIP-90` |

## Top-level section mapping

### README

| Existing section | Destination/disposition |
| --- | --- |
| `이 문서의 책임`, `현재 문서 체계`, archive/external/content/decision/Goal indexes | Retained and compressed in README; link targets preserved |
| `Source Of Truth 순서` | Manual `PIP-00#source-and-authority-model`; README keeps one-sentence pointer |
| `Policy-To-Implementation Pipeline` | Manual Reader Contract, `PIP-00#pipeline-and-ati-crosswalk`, `PIP-10`–`70` |
| `정리 원칙` | README retains index-maintenance-only rules; PIP history rule moves to `PIP-80` |

### Engineering workflow

| Existing section | Destination/disposition |
| --- | --- |
| Responsibility, purpose, source selection, canonical boundary | Generic engineering content retained; PIP-specific precedence points to `PIP-00` |
| Code/document sync and canonical update matrix | Retained as generic classification; PIP timing/realization owned by `PIP-50` |
| Implementation principles | Generic minimal-change/authority principles retained; authorization and atomic PIP rules point to manual |
| `Goal Lifecycle And Review Order` | Manual `PIP-00#pipeline`, `PIP-40`, `PIP-60`, `PIP-70`, `PIP-80` |
| Refactor sections | Retained unchanged in responsibility, with completion Fit link to `PIP-60` |
| Legacy/docs cleanup | Generic cleanup retained; PIP compatibility/atomic cutover moves to `PIP-50` |
| Tests/debug outputs | Retained as generic runner/tool hygiene; PIP evidence semantics moves to `PIP-00`, `PIP-50` |
| User confirmation | Generic pointer; materiality owner is `PIP-00`/`PIP-20` |
| Goal operation/template/completion report | Manual `PIP-40`, `PIP-60`, `PIP-90` |
| Git/File Safety | Retained generic safety; PIP baseline attribution owner is `PIP-00` |

### Readiness guide

| Existing top-level section | Manual destination |
| --- | --- |
| `0. End-to-End Lifecycle` | `PIP-00#pipeline-and-ati-crosswalk` |
| `1–4`, `4A` purpose, modes, inputs, closure | `PIP-10`, `PIP-20` |
| `5–7` Goal boundary, baseline, review | `PIP-20`, `PIP-30`, global baseline in `PIP-00` |
| `8–10` difficulty/risk/confidence/policy decisions | `PIP-20` |
| `11–12` packages/dependency/sequence | `PIP-30` |
| `12A–12B` Goal handoff/preflight | `PIP-40` |
| `13–14` migration/compatibility/test-first | `PIP-50` |
| `15`, `15C` gate and activation | `PIP-40`, `PIP-60` |
| `15A` change control | `PIP-80` |
| `15B` Full/Fast | `PIP-10`, `PIP-40` |
| `16`, `16A` artifacts/templates | `PIP-90` |
| `17–18` readiness completion | `PIP-20`, `PIP-30`, `PIP-40` |
| `18A` implementation/audit handoff | `PIP-50`, `PIP-60`, `PIP-70` |
| `19–22` stop/fail/report/campaign completion | owner partition’s rewind plus `PIP-70`, formats `PIP-90` |

### Goal authoring/execution guide

| Existing top-level section | Manual destination |
| --- | --- |
| `0`, `0.1` pipeline/terms | `PIP-00` |
| `0.2` metadata | `PIP-00#state-model`, `PIP-90#metadata` |
| `0.3` evidence refs | `PIP-00#evidence`, format in `PIP-90` |
| `0.4` authorization | `PIP-00#authorization`, binding in `PIP-40` |
| `1` inputs | `PIP-10`, `PIP-40` |
| `2` Single/Master | `PIP-30` |
| `3`, `3A` new/resume/Gate/handoff | `PIP-40`; archive/correction in `PIP-80` |
| `4` Goal artifact responsibilities | `PIP-90` |
| `5` statuses/dispositions | state model `PIP-00`, events `PIP-80` |
| `6–7` investigation/policy decision | `PIP-20` |
| `8` dirty baseline | `PIP-00`, phase check `PIP-40` |
| `9–12` implementation/legacy/test/canonical | `PIP-50` |
| `13`, `13A` completion/finalization | `PIP-60` |
| `14` Master/Phase | `PIP-30`, `PIP-40`, `PIP-60`, `PIP-70` |
| `15–18` completion/report/templates | `PIP-60`, `PIP-70`, `PIP-80`, `PIP-90` |

### Completion guide

| Existing top-level section | Manual destination |
| --- | --- |
| `1–6` purpose, mode, input, SoT, matrix, procedure | `PIP-60` plus global source/evidence in `PIP-00` |
| `7–12` runtime/data/DTO/store/consumer/legacy/diff/validation/canonical | Phase execution evidence `PIP-50`, audit checks `PIP-60` |
| `13`, `13A`, `13B` long-term Fit/reviewer/finalization | `PIP-60` |
| `14` verdict/status mapping | state model `PIP-00`, failure events `PIP-80`, verdict `PIP-60` |
| `15` Phase audit | `PIP-60` |
| `16` cross-component/Master audit | `PIP-70` |
| `17–18` audit template/output | `PIP-90`, gates `PIP-60/70` |

### Prompt templates

| Existing section | Destination/disposition |
| --- | --- |
| Responsibility/use/placeholder/input sections | Retained in thin entrypoint, reduced to invocation fields |
| Copied reusable lifecycle prompt | Removed as duplicate prose; invokes `PIP-10/20/30/40/90` |
| Instance checklist/change rules | Manual `PIP-90`; entrypoint checks only that required invocation fields are supplied |

## Historical and active-reference treatment

- Archive Goal files are not bulk rewritten. Their old guide links remain historical context and resolve to compatibility entrypoints.
- Compatibility entrypoints explicitly warn that old metadata/ATI wording must be interpreted through the manual crosswalk for new or resumed work.
- Existing terminal audits remain immutable. A future correction creates a linked Goal under `PIP-80`.
- This consolidation Goal is the only active/current reference that must bridge the cutover. Its Schema 2 metadata remains historical-compatible; finalization follows its frozen Gate and then the new manual’s `PIP-60` invariants.
- No runtime/data/product canonical is in the allowed scope; “canonical sync” for this Goal is the docs owner cutover itself.

## Explicit removals

No file is deleted or moved. The following prose classes are removed from current entrypoints only after their normalized owner exists:

- duplicate lifecycle narratives;
- repeated source/dirty/authorization/checklist rules;
- copied status tables and stop-condition lists;
- full prompt bodies that shadow guide rules;
- redundant rationale, repeated examples, and historical explanation.

Their normative meanings are mapped in `RULE_INVENTORY.md`; compatibility links and historical file paths remain.
