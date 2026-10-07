"""Prepare a reviewable correction; never mutate product files while validation runs."""
import difflib,json,re
from run import ROOT,HERE,identity,sha
out=HERE/'evidence/accounting_dedup_correction_001';before=identity();rows=[];patch=[]
paths=['independent_objects.rs','object_attack.rs','object_creation.rs','object_evolution.rs','object_query.rs','spawn_kernel.rs','sustained/segmented.rs']
regex=r'(?:crate::authoring::control|crate::game::battle::runtime_control|runtime_control|control|native)::charge\([\s\S]*?\)\?;'
for local in paths:
 path='core_enhanced/src/game/battle/core/'+local;p=ROOT/path;s=p.read_text();hits=list(re.finditer(regex,s));removals=[];sites=[]
 for a,b in zip(hits,hits[1:]):
  if s[a.end():b.start()].strip():continue
  aa=re.findall(r'"([^"]+)"',a.group());bb=re.findall(r'"([^"]+)"',b.group())
  def shared(m):return m.group().startswith(('crate::authoring::control::','control::'))
  if aa and aa==bb and shared(a)!=shared(b):
   # Assert that the full units expression is identical, including casts.
   args=lambda m:re.sub(r'\s+','',m.group().split('::charge(',1)[1]).replace(',)?;',')?;');assert args(a)==args(b),(path,aa)
   m=b if shared(a) else a;removals.append((s.rfind('\n',0,m.start())+1,m.end()+1));sites+=aa
 for a,b in reversed(removals):s=s[:a]+s[b:]
 if local=='object_attack.rs':
  a=s.index('        let cells = pattern');b=s.index('        crate::authoring::control::charge(self.units.len()',a)
  s=s[:a]+'''        if !crate::authoring::control::is_active() {
            runtime_control::charge(pattern.rows.len() as u64, "trap_pattern_rows")?;
            let cells = pattern
                .rows
                .iter()
                .try_fold(0u64, |n, row| n.checked_add(row.len() as u64 + 1));
            // Membership revalidates the pattern and scans its anchor/cells.
            // Six visits per byte bound those scans, without a second native
            // debit when the shared authoring projection meter already ran.
            let work = cells
                .and_then(|n| n.checked_mul(self.units.len() as u64 + 1))
                .and_then(|n| n.checked_mul(6))
                .ok_or(GameError::InvalidAction)?;
            runtime_control::charge(work, "trap_pattern_scan")?;
        }
'''+s[b:]
 if s!=p.read_text():
  for folder,text in [('before',p.read_text()),('after',s)]:q=out/folder/path;q.parent.mkdir(parents=True,exist_ok=True);q.write_text(text)
  rows.append({'path':path,'before_sha256':sha(p),'after_sha256':sha(out/'after'/path),'deduplicated_sites':sites})
  patch+=difflib.unified_diff(p.read_text().splitlines(keepends=True),s.splitlines(keepends=True),fromfile='a/'+path,tofile='b/'+path)
assert sum(len(r['deduplicated_sites']) for r in rows)==19
(out/'PREPARED.patch').write_text(''.join(patch))
(out/'SOURCE_BEFORE.json').write_text(json.dumps(before,indent=2)+'\n')
(out/'PREPARED.json').write_text(json.dumps({'source_before':before['sha256'],'files':rows,'patch_sha256':sha(out/'PREPARED.patch'),'claim':'Nineteen same-units/same-site native scan debits are already forwarded by the retained shared charge. Remove only the duplicate native call. Trap pattern helper forwards native in authoring scope; native-only path separately preprices row traversal and bounded membership scans. No limits, authored rules, state mutations or query payloads change.','status':'prepared, not applied'},indent=2)+'\n')
print('Prepared19 duplicate-debit removals and native-only trap membership preflight')
