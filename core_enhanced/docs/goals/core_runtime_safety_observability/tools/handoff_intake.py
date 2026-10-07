#!/usr/bin/env python3
import hashlib
import json
import subprocess
import tarfile
from initialize import HERE, ROOT, artifact, write
from run import identity, sha
v=json.loads((HERE/'P2G_50_VALIDATION_002.json').read_text());assert v['result']=='passed' and not v['failures']
def first(name):return json.loads((HERE/name).read_text())['artifacts'][0]
freeze=first('P2G_30_FREEZE_001.json');plan=first('P2G_40_PHASE_PLAN_001.json')
goal={'Artifact Type':'Goal Set','Artifact ID':'GS-CRSO-001','Content Digest':'sha256:'+sha(HERE/'P2G_50_GOAL_SET.ko.md')}
write('P2G_50_RESULT_001.json',{'stage':'P2G-50','status':'complete','Goal Set Ref':goal,'member_ids':['GOAL-CRSO-MASTER','GOAL-CRSO-S1','GOAL-CRSO-S2'],'validator':'P2G_50_VALIDATION_002.json','semantic_review':'TD01–07 and all primary/supporting boundaries mapped; actual server persistence rollback included; no assumed deployed consumer or Grafana execution; full10 dimensions per Phase; independent audit required.', 'P2G-50 Resolution Record Ref Set':[],'next':'P2G-60'})
handoff=artifact('Handoff Artifact','HA-CRSO-001',{
 'Predecessor Handoff Artifact Ref':None,'Result Composite Basis':{'Working Revision Ref':freeze['payload']['Working Revision Ref'],'Final Policy Freeze Ref':freeze['ref'],'Phase Plan Ref':plan['ref'],'Goal Set Ref':goal},
 'Final Policy Freeze Ref':freeze['ref'],'Phase Plan Ref':plan['ref'],'Goal Set Ref':goal,
 'Remaining Gap/Blocker/Limitation Summary':'No policy/source gap preventing GEX. Implementation/tests/current performance and actual external Grafana evidence pending. Server baseline fails and public application adapter/persistence capability is required S1 work. Queue and exact interface details constrained by CODE_REVIEW TD01–07. R05 warning retained.',
 'Scope/Authorization Boundary':'P2G only policy/plan/Goal/Handoff. No activation, implementation authorization, production mutation or completion verdict by this artifact. User continuous request separately authorizes GEX implementation/audits/correction and subsequent roadmap work.',
 'Out-of-Scope Summary':'20minute test, arbitrary new caps/per-battle RSS cap, U03 session fault policy, continuation/automatic skill cancellation, mod self-approved settings, Unity/final boss unique content, unrelated dirty work/save deletion.',
 'Lifecycle Disposition':'current'})
write('P2G_60_HANDOFF_001.json',{'artifacts':[handoff]})
write('P2G_60_RESULT_001.json',{'stage':'P2G-60','status':'complete','Handoff Artifact Ref':handoff['ref'],'P2G-60 Resolution Record Ref Set':[],'checks':{'upstream_exact':True,'all12_policy_to_phase_goal':True,'Master_no_implementation':True,'no_production_mutation':True,'known_unknowns_preserved':True},'next':'separately user-authorized GEX-00 GOAL-CRSO-S1'})
# Separate GEX-00 pass: preserve a complete source basis before any mutation.
d=HERE/'execution/s1';d.mkdir(parents=True,exist_ok=False);(d/'baseline').mkdir();(d/'evidence').mkdir()
before=identity()
for rel in ['core_enhanced/docs/core_runtime_contract.md','core_enhanced/docs/data_loading_contract.md','core_enhanced/docs/README.md','core_enhanced/docs/ROADMAP_STATUS.md','core_enhanced/docs/roadmaps/core_first/ROADMAP.ko.md']:
 p=ROOT/rel
 if p.exists() and not any(r['path']==rel for r in before['files']):before['files'].append({'path':rel,'bytes':p.stat().st_size,'sha256':sha(p)})
before['files'].sort(key=lambda r:r['path']);before['sha256']=hashlib.sha256(json.dumps(before['files'],sort_keys=True).encode()).hexdigest()
(d/'baseline/SOURCE_BEFORE.json').write_text(json.dumps(before,indent=2)+'\n')
with tarfile.open(d/'baseline/SOURCE_BEFORE.tar.gz','w:gz') as tar:
 for row in before['files']:tar.add(ROOT/row['path'],arcname=row['path'])
(d/'baseline/ARCHIVE.json').write_text(json.dumps({'sha256':sha(d/'baseline/SOURCE_BEFORE.tar.gz'),'source_sha256':before['sha256'],'files':len(before['files'])},indent=2)+'\n')
u01=ROOT/'core_enhanced/docs/goals/content_authoring_r08_r12_runtime/execution/single_goal/corrections/f01_action_flow_20261004_001'
prior=json.loads((u01/'target/f01_action_flow_001/MANIFEST.json').read_text())
matched=[];changed=[]
for path,size,digest in prior['source_files']:
 if path.startswith(('core_enhanced/src/','core_enhanced/tests/','game_resources/data/')) or path in ['Cargo.toml','Cargo.lock','core_enhanced/Cargo.toml']:
  (matched if sha(ROOT/path)==digest else changed).append(path)
