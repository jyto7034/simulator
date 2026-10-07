"""Document-only publication successor, with original sealed evidence preserved."""
from pathlib import Path
import json,hashlib,tarfile,tempfile,difflib,subprocess,sys,re
ROOT=Path('/mnt/f/work/simulator');P=Path(__file__).resolve().parent;H=P.parents[1]/'reviews/r08_r12_20261002_001'
sys.path.insert(0,str(H));import run
sha=run.sha
sealed=json.loads((H/'AUDIT_CLOSURE.json').read_text());assert sha(ROOT/sealed['manifest']['path'])==sealed['manifest']['sha256']
for r in json.loads((H/'AUDIT_MANIFEST.json').read_text())['files']:assert sha(ROOT/r['path'])==r['sha256']
entry=json.loads((H/'AUDIT_TARGET.json').read_text())['current'];actual=run.identity()
a={r[0]:r for r in entry['files']};b={r[0]:r for r in actual['files']}
assert a.keys()==b.keys()
changed=sorted(n for n in a if a[n]!=b[n]);assert changed==['core_enhanced/docs/roadmaps/content_authoring/ROADMAP_STATUS.md']
old=json.loads((P/'STATUS_BEFORE.json').read_text());rows=[]
with tarfile.open(P/'STATUS_AFTER.tar.gz','x:gz') as archive:
 for r in old:
  p=ROOT/r['path'];archive.add(p,arcname=r['path'],recursive=False);rows.append(dict(path=r['path'],bytes=p.stat().st_size,sha256=sha(p)))
with tempfile.TemporaryDirectory(prefix='r08-r12-audit-status-') as temp:
 dest=Path(temp)
 with tarfile.open(P/'STATUS_BEFORE.tar.gz') as archive:archive.extractall(dest,filter='data')
 patch=[]
 for r in old:
  n=r['path'];assert sha(dest/n)==r['sha256']
  patch.extend(difflib.unified_diff((dest/n).read_text().splitlines(True),(ROOT/n).read_text().splitlines(True),fromfile='a/'+n,tofile='b/'+n))
 (P/'STATUS_FORWARD.patch').write_text(''.join(patch))
 for reverse,expected in [(False,rows),(True,old)]:
  p=subprocess.run(['patch',*(['-R'] if reverse else []),'-p1','--fuzz=0','--batch','--input',str(P/'STATUS_FORWARD.patch')],cwd=dest,capture_output=True)
  assert p.returncode==0,p.stderr
  for r in expected:assert sha(dest/r['path'])==r['sha256']
  (P/('STATUS_REVERSE.log' if reverse else 'STATUS_FORWARD.log')).write_bytes(p.stdout+p.stderr)
report=P/'COMPLETION_AUDIT.md';assert entry['digest'] in report.read_text()
result=dict(kind='post-audit document-only publication and narrative digest correction',audit_closure=str((H/'AUDIT_CLOSURE.json').relative_to(ROOT)),audit_closure_sha256=sha(H/'AUDIT_CLOSURE.json'),original_audit_manifest_preserved=True,original_audit_records_preserved=sealed['manifest']['file_count'],verdict='Not Complete',blocking_findings=['F01'],audited_source_digest=entry['digest'],current_source_digest_after_status_publication=actual['digest'],source_membership_unchanged=True,source_changed_members=changed,source_nonimpact='ROADMAP_STATUS describes the already-issued Not Complete verdict; all other 386 audit-source members unchanged',before_status_documents=old,after_status_documents=rows,status_forward_reverse_isolated_fuzz0=True,narrative_correction=dict(original=str((H/'COMPLETION_AUDIT.md').relative_to(ROOT)),original_preserved=True,wrong_current_digest='a1289812bb0083a419c0767a5f157317e67e4c7602c368b230f5193f455eec8b',correct_audited_digest=entry['digest'],basis='AUDIT_TARGET, all command receipts and AUDIT_CLOSURE already agree on the correct digest; only narrative copied a stale preparation digest',successor=str(report.relative_to(ROOT)),successor_sha256=sha(report)),product_code_or_data_modified=False,r13_r16_started=False,r13_r16_final_target_frozen=False)
with (P/'PUBLICATION_RECORD.json').open('x') as f:json.dump(result,f,ensure_ascii=False,indent=2);f.write('\n')
for p in [report,ROOT/'core_enhanced/docs/goals/content_authoring_r13_r16_runtime/PLAN.md']:
 for link in re.findall(r'\]\(([^)]+)\)',p.read_text()):
  if '://' not in link:assert (p.parent/link.split('#')[0]).exists(),(p,link)
print(json.dumps(dict(verdict='Not Complete',audited_digest=entry['digest'],published_digest=actual['digest'],only_source_delta=changed,previous_audit_seal_intact=True,status_patch_forward_reverse=True,links_valid=True)))
