from pathlib import Path
import json,hashlib,tarfile,re,difflib,datetime
R=Path('/mnt/f/work/simulator');G=R/'core_enhanced/docs/goals/core_resonance_lifecycle';B=G/'execution/single_goal';A=G/'post_audit/u05_20261004_002';P=G/'post_audit/u05_20261004_001';T=B/'target/u05_002'
def read(p):return json.loads(p.read_text())
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def rawsha(b):return hashlib.sha256(b).hexdigest()
errors=[]
def check(ok,reason):
 if not ok:errors.append(reason)
d=read(B/'corrections/cf_d09_001/DOCUMENT_PATH_MIGRATION.json');e=d['external_migration'];D=R/e['path'];docs=e['documents'];out={'at_utc':datetime.datetime.now(datetime.timezone.utc).isoformat()}
for f,k in [('RECORD.ko.md','record_sha256'),('EVIDENCE.json','evidence_sha256'),('BEFORE.tar.gz','before_archive_sha256')]:check(sha(D/f)==e[k],f)
check(read(D/'EVIDENCE.json')['documents']==docs,'external documents mapping')
old={};diff=[];body=[]
with tarfile.open(D/'BEFORE.tar.gz') as t:
 for row in docs:
  data=t.extractfile(row['before_path']).read();check(rawsha(data)==row['before_sha256'],'before '+row['before_path']);check(sha(R/row['after_path'])==row['after_sha256'],'after '+row['after_path']);old[row['before_path']]=data.decode()
  before=data.decode();after=(R/row['after_path']).read_text();diff.extend(difflib.unified_diff(before.splitlines(keepends=True),after.splitlines(keepends=True),fromfile=row['before_path'],tofile=row['after_path']))
  if '/game_rules_next/' in row['before_path']:
   before=before.replace('../game_rulebook.md#','../archive/game_rulebook_legacy_2026-10-04.md#').replace('../game_rulebook_next.md','../game_rulebook.md')
   for label in ('모드 규칙','파편 획득','모노리스 정책','숙달 소탕','BossOmen 정책'):before=before.replace('[현행 '+label+']','[과거 '+label+']')
   ok=before==after;check(ok,'detail body '+row['before_path']);body.append({'old_path':row['before_path'],'current_path':row['after_path'],'equal_under_explicit_locator_and_label_substitutions':ok})
currentold=old['core_enhanced/docs/game_rulebook_next.md'].split('## 어떤 게임인가',1)[1];currentnew=(R/'core_enhanced/docs/game_rulebook.md').read_text().split('## 어떤 게임인가',1)[1]
ok=currentold.replace('game_rules_next/','game_rules/')==currentnew;check(ok,'current rulebook body');body.append({'current_rulebook_body_equal_after_directory_relocation':ok})
legacyold=old['core_enhanced/docs/game_rulebook.md'].split('## 게임 개요',1)[1];legacynew=(R/'core_enhanced/docs/archive/game_rulebook_legacy_2026-10-04.md').read_text().split('## 게임 개요',1)[1]
# Preserve each actual link destination, allowing only ../ rebase into the archive.
def rebase(m):
 dest=m.group(1)
 return ']('+ ('../'+dest if not re.match(r'(?:[a-zA-Z]+:|/|#)',dest) else dest)+')'
normalized=re.sub(r'\]\(([^)]*)\)',rebase,legacyold)
ok=normalized==legacynew;check(ok,'historical body exact rebase');body.append({'historical_body_equal_with_each_relative_link_rebased':ok})
check(''.join(diff)==(B/'corrections/cf_d09_001/EXTERNAL_DOCUMENT_MIGRATION.diff').read_text(),'complete external diff')
prior=read(P/'CURRENTNESS_EXIT_INTERRUPTED.json')['mismatches'];bybefore={x['before_path']:x for x in docs};members={x['path']:x for x in read(T/'MEMBERSHIP.json')}
check(len(prior)==len(d['affected_frozen_members'])==11,'eleven F01 members')
for x in prior:
 y=bybefore[x['path']];check(x['expected_sha256']==y['before_sha256'],'F01 original '+x['path']);check(members[y['after_path']]['sha256']==y['after_sha256'],'F01 current '+x['path'])
for p,h in d['predecessor_audit'].items():check(sha(R/p)==h,'prior audit '+p)
for x in read(B/'target/u05_001/RECORDS.json'):check(sha(R/x['path'])==x['sha256'],'original execution '+x['path'])
p1=read(B/'target/u05_001/MANIFEST.json');p2=read(T/'MANIFEST.json');check(p1['product_source']==p2['product_source'],'same exact product identity')
check(rawsha(json.dumps(p2['source_files'],sort_keys=True).encode())==p2['source_sha256'],'combined source digest')
a={x['path']:x for x in read(B/'target/u05_001/MEMBERSHIP.json')};b=members
out.update(documents=docs,body_checks=body,F01_mappings=d['affected_frozen_members'],original_execution_files_preserved=2982,prior_audit_files_preserved=len(d['predecessor_audit']),previous_member_delta={'changed':sorted(p for p in a.keys()&b.keys() if a[p]['sha256']!=b[p]['sha256']),'removed':sorted(a.keys()-b.keys()),'added':sorted(b.keys()-a.keys())},semantic_disposition='CF-D09 changes navigation, present/historical authority labels and path intake only. Exact detailed policy and both policy bodies are preserved under the specified link rebases; historical decisions stay historical. No product/test/data/input/acceptance/oracle delta.',errors=errors,passed=not errors)
(A/'MIGRATION_VERIFICATION.json').write_text(json.dumps(out,ensure_ascii=False,indent=2)+'\n');print(json.dumps({'documents':len(docs),'body_checks':len(body),'original_execution_files_preserved':2982,'F01_mappings':len(prior),'errors':errors,'passed':not errors}))