assert not changed,changed
closure=ROOT/'core_enhanced/docs/goals/content_authoring_r08_r12_runtime/post_audit/20261004_f01_001/AUDIT_CLOSURE.json'
(d/'baseline/CURRENTNESS.json').write_text(json.dumps({'u01_manifest_sha256':sha(u01/'target/f01_action_flow_001/MANIFEST.json'),'u01_audit_closure_sha256':sha(closure),'relevant_u01_source_matched':len(matched),'changed':changed,'reused_baseline_receipts':[str((u01/'evidence'/f).relative_to(ROOT)) for f in ['all_default_002.json','all_feature_001.json','public_oracle_after_001.json','public_cli_001.json'] if (u01/'evidence'/f).exists()],'limitations':'Exact unchanged relevant source baseline reuse; future edits invalidate affected seams. Server baseline failure separately preserved.'},indent=2)+'\n')
(d/'PLAN.md').write_text('''# GOAL-CRSO-S1 실행

GEX revision: 2026-09-13-gex-r7. P2G current r7, quality contract2.
Root: core_first U02, GS-CRSO-001; selected GOAL-CRSO-S1 / PHASE-CRSO-S1.
Exact upstream: ../../P2G_60_HANDOFF_001.json HA-CRSO-001, ../../P2G_50_GOAL_SET.ko.md,
../../P2G_30_FREEZE_001.json, ../../P2G_40_PHASE_PLAN_001.json, ../../P2G_40_RECEIPT_001.json.
Request: ../../P2G_00_INITIAL_001.json IN-001 and core_first CONTINUOUS_REQUEST_20261004.ko.md.

Mode goal-run, explicit implementation/self-tests/independent audit/correction and next-card authority.
Intermediate review opt-in: no. Audit handoff stop is superseded by explicit user audit authorization.
S2 must wait for S1 independent Complete/High; Master GEX-30 after both Phase audits.
Reviewer: separate implementation-nonparticipant GPT-6 Astra / High, only after stable target.

GEX-00 complete: P2G current exact chain, U01 independent closure and relevant source identity verified;
S1 is the only eligible implementation member. No production change during intake.
GEX-10 now active: CONFIG→CORE→PORTS→S1VERIFY. Detailed decisions in ../../CODE_REVIEW_001.ko.md TD01–04.
No metrics dependency here. S1 changes settings/default/constructor/generation/authoring/host/history only.

Before source: baseline/SOURCE_BEFORE.json + SOURCE_BEFORE.tar.gz; immutable SHA in ARCHIVE.json.
Prior Core baseline reuse: baseline/CURRENTNESS.json. Server baseline fails before U02
(../../evidence/server_before_check_001.json); fix is explicitly assigned PORTS, not a fresh regression.
Existing dirty/untracked work is baseline, not agent-owned by git status. No reset or broad cleanup.
Each owned final delta is measured against archived actual bytes; new membership tracked.

Validation: exact Goal V01–08/V12 and all10 dimensions. Relevant baseline invalidated by product/fixture/config
changes must run on current source, nonzero. Full final core default/test-fixtures, changed host tests/check,
format/clippy, actual public CLI/host and predecessor U01/R06/R07 impact. R05 raw warning separated from policy.
No arbitrary new latency cap or20minute run. Implementation complete requires each mandatory seam, restore
proof, final target/records/references and independent audit; self-tests do not create Complete verdict.

Roles: PLAN.md current scope/status; EXPERIMENTS.md command/evidence ledger; EXPERIMENT_NOTES.md choices/failures;
COVERAGE.json current required conditions; evidence/ immutable logs; baseline/ before actual; target/ final audit target.
Stop only user stop, approved scope complete, or actual unresolvable blocker within current permissions/environment.
Next: GEX-10 detailed default/constructor API preparation and product implementation.
''')
(d/'EXPERIMENTS.md').write_text('# S1 실험 기록\n\nGEX-00 인수: 관련 U01 Core source unchanged, audited baseline reused. Server before check exit101 preserved.\nP2G40 passed; P2G50 first formatting check failed3fields, corrected before publication; second passed20dimensions.\n')
(d/'EXPERIMENT_NOTES.md').write_text('# S1 선택과 실패 기록\n\nP2G canonical 필드 표현을 successor revision으로 교정했고 이전 원본은 보존했다. 의미 변경 없음.\nHost persistence rollback은 public opaque session/revision capability로 준비하며 private Core export는 금지한다.\n')
view=(HERE/'WORKING_POLICY.ko.md').read_text();view+='\n현재 P2G-60 complete, GS-CRSO-001/GOAL-CRSO-S1 GEX-00 complete → GEX-10 active.\n[실행 PLAN](execution/s1/PLAN.md), [Handoff](P2G_60_HANDOFF_001.json). S2는 S1 감사 후 실행한다.\n';(HERE/'WORKING_POLICY.ko.md').write_text(view)
print('P2G-50/60 complete; GEX-00 S1 complete;',len(before['files']),'before files archived;',len(matched),'U01 relevant files unchanged; GEX-10 eligible')
