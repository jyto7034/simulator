from pathlib import Path
import json,re
R=Path('/mnt/f/work/simulator');O=R/'core_enhanced/docs/testing/core_baseline_failure_correction_20261014_001';T=R/'core_enhanced/docs/testing/core_baseline_failure_triage_20261014_001'
runs=[];latest={}
for p in sorted(O.glob('*.result.json'),key=lambda p:p.stat().st_mtime):
 r=json.loads(p.read_text());s=(O/r['stdout']).read_text();summaries=re.findall(r'^test result: (.+)$',s,re.M)
 names={n:v for n,v in re.findall(r'^test (\S+)(?: - should panic)? \.\.\. (ok|FAILED|ignored)(?:, .*)?$',s,re.M)}
 label=p.name[:-12]
 env_path=O/(label+'.environment.json')
 runs.append({'label':label,**r,'environment':json.loads(env_path.read_text()) if env_path.exists() else {},'summaries':summaries,'printed_status_counts':{v:list(names.values()).count(v) for v in ['ok','FAILED','ignored']},'test_results':names})
 for n,v in names.items():latest[n]={'status':v,'run':label}
mapping=json.loads((O/'test_name_mapping.json').read_text());rows=[]
for item in json.loads((T/'MATRIX.json').read_text()):
 current=mapping.get(item['test'],[item['test']]);results={n:latest.get(n,{'status':'not_run'}) for n in current}
 rows.append({'id':item['id'],'original_test':item['test'],'original_source':item['source'],'original_cause':item['cause_id'],'correction_basis':item['next_action'],'disposition':item['recommendation'],'current_tests':current,'latest_execution':results,'all_current_tests_passed':all(v['status']=='ok' for v in results.values())})
(O/'EXECUTION_INDEX.json').write_text(json.dumps(runs,ensure_ascii=False,indent=2))
(O/'CORRECTION_MATRIX.json').write_text(json.dumps(rows,ensure_ascii=False,indent=2))
print('covered',sum(r['all_current_tests_passed'] for r in rows),'of',len(rows),'original tests')
for r in rows:
 if not r['all_current_tests_passed']:print(r['id'],r['latest_execution'])
