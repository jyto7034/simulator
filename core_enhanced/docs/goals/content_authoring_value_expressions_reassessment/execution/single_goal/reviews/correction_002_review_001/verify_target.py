"""Read-only currentness, exact archive/delta and historical recovery checks."""
from pathlib import Path
import json, hashlib, tarfile, importlib.util, contextlib, io
from datetime import datetime, timezone

ROOT = Path('/mnt/f/work/simulator')
TOPIC = ROOT / 'core_enhanced/docs/goals/content_authoring_value_expressions_reassessment'
BASE = TOPIC / 'execution/single_goal/corrections/correction_001_r01_r03'
TARGET = BASE / 'target/correction_002/MANIFEST.json'
sha = lambda data: hashlib.sha256(data).hexdigest()
assert sha(TARGET.read_bytes()) == '1bedd79cf745bb489db18172b3c29bc3477f728fa9849d2b4e36f357224ff85a'
m = json.loads(TARGET.read_text())
b = json.loads((ROOT / m['pre_fix_baseline']['manifest']).read_text())
spec = importlib.util.spec_from_file_location('implementer_verifier', TOPIC / 'authoring_scripts/r06_verify_correction_002_target.py')
v = importlib.util.module_from_spec(spec)
spec.loader.exec_module(v)
out = io.StringIO()
with contextlib.redirect_stdout(out):
    v.main()
checks = []
def scan(value):
    if isinstance(value, dict):
        for pk, hk in [('path','sha256'),('manifest','manifest_sha256'),('receipt','receipt_sha256')]:
            if pk in value and hk in value:
                assert sha((ROOT / value[pk]).read_bytes()) == value[hk], value[pk]
                checks.append(value[pk])
        for item in value.values():
            scan(item)
    elif isinstance(value,list):
        for item in value:
            scan(item)
scan(m)
archives = []
for label, section in [('source_before',b['source']),('source_after',m['current_source']),('documents_before',b['mutable_documents']),('documents_after',m['documents']),('evidence_after',m['evidence']),('predecessor_records',b['predecessor_execution_records']),('review_inputs',b['review_inputs'])]:
    archive_row = section['archive']
    assert sha((ROOT / archive_row['path']).read_bytes()) == archive_row['sha256']
    rows = {r['path']:r for r in section['files']}
    with tarfile.open(ROOT / archive_row['path'],'r:gz') as archive:
        members = [f for f in archive.getmembers() if f.isfile()]
        assert len(members) == len(rows) == len({f.name for f in members}), label
        assert {f.name for f in members} == set(rows), label
        for f in members:
            data = archive.extractfile(f).read()
            assert sha(data) == rows[f.name]['sha256'], (label,f.name)
            assert len(data) == rows[f.name]['bytes'], (label,f.name)
    archives.append({'label':label,'verified_files':len(rows)})
deltas = {}
for label, before, after in [('source',b['source'],m['current_source']),('documents',b['mutable_documents'],m['documents'])]:
    old = {r['path']:r['sha256'] for r in before['files']}
    new = {r['path']:r['sha256'] for r in after['files']}
    actual = {'added':sorted(new.keys()-old.keys()),'deleted':sorted(old.keys()-new.keys()),'modified':sorted(p for p in old.keys()&new.keys() if old[p]!=new[p])}
    declared = after['changes_from_pre_fix']
    for kind, paths in actual.items():
        assert declared[kind] == len(paths), (label,kind)
        assert set(paths) == {r['path'] for r in declared['files'] if r['state']==kind}, (label,kind)
    deltas[label] = actual
predecessor = json.loads((ROOT / m['predecessor_target']['manifest']).read_text())
assert b['source']['files'] == predecessor['current_source']['files']
historical = m['r03_recovery']['historical_plan']
assert historical['bytes'] == 4460
assert historical['sha256'] == '661dd456d923e323cfd5704dd520e322251ba9a603303a4b73c295e6c13412c7'
print(json.dumps({'checked_at':datetime.now(timezone.utc).isoformat(),'target':m['record'],'manifest_sha256':sha(TARGET.read_bytes()),'reference_hash_checks':len(checks),'source_files':len(m['current_source']['files']),'document_files':len(m['documents']['files']),'evidence_files':len(m['evidence']['files']),'execution_records':len(m['execution_records']),'archives':archives,'deltas':deltas,'r03_recovered_bytes':historical['bytes'],'historical_baseline_unchanged':True,'implementer_verifier':out.getvalue().strip(),'passed':True},ensure_ascii=False,indent=2))
