"""Verify Master entry bindings without claiming an independent verdict."""
import json
from pathlib import Path
import sys
M=Path(__file__).resolve().parents[1]
D=M.parents[1]
sys.path.insert(0,str(D/'tools'))
import run
c=json.loads((M/'COVERAGE.json').read_text())
assert c['source_sha256']==run.identity()['sha256']
assert len(c['acceptances'])==12 and len(c['integration'])==13
assert all(a['verdict']=='PASS' for a in c['acceptances'])
assert len({a['policy_ref']['Artifact ID'] for a in c['acceptances']})==12
assert len({a['frozen_assignment']['id'] for a in c['integration']})==13
for path,digest in c['references'].items():
    assert run.sha(D/path)==digest,path
for name,binding in c['receipt_bindings'].items():
    p=D/'evidence'/(name+'.json')
    assert run.sha(p)==binding['receipt_sha256'],name
    assert run.sha(p.with_suffix('.log'))==binding['log_sha256'],name
p=json.loads((D/'execution/s2/corrections/n01_metric_docs_001/target/s2_n01_001/MANIFEST.json').read_text())
assert p['product_source']==run.identity()
for row in p['source_files']:
    f=run.ROOT/row['path']
    assert f.stat().st_size==row['bytes'] and run.sha(f)==row['sha256'],row['path']
i=json.loads((M/'PREDECESSOR_IMPACT.json').read_text())
assert len(i['conditions'])==88
print(json.dumps(dict(passed=True,source_sha256=c['source_sha256'],unchanged_source_files=len(p['source_files']),acceptances=12,integration=13,receipt_bindings=len(c['receipt_bindings']),inherited=88)))
