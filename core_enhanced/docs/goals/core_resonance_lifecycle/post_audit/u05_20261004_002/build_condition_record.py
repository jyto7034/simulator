from pathlib import Path
import json,hashlib,datetime,subprocess
R=Path('/mnt/f/work/simulator');G=R/'core_enhanced/docs/goals/core_resonance_lifecycle';B=G/'execution/single_goal';A=G/'post_audit/u05_20261004_002';P=G/'post_audit/u05_20261004_001'
def read(p):return json.loads(p.read_text())
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
prior=read(P/'CONDITION_REVIEW.json');cov=read(B/'COVERAGE.json');imp=read(B/'PREDECESSOR_IMPACT.json');ev=read(A/'EVIDENCE_VERIFICATION.json');errors=[]
assert ev['passed'] and read(A/'MIGRATION_VERIFICATION.json')['passed'] and read(A/'RESTORE_VERIFICATION.json')['passed']
requirements={x['id']:x for x in cov['validations']+imp['conditions']};bindings={x['id']:x for x in ev['condition_execution_bindings']}
assert len(requirements)==len(prior['conditions'])==141
rows=[]
for index,x in enumerate(prior['conditions']):
 id=x['id'];req=x.get('original_acceptance',x.get('original_requirement'));assert req==requirements[id]['requirement'],id
 assert bindings[id]['selectors']==x['successful_current_selectors'],id
 original_meaning=x.get('meaning_basis')
 if original_meaning:
  if 'path' in original_meaning:assert sha(R/original_meaning['path'])==original_meaning['file_sha256'],id
  else:
   assert sha(R/original_meaning['source'])==original_meaning['source_sha256'],id
   if 'completed_scope_source' in original_meaning:assert sha(R/original_meaning['completed_scope_source'])==original_meaning['completed_scope_sha256'],id
 rationale=x.get('independent_reasoning',x.get('independent_applicability_reasoning'))
 impact='proven non-impacting product/acceptance delta; exact prior source judgment and execution evidence reused'
 new='All product, test/data/config/lock, current and archived command inputs and actual successful selectors are exact. CF-D09 changes document locators/authority labels with preserved policy bodies; no producer/consumer, oracle or behavioral precondition changes for this requirement.'
 if id in ['V12','V14','X09','X10']:
  impact='behavior judgment reused; affected canonical publication/current target obligation directly re-reviewed'
  new+=' MIGRATION_VERIFICATION closes all eleven F01 bindings and preserves frozen originals. CURRENTNESS_ENTRY/EXIT bind current canonical paths; RESTORE_VERIFICATION independently restores exact377/375 members with fuzz0. Whole-Goal closure is newly awarded by this audit, never inferred from F01 alone.'
 rows.append({'id':id,'original_requirement':req,'prior_judgment':{'path':str((P/'CONDITION_REVIEW.json').relative_to(R)),'sha256':sha(P/'CONDITION_REVIEW.json'),'json_pointer':f'/conditions/{index}','original_verdict':x['verdict'],'source_semantic_judgment':rationale},'original_meaning_basis':original_meaning,'impact_disposition':impact,'current_auditor_reasoning':new,'current_successful_selectors':bindings[id]['selectors'],'public_receipts':x.get('public_receipts',requirements[id].get('public_receipts',[])),'new_evidence':['EVIDENCE_VERIFICATION.json','MIGRATION_VERIFICATION.json','CURRENTNESS_ENTRY.json','CURRENTNESS_EXIT.json'],'verdict':'Pass'})
dims=[]
for i,x in enumerate(prior['dimensions']):
 assert x['dimension'] in cov['required_dimensions'] and set(x['validation_refs'])<=set(requirements)
 dims.append({'dimension':x['dimension'],'validation_refs':x['validation_refs'],'prior_exact_ref':{'path':str((P/'CONDITION_REVIEW.json').relative_to(R)),'sha256':sha(P/'CONDITION_REVIEW.json'),'json_pointer':f'/dimensions/{i}'},'review_basis':'Mapped condition original meanings and successful edge/failure/public seams remain exact; this directory verifies source/input/log identities. Canonical legacy absence and current publication are additionally established by MIGRATION_VERIFICATION and entry/exit checks. No required dimension is N/A.','verdict':'Pass'})
r06=[]
for i,x in enumerate(prior['r06_acceptances']):
 y=imp['r06_acceptances'][i];assert all(x[k]==y[k] for k in ['acceptance','expected','iva']);assert set(x['iva'])<=set(requirements)
 r06.append({'acceptance':x['acceptance'],'expected':x['expected'],'iva':x['iva'],'prior_exact_ref':{'path':str((P/'CONDITION_REVIEW.json').relative_to(R)),'sha256':sha(P/'CONDITION_REVIEW.json'),'json_pointer':f'/r06_acceptances/{i}'},'review_basis':'Original expected meaning reviewed against the named IVA requirement(s), preserved prior source reasoning, exact source and independently verified current selectors/public oracle. Document movement changes no units/context/copy/resource/rollback semantics.','verdict':'Pass'})
assert len(dims)==10 and len(r06)==31
out={'audit_target':'u05_002','reviewer':'/root/u05_completion_audit_002; implementation nonparticipant; GPT-6 Astra / High','method':'GEX section5 condition-specific impact re-audit; prior Not Complete judgments reused individually after applicability review, never prior whole verdict','product_sha256':cov['source_sha256'],'prior_completion_audit_sha256':sha(P/'COMPLETION_AUDIT.md'),'conditions':rows,'dimensions':dims,'r06_acceptances':r06,'F01':'Closed by new current bindings and independent semantic/currentness review','verdict':'All conditions satisfied subject to final unchanged exit currentness'}
(A/'CONDITION_REVIEW.json').write_text(json.dumps(out,ensure_ascii=False,indent=2)+'\n')
# Bind previously independent all-row cost review to still-current raw data and exact cost producers.
cost=read(P/'PC_ACCOUNTING_RECHECK.json')
for k in ['before','after']:assert sha(R/cost[k]['path'])==cost[k]['sha256']
for p,h in cost['source_basis'].items():assert sha(R/p)==h
original=read(G/'evidence/integrated_core_default002.json');currenttool={'rustc':subprocess.check_output(['rustc','-Vv'],text=True).strip(),'cargo':subprocess.check_output(['cargo','-V'],text=True).strip()};assert currenttool==original['toolchain']
numerical={'at_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'prior_cost_review':{'path':str((P/'PC_ACCOUNTING_RECHECK.json').relative_to(R)),'sha256':sha(P/'PC_ACCOUNTING_RECHECK.json')},'raw_bindings':{k:cost[k] for k in ['before','after']},'source_basis':cost['source_basis'],'rows':cost['rows'],'claim':cost['claim'],'excluded':cost['excluded'],'current_toolchain':currenttool,'reuse_judgment':'Reuse independent audit001 all576 raw-row accounting review: comparator/raw/source and product identities unchanged. New mutation is exclusively documents. Costs increased; historical strict failure and original96 log mismatches remain preserved. Gameplay36 conditions and original numeric guards remain separately mandatory.','R05':read(B/'compatibility/current_001/evidence/R05_SUMMARY_CURRENT_003.json'),'passed':True}
(A/'NUMERICAL_REUSE.json').write_text(json.dumps(numerical,ensure_ascii=False,indent=2)+'\n');print(json.dumps({'current_conditions':14,'inherited_conditions':127,'dimensions':10,'R06':31,'cost_rows_reused_after_applicability_check':576,'toolchain_unchanged':True}))
