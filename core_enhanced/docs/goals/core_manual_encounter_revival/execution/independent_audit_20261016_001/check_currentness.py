"""Audit exit check against the two immutable manifests and reviewed overlay."""
from pathlib import Path
import hashlib
import json
import subprocess

A=Path(__file__).resolve().parent
E=A.parent
R=next(p for p in E.parents if (p/'game_server/Cargo.toml').is_file())
load=lambda p:json.loads(p.read_bytes())
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
source,evidence={},{}
refs=[]
entry=load(A/'ENTRY_CURRENTNESS.json')
freeze=load(A/'FREEZE_VERIFICATION.json')
for stage,tid in [('single_goal','u17_current_001'),('storage_refactor','u17_storage_001')]:
    t=E/stage/'target'/tid
    assert sha(t/'AUDIT_TARGET_RECORD.json')==entry[stage]['record_sha256']
    for name,digest in freeze[tid]['archive_hashes'].items():
        assert sha(t/name)==digest,(tid,name)
    source.update({r['path']:r for r in load(t/'SOURCE_MANIFEST.json')['files']})
    evidence.update({r['path']:r for r in load(t/'EVIDENCE_MANIFEST.json')['files']})
    refs.append(dict(path=str(t.relative_to(R)),record_sha256=entry[stage]['record_sha256'],
        source_manifest_sha256=sha(t/'SOURCE_MANIFEST.json'),evidence_manifest_sha256=sha(t/'EVIDENCE_MANIFEST.json')))
roadmap='core_enhanced/docs/roadmaps/core_first/ROADMAP.ko.md'
# This exact copy is made once before the exit check; it is the reviewed roadmap
# from ROADMAP_DELTA.diff, not a silently rewritten historical target member.
roadmap_copy=A/'ROADMAP_REVIEWED.ko.md'
assert roadmap_copy.is_file()
source[roadmap]=dict(path=roadmap,bytes=roadmap_copy.stat().st_size,sha256=sha(roadmap_copy))
for label,rows in [('source',source),('evidence',evidence)]:
    mismatches=[]
    for path,row in rows.items():
        p=R/path
        if not p.is_file() or p.stat().st_size!=row['bytes'] or sha(p)!=row['sha256']:
            mismatches.append(path)
    assert not mismatches,(label,mismatches)
roots=['core_enhanced/src','core_enhanced/tests','game_server/src','game_resources/data']
actual={str(p.relative_to(R)) for root in roots for p in (R/root).rglob('*') if p.is_file()}
expected={p for p in source if any(p.startswith(root+'/') for root in roots)}
assert actual==expected,dict(added=sorted(actual-expected),missing=sorted(expected-actual))
prerequisite={}
for suffix in ['ko.md','json']:
    p=R/('core_enhanced/docs/roadmaps/core_first/execution/u16_final_audit_20261015_001/CURRENT_DISPOSITION.'+suffix)
    prerequisite[str(p.relative_to(R))]=sha(p)
result=dict(passed=True,currentness='current / unchanged relative to reviewed composite audit target',
    source_members=len(source),evidence_members=len(evidence),product_source_members=len(actual),
    target_records=refs,roadmap_overlay=dict(path=roadmap,sha256=source[roadmap]['sha256'],
        reason='approved storage follow-up registration/status only; original U17 requirements unchanged; new follow-up requirements audited'),
    prerequisite_disposition=prerequisite,
    head=subprocess.check_output(['git','rev-parse','HEAD'],cwd=R,text=True).strip(),
    product_writes_by_auditor=0,exclusions=['audit-local files/output','mutable HANDOFF/EXECUTION_HISTORY/ROADMAP_STATUS','private issuer keys','unrelated workspace/cache'])
assert result['head']==entry['head']
with (A/'EXIT_CURRENTNESS.json').open('x') as f:
    json.dump(result,f,ensure_ascii=False,indent=2);f.write('\n')
print(json.dumps(result,ensure_ascii=False,indent=2))
