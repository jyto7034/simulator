from pathlib import Path
import hashlib, json, tarfile, re
HERE = Path(__file__).resolve().parent
CFG = json.loads((HERE / 'config.json').read_text())
ROOT, SEG = Path(CFG['root']), Path(CFG['segment'])
def sha(b): return hashlib.sha256(b).hexdigest()
def check_record(rec):
    b = (ROOT / rec['path']).read_bytes()
    assert len(b) == rec['bytes'] and sha(b) == rec['sha256'], rec['path']
mp = SEG / 'target/action_result_snapshot_001/MANIFEST.json'
assert sha(mp.read_bytes()) == CFG['target_sha256']
m = json.loads(mp.read_text())
expected = {f['path']: f['after'] for f in m['source']['files']}
assert len(expected) == len(m['source']['files']) == 333
assert CFG['source'] == m['source']['files']
for path, rec in expected.items(): check_record(dict(path=path, **rec))
for rec in [m['baseline'], m['predecessor'], m['after_archive']] + m['records'] + m['artifacts']: check_record(rec)
with tarfile.open(ROOT / m['after_archive']['path']) as t:
    members = [x for x in t.getmembers() if x.isfile()]
    assert len(members) == len(expected) and {x.name for x in members} == set(expected)
    for x in members:
        b = t.extractfile(x).read()
        assert dict(bytes=len(b), sha256=sha(b)) == expected[x.name]
expected_rows = sorted([[p, r['bytes'], r['sha256']] for p,r in expected.items()])
evidence = SEG / 'evidence'
receipts = {}
for tag in ['check_after_heap_001','focused_after_heap_001','all_targets_box_001','clippy_box_001','format_after_heap_001','r05_guard_box_001']:
    d = json.loads((evidence / (tag + '.json')).read_text())
    assert d['exit_code'] == 0
    assert d['source_before'] == d['source_after']
    assert sorted(d['source_before']['files']) == expected_rows
    assert d['source_before']['digest'] == m['source']['after_digest']
    assert sha((evidence / (tag + '.log')).read_bytes()) == d['log_sha256']
    receipts[tag] = dict(exit_code=d['exit_code'], test_passed=d['test_passed'], digest=d['source_before']['digest'])
assert receipts['all_targets_box_001']['test_passed'] == 1491
pc = json.loads((evidence / 'pc_isolate_002.json').read_text())
run = json.loads((evidence / 'pc_box_001.json').read_text())
assert pc['current_source'] == pc['copied_source']
assert sorted(pc['copied_source']['files']) == expected_rows
assert pc['built_source'] == run['source_before'] == run['source_after']
assert run['cwd'] == run['source_dir'] == pc['isolate'] and run['exit_code'] == 0
assert sha((evidence / 'pc_box_001.log').read_bytes()) == run['log_sha256']
direct = json.loads((HERE / 'focused_001.json').read_text())
assert direct['exit_code'] == 0 and direct['source_before'] == direct['source_after'] == {p:r['sha256'] for p,r in expected.items()}
assert direct['workspace_before'] == direct['workspace_after'] == direct['source_before']
assert re.search(r'test result: ok[.] 30 passed; 0 failed', (HERE / 'focused_001.log').read_text())
summary = dict(target_sha256=CFG['target_sha256'], source_files=333, archive_exact=True, currentness=True,
               records_verified=len(m['records']), artifacts_verified=len(m['artifacts']), final_receipts=receipts,
               pc_copied_and_built_binding=True, reviewer_focused_passed=30)
(HERE / 'verification.json').write_text(json.dumps(summary, indent=2)+'\n')
print(json.dumps(summary))
