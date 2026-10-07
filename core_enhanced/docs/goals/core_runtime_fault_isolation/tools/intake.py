#!/usr/bin/env python3
import hashlib,json,subprocess,tarfile
from initialize import HERE,ROOT
from run import identity,sha
assert (HERE/'P2G_60_RESULT_001.json').is_file()
d=HERE/'execution/single_goal';d.mkdir(parents=True,exist_ok=False)
for name in ['baseline','evidence']: (d/name).mkdir()
b=identity();prior=json.loads((HERE/'evidence/PREDECESSOR_BASELINE_001.json').read_text())
# Intake source review was fixed before authoring; assert every selected source still exact.
idx=json.loads((HERE/'SOURCE_INDEX_003.json').read_text())
for row in idx['files']: assert sha(ROOT/row['path'])==row['sha256'],row['path']
for rel in ['core_enhanced/docs/core_runtime_contract.md','core_enhanced/docs/data_loading_contract.md','core_enhanced/docs/README.md','core_enhanced/docs/ROADMAP_STATUS.md','core_enhanced/docs/roadmaps/core_first/ROADMAP.ko.md','game_server/README.md']:
 p=ROOT/rel
 if p.is_file() and not any(r['path']==rel for r in b['files']):b['files'].append({'path':rel,'bytes':p.stat().st_size,'sha256':sha(p)})
b['files'].sort(key=lambda r:r['path']);b['sha256']=hashlib.sha256(json.dumps(b['files'],sort_keys=True).encode()).hexdigest()
(d/'baseline/SOURCE_BEFORE.json').write_text(json.dumps(b,indent=2)+'\n')
with tarfile.open(d/'baseline/SOURCE_BEFORE.tar.gz','x:gz') as t:
 for row in b['files']:t.add(ROOT/row['path'],arcname=row['path'],recursive=False)
(d/'baseline/ARCHIVE.json').write_text(json.dumps({'sha256':sha(d/'baseline/SOURCE_BEFORE.tar.gz'),'source_sha256':b['sha256'],'files':len(b['files'])},indent=2)+'\n')
(d/'baseline/GIT_STATUS.txt').write_text(subprocess.check_output(['git','status','--short'],cwd=ROOT,text=True))
(d/'baseline/CURRENTNESS.json').write_text(json.dumps({'predecessor':'../../../../evidence/PREDECESSOR_BASELINE_001.json','receipt_sha256':sha(HERE/'evidence/PREDECESSOR_BASELINE_001.json'),'review_source_index_sha256':sha(HERE/'SOURCE_INDEX_003.json'),'reviewed_source_all_unchanged':True,'baseline':'U02 independent Master002 Complete/High; exact Core/host product359 unchanged at U03 intake; initial tests reused only. Changed inputs invalidate affected seams.','production_mutation':False},indent=2)+'\n')
(d/'PLAN.md').write_text('''# GOAL-CRFI-01 실행

GEX revision2026-09-13-gex-r7; P2G r7 quality2. Root core_first U03/A06.
Mode goal-run: original continuous request in ../../P2G_00_INITIAL_001.json IN-001 and
../../../roadmaps/core_first/execution/CONTINUOUS_REQUEST_20261004.ko.md; delegated decisions CF-D06/commands CF-D07.
Selected GS-CRFI-001 / GOAL-CRFI-01 / PHASE-CRFI-01, AG-CRFI-RUNTIME + WP-CRFI-VERIFY.
Upstream ../../P2G_60_HANDOFF_001.json HA-CRFI-001, ../../P2G_50_GOAL_SET.ko.md,
../../P2G_30_FREEZE_003.json FPF-CRFI-003, ../../P2G_40_PHASE_PLAN_002.json PP-CRFI-002,
../../P2G_40_RECEIPT_002.json; schema closures ../../P2G_40_RESULT_002.json. All exact refs in Handoff.

GEX-00 complete: sole single Goal eligible after U02 independent Master002 Complete/High.
Before actual bytes archived in baseline/SOURCE_BEFORE.tar.gz and SOURCE_BEFORE.json; digest ARCHIVE.json.
Existing dirty/untracked files are user baseline; no git reset or broad cleanup. Source reviewed59 unchanged.
Initial U02 Core1701/1701/1707, host77/84 and separate allocator/public consumer baseline reused via
../../evidence/PREDECESSOR_BASELINE_001.json; not represented as post-U03 validation.
This pass made no production mutation. Next separate GEX-10: Core→Host/observation→integrated verification.

Scope, source design and required acceptance: ../../CODE_REVIEW_001.ko.md RQ01–08 TD01–08 V01–12
and exact Goal. All12 frozen acceptances/all10 dimensions required. Core owns bounded sticky first fault
outside GameCore rollback; raw typed errors classified with operation origin before diagnostics.
Every public mutation/export/capture/reset gate, actual actor/timer/reconnect/admin/management/writer,
minimal fault snapshot and observer terminal cleanup delivered as one atomic unit.
Explicit Quit→new owner→existing validated Continue only; private speculative retry retained.
No U04 normal result/save policy,20minute test,arbitrary caps,save deletion or original audit edits.

Required validation: focused actual native/dynamic-expression faults and benign refusals; source-bound
Core default/test-fixtures/observation and host default/observation final matrix,fmt/check/clippy,
release public Core/headless websocket host,real file/stop/two-player/recovery,observer/private authoring,
100 predecessor conditions+31 R06 mapping. Preserve failed/zero executions and R05 warning/OBS001.
Restore proof must exercise exact before/delta/after. No self-awarded Complete verdict.
After stable target, separate implementation-nonparticipant GPT-6 Astra / High audit; fix/re-audit
findings before any U04 implementation. No intermediate approval/checkpoint stop was requested.

Roles: PLAN.md current scope/pass; EXPERIMENTS.md commands/evidence; EXPERIMENT_NOTES.md choices/failures;
COVERAGE.json seam/condition receipts; evidence/ immutable runs; baseline/ original; target/ audit closure.
Stop only on user stop, approved roadmap completion, or an actual unresolvable permission/environment blocker.
''')
(d/'EXPERIMENTS.md').write_text('# U03 검증 기록\n\nP2G-40/50 current validators passed; semantic mapping and Handoff complete.\nInitial baseline reused from exact U02 audited source; all post-change U03 evidence pending.\nGEX-00 source/dirty archive complete; no production mutation.\n')
(d/'EXPERIMENT_NOTES.md').write_text('# U03 기술 선택 및 실패\n\nP2G schema-only successor repairs preserved; policy meanings unchanged. Python executable is python3 (python absent).\nConcrete source boundary refinement follows CODE_REVIEW TD01–08 before mutation.\n')
with (HERE/'WORKING_POLICY.ko.md').open('a') as f:f.write('\nP2G-50/60 complete; [Handoff](P2G_60_HANDOFF_001.json), [GEX-00 completed PLAN](execution/single_goal/PLAN.md). Next GEX-10 under original continuous authority.\n')
print('GEX-00 complete, archived',len(b['files']),'files; no product mutation')
