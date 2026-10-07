import pathlib,json,hashlib,re,tarfile
R=pathlib.Path('/mnt/f/work/simulator');H=R/'core_enhanced/docs/goals/core_result_persistence';A=pathlib.Path(__file__).parent;S=H/'execution/single_goal';C=json.loads((S/'COVERAGE_003.json').read_text());T=S/'target/u04_003';M=json.loads((T/'MANIFEST.json').read_text())
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
rows=[];logs={}
for name,b in C['receipt_bindings'].items():
 p=H/'evidence'/f'{name}.json';l=p.with_suffix('.log');d=json.loads(p.read_text());assert sha(p)==b['receipt_sha256'];assert sha(l)==b['log_sha256']==d['log_sha256'];assert d['source_unchanged']
 assert d['source_before']==d['source_after']==M['product_source'];assert d['source_before']['sha256']==C['source_sha256'];assert d['exit_code']==b['raw_exit_code']
 assert d['exit_code']==(101 if name=='compat_r05_005' else 0),(name,d['exit_code'])
 for inp in d['command_inputs']:assert sha(H/inp['archive'])==inp['sha256']
 assert sha(H/'evidence/runner_versions'/(d['runner_sha256']+'.py'))==d['runner_sha256']
 log=l.read_text(errors='replace');logs[name]=log
 counts=[list(map(int,m)) for m in re.findall(r'test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored;',log)]
 totals=[sum(r[i] for r in counts) for i in range(3)]
 rows.append({'id':name,'command':d['command'],'exit':d['exit_code'],'receipt_sha256':sha(p),'log_sha256':sha(l),'source_sha256':C['source_sha256'],'test_totals':totals if counts else None})
for name,expected in C['suite_counts'].items():
 actual=next(r for r in rows if r['id']==name)['test_totals'];assert actual==[expected[x] for x in ['passed','failed','ignored']],(name,actual,expected)
selector_rows=[];P=json.loads((S/'PREDECESSOR_IMPACT.json').read_text())
selectors=set()
for v in C['validations']:
 for k in ['core_selectors','host_selectors','observed_selectors']:selectors.update(v[k])
for r in P['conditions']:selectors.update(r.get('selectors',[]))
# current log binding checks exact named results, not substring counts
for selector in sorted(selectors):
 hits=[n for n,l in logs.items() if re.search(r'^test '+re.escape(selector)+r' \.\.\. ok$',l,re.M)]
 selector_rows.append({'selector':selector,'passing_receipts':hits})
# Retired selectors are deliberate references; record them for semantic adjudication.
missing=[r['selector'] for r in selector_rows if not r['passing_receipts']]
(A/'EVIDENCE_BINDINGS.json').write_text(json.dumps({'receipts':rows,'selectors':selector_rows,'selectors_without_exact_pass':missing},indent=2)+'\n')
print('receipts',len(rows),'selectors',len(selectors),'missing',missing)
