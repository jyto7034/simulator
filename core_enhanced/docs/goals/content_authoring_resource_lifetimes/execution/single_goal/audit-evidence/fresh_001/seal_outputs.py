import datetime, json, re
from verify_currentness import ROOT, BASE, OUT, TARGET, EXPECTED, digest

report=BASE/'COMPLETION_AUDIT_001.md'
pointer=BASE/'COMPLETION_AUDIT.md'
registry=OUT/'AUDIT_OUTPUTS.json'
read=lambda p:json.loads(p.read_text())
assert digest(TARGET)==EXPECTED
assert read(OUT/'ENTRY_CURRENTNESS.json')['pass']
assert read(OUT/'CLOSURE_CURRENTNESS.json')['pass']
assert read(OUT/'EVIDENCE_VERIFIED_ENTRY.json')['passed']
assert read(OUT/'EVIDENCE_CLOSURE.json')['passed']
assert len(re.findall(r'^\| E\d\d\[\d\] \|',report.read_text(),re.M))==33
assert '**Verdict: Not Complete.**' in report.read_text()
paths=[report,pointer]+sorted(p for p in OUT.rglob('*') if p.is_file() and p!=registry)
assert len(paths)==len(set(paths))
manifest_paths={r['path'] for r in read(TARGET)['manifest']}
outputs=[{'path':str(p.relative_to(ROOT)),'bytes':p.stat().st_size,'sha256':digest(p)} for p in paths]
assert all(row['path'] not in manifest_paths for row in outputs)
r={'record':'R05-FRESH-AUDIT-OUTPUTS-001',
   'sealed_at_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),
   'reviewer':'/root/r05_fresh_completion_audit; separate same-model, no implementation participation or conversation fork',
   'verdict':'Not Complete','finding':'F-R05-001','runtime_panic_reproduced':False,
   'target':str(TARGET.relative_to(ROOT)),'target_sha256':EXPECTED,
   'report':str(report.relative_to(ROOT)),'report_sha256':digest(report),
   'entry_and_closure':'current / unchanged',
   'required_predecessor_supplements':'all four exact target/report/registry identities and 47 registered outputs verified at entry and closure',
   'source_mutations_by_auditor':False,'new_builds_or_gameplay_runs':False,
   'excluded_self':str(registry.relative_to(ROOT)),
   'outputs':outputs}
registry.write_text(json.dumps(r,indent=2)+'\n')
for row in outputs:
    p=ROOT/row['path']
    assert digest(p)==row['sha256'] and p.stat().st_size==row['bytes']
assert {p for p in OUT.rglob('*') if p.is_file() and p!=registry}==set(paths)-{report,pointer}
print(json.dumps({'verdict':r['verdict'],'outputs':len(outputs),'target_sha256':EXPECTED,
    'report_sha256':digest(report),'registry_sha256':digest(registry)},indent=2))
