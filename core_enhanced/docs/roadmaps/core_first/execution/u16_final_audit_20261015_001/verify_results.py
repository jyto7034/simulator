"""Independent arithmetic/raw-log checks for this audit's reported results."""
import hashlib,json,re
from pathlib import Path
A=Path(__file__).resolve().parent
R=next(p for p in A.parents if (p/'game_server/Cargo.toml').exists())
load=lambda p:json.loads(p.read_text())
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
rows=[]
for name,amount,filename in [('small',1000,'RESULT003.json'),('large',10000000,'RESULT003.json'),('larger',100000000,'RESULT.json')]:
    p=A/('xp_'+name)/filename;v=load(p)
    trace=[json.loads(s) for s in p.with_name(p.name+'.trace.jsonl').read_text().splitlines()]
    assert v['passed'] and v['operations']==len(trace)==52 and v['exact_requested_bytes_recovered']
    assert all(x['matches_recorded_host_gameplay'] for x in trace)
    assert all(x['accepted']==(x['request'].get('reject')!=True) for x in trace)
    for x in trace:
        for employee in x['run'].get('roster',{}).get('employees',[]):
            expected=1 if x['index']<49 or employee['employee_definition_id']=='field_medic' else 1+amount+24
            assert (employee['level'],employee['experience'])==(expected,0),(name,x['index'],employee)
    stage=next(s for s in v['stages'] if s['kind']=='operation' and s['index']==49)
    assert stage['accepted'] and trace[49]['run']['game_state']=='combat_result'
    rows.append({'variant':name,'first_round_xp':amount,'level_up_iterations_two_survivors':2*(amount+24),
                 'public_operations':len(trace),'terminal_seconds':stage['seconds'],'accounted_native_work':stage['native']['work'],
                 'native_profile_hard_bytes':stage['native']['retained']['owner_hard_bytes'],
                 'exact_xp_arithmetic':True,'exact_requested_heap_recovered':True,
                 'raw_result':str(p.relative_to(R)),'raw_result_sha256':sha(p),'trace_sha256':sha(p.with_name(p.name+'.trace.jsonl'))})
log=(A/'focused.log').read_text()
passed=re.findall(r'^test (\S+) \.\.\. ok$',log,re.M)
assert len(passed)==45 and '45 passed; 0 failed; 0 ignored' in log
correction=A.parent/'test_correction_20261015_001'
assert all(sha(R/x['path'])==x['after_sha256'] for x in load(correction/'changes.json'))
feature=(correction/'feature_final001.log')
if not feature.exists():
    record=load(correction/'RESULT.json');feature=correction/next(r['log'] for r in record['runs'] if r.get('counts',{}).get('passed')==96)
feature_passed=set(re.findall(r'^test (\S+) \.\.\. ok$',feature.read_text(),re.M))
assert len(feature_passed)==96 and not re.search(r'^test .* \.\.\. FAILED$',feature.read_text(),re.M)
default=(A.parent/'full_tests_20261015_001/default.log').read_text()
assert '2156 passed; 0 failed; 6 ignored' in default
history=load(A/'HISTORY_RECHECK.json');assert len(history)==4 and all(x['passed'] for x in history)
result={'focused':{'passed':len(passed),'failed':0,'ignored':0,'exit_code':0,'names':passed,'log_sha256':sha(A/'focused.log')},
        'xp':rows,'xp_binary_sha256':sha(R/'.cache/u164-review-dev-target/debug/u16_xp_audit'),
        'historical_rechecks_passed':4,'post_audit_six_correction_files_match':True,
        'reused_feature_passes':len(feature_passed),'feature_log':str(feature.relative_to(R)),
        'default_core_lib_passes_reused':2156,'default_core_total_including_integration':2228,
        'note':'No new full-workspace run. Internal work is not elapsed time; the per-level loop has no work charge/poll. XP probe passes are reproductions, not a safety pass.'}
(A/'VERIFIED_RESULTS.json').write_text(json.dumps(result,indent=2,ensure_ascii=False)+'\n')
print(json.dumps({'focused':len(passed),'historical_rechecks':4,'xp':rows},ensure_ascii=False))
