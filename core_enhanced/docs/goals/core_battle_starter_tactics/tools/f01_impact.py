"""Capture the bounded F01 product delta and exact old evidence applicability."""
import datetime,difflib,json,re,tarfile
from run import ROOT,HERE,identity,sha
S=HERE/'execution/single_goal';F=S/'correction/f01_001';T=S/'target/u06_001';A=HERE/'post_audit/u06_20261005_001'
old=json.loads((T/'MANIFEST.json').read_text())['product_source'];current=identity()
before={r['path']:r for r in old['files']};after={r['path']:r for r in current['files']}
assert before.keys()==after.keys()
changed=sorted(p for p in before if before[p]!=after[p])
production='core_enhanced/src/game/battle/core/basic_heal.rs'
query_tests='core_enhanced/src/game/battle/core/targeting.rs'
public_tests='core_enhanced/src/game/world/tests/u06_starter.rs'
assert changed==sorted([production,query_tests,public_tests]),changed
patches=[];old_text={}
with tarfile.open(T/'SOURCE_AFTER.tar.gz') as tar:
 for path in changed:
  raw=tar.extractfile(path).read();old_text[path]=raw.decode()
  assert __import__('hashlib').sha256(raw).hexdigest()==before[path]['sha256']
  patches.extend(difflib.unified_diff(old_text[path].splitlines(keepends=True),(ROOT/path).read_text().splitlines(keepends=True),fromfile='a/'+path,tofile='b/'+path))
assert old_text[query_tests].split('#[cfg(test)]')[0]==(ROOT/query_tests).read_text().split('#[cfg(test)]')[0]
assert '#[cfg(test)]' in old_text[query_tests]
assert '#[cfg(test)]\nmod tests;' in (ROOT/'core_enhanced/src/game/world.rs').read_text()
(F/'PRODUCT_DELTA.patch').write_text(''.join(patches))
C=S/'compatibility/current_001';adapt=json.loads((C/'ADAPTATION_FINAL.json').read_text())
for row in adapt['files']:assert sha(ROOT/row['copy'])==row['copy_sha256']
inputs=[row for row in adapt['files'] if row['semantic_change'].startswith('none;')]
assert len(inputs)==419
assert not [row['copy'] for row in inputs if b'action: Heal(' in (ROOT/row['copy']).read_bytes()]
# The only official basic-Heal definitions have strictly positive flat terms.
heal_sources=[]
for root in ['game_resources/data','game_resources/live_data']:
 for p in (ROOT/root).rglob('*.ron'):
  for match in re.finditer(r'action:\s*Heal\(flat:\s*(\d+),\s*spell_power_percent:\s*(\d+)',p.read_text()):
   assert int(match[1])>0
   heal_sources.append({'path':str(p.relative_to(ROOT)),'sha256':sha(p),'flat':int(match[1]),'percent':int(match[2])})
assert len(heal_sources)==2,heal_sources
pc=json.loads((C/'evidence/PC_IMMEDIATE_ACCOUNTED_U06.json').read_text())
for path,digest in pc['source_basis'].items():assert sha(ROOT/path)==digest
selection=json.loads((S/'EVIDENCE_SELECTION.json').read_text())
reused=selection['checks'][6:]+[n for n in selection['public_receipts'] if n!='public_starter_schema_002']+[selection['r05_receipt']]
rows={}
for name in reused:
 receipt=HERE/'evidence'/(name+'.json');r=json.loads(receipt.read_text());log=HERE/'evidence'/(name+'.log')
 assert r['source_before']==old==r['source_after'] and r['source_unchanged']
 assert r['exit_code'] in ((0,101) if name==selection['r05_receipt'] else (0,))
 assert r['log_sha256']==sha(log)
 if name.startswith('compat_pc_'):
  basis='PC fixed36/576 synthetic employee_default profiles use Damage with no weapon profile. BasicHeal owner is never invoked; all eight layout/accounting source files and frozen probe/oracle inputs are byte-identical.'
 elif name.startswith('compat_r05_') or name.startswith('public_guards'):
  basis='Unchanged final_performance/public_guards workload uses Little Red and dummy abnormality Damage actors, not starter Heal. Retained state/CloneShape/meter and numerical caps are unchanged. Timing is an old shared-host observation, not a fresh measurement.'
 elif name.startswith('compat_') or name in ('public_schema_001','public_independent_area_001'):
  basis='All419 inherited input copies and18 adapted harness files are byte-identical. Inherited basic profiles do not author Heal. Parser/schema/defaults/skills/trace/CLI/host/storage code is unchanged; the affected basic-Heal public path is separately freshly executed.'
 else:
  basis='Host/store/transport/fault/native-observation owners and inputs are byte-identical. Current official basic Heal flat terms8/4 are always positive for u32 SP; their arithmetic/release result is unchanged. New zero-effective fixtures are covered by fresh CoreApplication, native and original-auditor probes.'
 rows[name]={'receipt_sha256':sha(receipt),'log_sha256':sha(log),'actual_execution_source_sha256':old['sha256'],'applicable_current_source_sha256':current['sha256'],'basis':basis,'rerun':False}
result={'recorded_at_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'status':'bounded source/input impact assessment; required fresh validation and independent judgment remain pending','source_before':old,'source_after':current,'changed_product_files':changed,'production_delta':[production],'test_only_delta':[query_tests,public_tests],'patch_sha256':sha(F/'PRODUCT_DELTA.patch'),'prior_manifest_sha256':sha(T/'MANIFEST.json'),'prior_audit_closure_sha256':sha(A/'AUDIT_CLOSURE.json'),'policy_unchanged':True,'data_schema_layout_unchanged':True,'native_accounting_source_basis':pc['source_basis'],'unchanged_inherited_inputs':419,'unchanged_harness_files':18,'official_positive_heal_definitions':heal_sources,'source_reasoning':'One allocation-free amount calculation is extracted verbatim from release. The shared is_basic_heal_target predicate now requires the computed signed-range-clamped amount to be positive. It only admits Heal; Damage paths never call this owner. For existing positive Heal the same amount is applied. No field, queue, serializer, source credential, carrier, global action gate, public DTO or cost limit changes. Only ineffective Heal automatic eligibility changes, intentionally correcting F01. New tests cover query/start/release/current stats and old behavioral combinations rerun in final native suites.','reused_receipts':rows,'fresh_required':['integrated_*009 five full suites plus native static/heap','f01_public_consumer_001','f01_reproduce_001','f01_public_release_build_001','public_starter_schema_003'],'limitations':['No claim that old executions ran on the new source. Exact old receipts retain their original hashes and current applicability is this impact judgment, subject to independent GEX-20.','No fresh PC/R05 latency or allocator measurement is asserted. Damage-only workload branch and unchanged layout/accounting are the reuse basis. Existing R05 warning and stress exclusion are unchanged.']}
with (F/'EVIDENCE_REUSE.json').open('x') as f:json.dump(result,f,ensure_ascii=False,indent=2);f.write('\n')
print(json.dumps({'changed_product_files':changed,'reused_receipts':len(rows),'current':current['sha256']}))
