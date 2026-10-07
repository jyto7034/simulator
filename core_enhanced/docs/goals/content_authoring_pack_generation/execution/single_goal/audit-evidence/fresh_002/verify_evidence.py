#!/usr/bin/env python3
"""Independent bounded source delta, execution receipt and preservation review."""
import difflib
import hashlib
import json
from pathlib import Path
import re
from datetime import datetime, timezone

ROOT = Path('/mnt/f/work/simulator')
BASE = ROOT / 'core_enhanced/docs/goals/content_authoring_pack_generation/execution/single_goal'
OUT = BASE / 'audit-evidence/fresh_002'
def sha(p): return hashlib.sha256(p.read_bytes()).hexdigest() if p.is_file() else None
def digest(b): return hashlib.sha256(b).hexdigest()
def read(n): return json.loads((BASE/n).read_text())
def inputs(n): return {r['path']:r['sha256'] for r in read('VALIDATION_INPUT_'+n+'.json')['inputs']}
def require(value, message):
    if not value: raise AssertionError(message)
def unified(before, after, old, new):
    return ''.join(difflib.unified_diff(before.splitlines(keepends=True),after.splitlines(keepends=True),fromfile=old,tofile=new))

def reconstruct_patch(patch, originals):
    lines=patch.splitlines(keepends=True); result={}; i=0
    while i<len(lines):
        require(lines[i].startswith('--- '), 'patch old header')
        path=lines[i][4:].strip().split('/',1)[1]
        require(lines[i+1].startswith('+++ '), 'patch new header'); i+=2
        source=(originals/path).read_text().splitlines(keepends=True); out=[]; pos=0
        while i<len(lines) and not lines[i].startswith('--- '):
            m=re.match(r'@@ -(\d+)(?:,(\d+))? \+(\d+)(?:,(\d+))? @@',lines[i]);require(m,'hunk header')
            start=int(m[1])-1; out.extend(source[pos:start]);pos=start;i+=1
            while i<len(lines) and not lines[i].startswith(('@@ ','--- ')):
                line=lines[i];i+=1
                if line[0] in ' -':
                    require(source[pos]==line[1:],'original hunk context '+path);pos+=1
                if line[0] in ' +': out.append(line[1:])
        out.extend(source[pos:]);result[path]=''.join(out)
    return result

def main():
    before=inputs('023'); current=inputs('027'); mid=inputs('025')
    require(set(before)==set(current)==set(mid),'input inventories changed')
    require(all(sha(ROOT/p)==s for p,s in current.items()),'current VI027 hash')
    original=Path('/tmp/capg-r02-correction-003-before')
    require(all(sha(original/p)==s for p,s in before.items()),'original VI023 copies')
    changed=sorted(p for p in current if current[p]!=before[p])
    impact=read('evidence/CORRECTION_IMPACT_003_FINAL.json')
    require(changed==sorted(r['path'] for r in impact['changed_inputs']),'correction change attribution')
    patch=''.join(unified((original/p).read_text(),(ROOT/p).read_text(),'before003/'+p,'after003/'+p) for p in changed)
    require(patch==(BASE/impact['patch']).read_text(),'final correction patch not exact')
    require(digest(patch.encode())==impact['patch_sha256'],'correction patch digest')
    prior=reconstruct_patch((BASE/'evidence/CORRECTION_DIFF_003.patch').read_text(),original)
    require(all(digest(s.encode())==mid[p] for p,s in prior.items()),'VI025 reconstruction')
    mid_delta=sorted(p for p in mid if mid[p]!=current[p])
    require(mid_delta==['core_enhanced/src/game/data/content_pack/tests.rs','core_enhanced/src/game/data/validation.rs'],'API delta')
    old=prior[mid_delta[0]];new=(ROOT/mid_delta[0]).read_text()
    require(new.startswith(old),'API earlier test bytes changed')
    p='core_enhanced/src/game/data/validation.rs';old=prior[p];new=(ROOT/p).read_text()
    a=old.index('    fn error(',old.index('impl GeneratedValidationContext'));b=old.index('\nfn baseline_tuning_path',a)
    c=new.index('    fn error(',new.index('impl GeneratedValidationContext'));d=new.index('\nfn baseline_tuning_path',c)
    require(old[:a]==new[:c] and old[b:]==new[d:],'API source outside error helper changed')
    vi022=inputs('022')
    observer_paths=sorted(p for p in vi022 if p.startswith(('core_enhanced/src/game/world','core_enhanced/tools/')) or p=='core_enhanced/src/application.rs')
    require(all(vi022[p]==current[p] for p in observer_paths),'observer world/application/tool input delta')
    actual_inputs={str(p.relative_to(ROOT)) for root in ['core_enhanced/src','core_enhanced/tests','game_resources/data'] for p in (ROOT/root).rglob('*') if p.is_file() and p.suffix in ('.rs','.ron')}
    require(not(actual_inputs-set(current)),'source/test/RON omission')
    t2=read('AUDIT_TARGET_002.json');t3=read('AUDIT_TARGET_003.json')
    m2={r['path']:r['sha256'] for r in t2['manifest']};m3={r['path']:r['sha256'] for r in t3['manifest']}
    target_delta={'changed':sorted(p for p in m2 if p in m3 and m2[p]!=m3[p]),'added':sorted(set(m3)-set(m2)),'removed':sorted(set(m2)-set(m3))}
    baseline=read('BASELINE_001.json');pres=read('evidence/PRESERVATION_003.json');orig=Path(baseline['backup_path'])
    require(all(sha(orig/p)==s for p,s in baseline['scope_hashes'].items()),'1850 original hashes')
    baseline_changed=sorted(p for p,s in baseline['scope_hashes'].items() if sha(ROOT/p)!=s)
    require(baseline_changed==sorted(r['path'] for r in pres['changed_existing']),'baseline changed set')
    require(not[p for p,s in baseline['scope_hashes'].items() if s and not(ROOT/p).is_file()],'baseline removal')
    scoped=''.join(unified((orig/p).read_text() if (orig/p).is_file() else '',(ROOT/p).read_text(),'a/'+p,'b/'+p) for p in pres['scoped_diff_paths'])
    require(scoped==(BASE/pres['scoped_diff']).read_text(),'full scoped patch differs')
    tools=read('evidence/TOOL_SUPPLEMENTAL_002.json')
    require(all(sha(ROOT/r['path'])==r['after_sha256'] for r in tools['files']),'supplemental tool after hashes')
    prior_tool=read('audit-evidence/fresh_001/tool_lock_review.json')
    tool_unchanged=[{'path':r['path'],'sha256':sha(ROOT/r['path'])} for r in tools['files']]
    b=BASE.parent.parent;handoff=json.loads((b/'P2G_60_HANDOFF_001.json').read_text())
    bindings=handoff['source_locations']+handoff['external_context_locations']
    require(all(sha(ROOT/r['path'])==r['sha256'] for r in bindings),'handoff hash')
    receipt=json.loads((b/'P2G_40_PHASE_PLAN_VALIDATION_RECEIPT_001.json').read_text())
    require(receipt['overall_result']=='passed' and not receipt['failure_summary'],'phase plan receipt')
    phase_bindings=[receipt[k] for k in ('evaluated_phase_plan','bound_final_policy_freeze')]+receipt['structural_input_evidence']['resolved_artifact_sources']
    require(all(sha(ROOT/r['path'])==r['sha256'] for r in phase_bindings),'exact phase receipt input bindings')
    executions=[]
    names=['full_core_004','correction_generated_003','all_targets_003','release_check_003','release_build_007','release_ab_005','correction_release_diagnostic_004','api_batch_002','public_guards_002','public_guards_allocation_001']
    for name in names:
        r=read('evidence/'+name+'.json');item={'name':name,'record_sha256':sha(BASE/'evidence'/ (name+'.json')),'command':r['command'],'terminal_exit_code':r['terminal_exit_code']}
        require(r['terminal_exit_code']==0,name+' terminal')
        for k in ('input','log','receipt'):
            if k in r and k+'_sha256' in r:
                require(sha(BASE/r[k])==r[k+'_sha256'],name+' '+k+' hash')
                item[k]=r[k];item[k+'_sha256']=r[k+'_sha256']
        if 'log' in r:
            log=(BASE/r['log']).read_text();summaries=[list(map(int,m)) for m in re.findall(r'test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored; (\d+) measured; (\d+) filtered out',log)]
            if summaries:item['actual_test_summaries']=summaries
            if name=='full_core_004':require([x[0] for x in summaries]==[1165,4,4,25,15,4] and all(sum(x[1:])==0 for x in summaries),'full core count')
            if name=='correction_generated_003':require(summaries==[[16,0,0,0,1149]],'generated counts')
        if 'receipt' in r:
            rc=read(r['receipt']);commands=rc.get('commands',[])
            if commands:
                successes=sum(x['exit_code']==0 for x in commands);item['actual_commands']=len(commands);item['successes']=successes;item['nonzero_exits']=len(commands)-successes
                durable=(BASE/r['receipt']).parent
                for cmd in commands:
                    for key in ('stdout','stderr','log'):
                        if key+'_sha256' not in cmd:continue
                        filename=Path(cmd[key]).name if key in cmd else cmd['name']+'.'+key
                        path=durable/filename;require(sha(path)==cmd[key+'_sha256'],name+' subprocess '+filename)
                    if cmd.get('required_diagnostics'):
                        text=(durable/Path(cmd['log']).name).read_text()
                        require(cmd['exit_code']!=0 and all(x in text for x in cmd['required_diagnostics']),name+' actual Rust diagnostics')
                if name in ('release_ab_005','correction_release_diagnostic_004'):require((len(commands),successes)==(10,4),name+' count')
                if name=='api_batch_002':require((len(commands),successes)==(11,5),'api count')
            if name=='release_ab_005':require(rc['actual_starting_currency']==[500,733,500],'AB result')
            if name=='correction_release_diagnostic_004':
                require(sha(durable/'probe.py')==rc['script_sha256'],'diagnostic script')
                require(all(rc[k] for k in ('original_inputs_restored','identical_valid_stdout','identical_repeated_errors','contract_oracle_pass')),'diagnostic actual oracles')
                policy=(durable/'invalid_b_1.stderr').read_text()
                require(all(x in policy for x in ['b/policy.ron',rc['mutation_locator'],'official/data/pve/encounters.ron','official/data/map/battlefield_templates.ron']),'policy source')
                for case in ('zone','route'):
                    text=(durable/('boss_invalid_'+case+'_1.stderr')).read_text()
                    require(all(x in text for x in ['custom/pattern.ron','patterns[0]','cwbs_reference_boss_behavior']),'boss contributor')
            if name.startswith('public_guards'):
                require(sha(ROOT/'core_enhanced/tools/runtime_architecture_final_validation/src/public_guards.rs')==rc['tool_source_sha256'],'observer exact source')
                for key in ('run_snapshot','checkpoint_export'):
                    require(rc[key]['sample_count']==101 and rc[key]['exact_repeatable'] and rc[key]['encoded_byte_cap_pass'],'observer direct oracles')
                require(rc['transaction_staging']['rejection_rollback_exact'],'observer rollback')
                require(rc['bounded_history_query']['all_caps_respected'] and rc['bounded_history_query']['scaling_pass'],'observer history')
                if rc['latency_gate_applicable']:
                    require(all(rc[k]['absolute_cap_pass'] for k in ('run_snapshot','checkpoint_export')),'observer latency caps')
                if rc['allocation_gate_applicable']:
                    require(rc['transaction_staging']['allocation_retained_history_independent_pass'],'observer allocation cap')
                    require(all(len(rc['transaction_staging'][k])==5 for k in ('low_allocation_raw_samples','high_allocation_raw_samples')),'observer allocation samples')
        executions.append(item)
    result={'reviewer':'/root/r02_correction_fresh_audit','observed_at':datetime.now(timezone.utc).isoformat(),
        'actual_check_kind':'read-only hash/content/source-evidence comparison; no new tests, builds or gameplay',
        'current_input_count':len(current),'original_vi023_copies':len(before),'correction_changed':changed,'correction_unchanged':len(current)-len(changed),
        'reconstructed_correction_patch_sha256':digest(patch.encode()),'VI025_reconstructed_files':len(prior),'VI025_to027_changed':mid_delta,
        'VI025_to027_unchanged':len(current)-len(mid_delta),'source_test_RON_enumerated':len(actual_inputs),'input_omissions':[],
        'observer_world_application_tools_input_paths_unchanged':observer_paths,
        'prior_target_delta':target_delta,'preservation':{'original_paths':len(baseline['scope_hashes']),'baseline_changed_existing':baseline_changed,
          'new_source_paths':pres['new_source_paths'],'scoped_patch_paths':len(pres['scoped_diff_paths']),'scoped_patch_sha256':digest(scoped.encode()),
          'supplemental_tools_unchanged':len(tools['files']),'tool_main_original_hash_limit':8,'excluded_parent_views_may_change':True},
        'handoff_bindings':bindings,'passed_phase_plan_receipt':receipt['phase_plan_validation_receipt_ref'],
        'phase_receipt_bindings':{k:receipt[k] for k in ('evaluated_phase_plan','bound_final_policy_freeze','structural_input_evidence')},
        'executions_reused':executions,'problems':[]}
    (OUT/'evidence_review.json').write_text(json.dumps(result,indent=2)+'\n')
    print(json.dumps({'current_input_count':len(current),'correction_changed':len(changed),'correction_unchanged':len(current)-len(changed),
        'prior_target_changed':len(target_delta['changed']),'prior_target_added':len(target_delta['added']),'prior_target_removed':len(target_delta['removed']),
        'baseline_paths':len(baseline['scope_hashes']),'baseline_changed':len(baseline_changed),'scoped_patch_paths':len(pres['scoped_diff_paths']),
        'executions':executions,'problems':[]},indent=2))

if __name__=='__main__':main()
