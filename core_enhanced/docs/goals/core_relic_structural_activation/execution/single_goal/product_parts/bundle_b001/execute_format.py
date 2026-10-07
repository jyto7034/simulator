#!/usr/bin/env python3
"""Owned-source format capture; refuses to change any cfg(test) item."""
import pathlib, subprocess, re, json, hashlib, datetime, sys
ROOT=pathlib.Path('/mnt/f/work/simulator'); OUT=pathlib.Path(__file__).resolve().parent
paths=['core_enhanced/src/application.rs', 'core_enhanced/src/game/ability/flow.rs', 'core_enhanced/src/game/ability/sustained.rs', 'core_enhanced/src/game/battle/core/basic_attack.rs', 'core_enhanced/src/game/battle/core/damage_runtime.rs', 'core_enhanced/src/game/battle/core/effective_delivery.rs', 'core_enhanced/src/game/battle/core/effective_skill.rs', 'core_enhanced/src/game/battle/core/mod.rs', 'core_enhanced/src/game/battle/core/relic_numeric.rs', 'core_enhanced/src/game/battle/core/runtime_facts.rs', 'core_enhanced/src/game/battle/core/sim.rs', 'core_enhanced/src/game/battle/core/skill_runtime/area.rs', 'core_enhanced/src/game/battle/core/skill_runtime/cast.rs', 'core_enhanced/src/game/battle/core/skill_runtime/flow/results.rs', 'core_enhanced/src/game/battle/core/skill_runtime/flow/structural.rs', 'core_enhanced/src/game/battle/core/skill_runtime/flow.rs', 'core_enhanced/src/game/battle/core/skill_runtime/projectile.rs', 'core_enhanced/src/game/battle/core/sustained/structural.rs', 'core_enhanced/src/game/battle/core/sustained.rs', 'core_enhanced/src/game/battle/core/target_usefulness.rs', 'core_enhanced/src/game/battle/core/types.rs', 'core_enhanced/src/game/battle/core/value_expr_runtime.rs', 'core_enhanced/src/game/battle/damage.rs', 'core_enhanced/src/game/battle/runtime_facts.rs', 'core_enhanced/src/game/behavior.rs', 'core_enhanced/src/game/data/skill_data.rs', 'core_enhanced/src/game/relic/structural/capabilities.rs', 'core_enhanced/src/game/relic/structural/compiler.rs', 'core_enhanced/src/game/relic/structural/dto.rs', 'core_enhanced/src/game/relic/structural/execution.rs', 'core_enhanced/src/game/relic/structural/publication.rs', 'core_enhanced/src/game/relic/structural/validation.rs', 'core_enhanced/src/game/relic/structural.rs', 'core_enhanced/src/game/relic.rs', 'core_enhanced/src/game/value_expr/catalog.rs', 'core_enhanced/src/game/world/action_capability.rs', 'core_enhanced/src/game/world/application.rs', 'core_enhanced/src/game/world/combat.rs', 'core_enhanced/src/game/world/relic_runtime.rs', 'core_enhanced/src/game/world.rs']
def masked(s):
 o=list(s);i=0
 while i<len(s):
  a=i;end=None
  if s.startswith('//',i): end=s.find('\n',i);end=len(s) if end<0 else end
  elif s.startswith('/*',i):
   depth=1;j=i+2
   while depth and j<len(s):
    if s.startswith('/*',j): depth+=1;j+=2
    elif s.startswith('*/',j): depth-=1;j+=2
    else:j+=1
   end=j
  else:
   raw=re.match(r'(?:br|r)(#*)"',s[i:])
   if raw:
    token='"'+raw.group(1);j=s.find(token,i+len(raw.group(0)));end=len(s) if j<0 else j+len(token)
   elif s[i]=='"':
    j=i+1
    while j<len(s):
     if s[j]=='\\':j+=2
     elif s[j]=='"':j+=1;break
     else:j+=1
    end=j
   elif s[i]=="'":
    char=re.match(r"""'(?:[^'\\\n]|\\(?:[nrt0\\'"]|x[0-9a-fA-F]{2}|u\{[0-9a-fA-F_]+\}))'""",s[i:])
    if char:end=i+len(char.group(0))
  if end is not None:
   for j in range(a,end):
    if o[j]!='\n':o[j]=' '
   i=end
  else:i+=1
 return ''.join(o)
def items(s):
 m=masked(s);out=[]
 for match in re.finditer(r'#\[cfg\([^\]]*\btest\b[^\]]*\)\]',m):
  start=match.start(); j=match.end()
  while j<len(m) and m[j] not in '{;':j+=1
  if j==len(m):raise RuntimeError('unterminated test item')
  if m[j]==';':end=j+1
  else:
   depth=1;end=j+1
   while depth:
    if m[end]=='{':depth+=1
    elif m[end]=='}':depth-=1
    end+=1
  out.append((start,end,s[start:end]))
 return out
def ref(p):
 b=p.read_bytes();return {'path':str(p.relative_to(ROOT)),'bytes':len(b),'sha256':hashlib.sha256(b).hexdigest()}
label=sys.argv[1]; dest=OUT/label;assert not dest.exists();dest.mkdir()
argv=['/home/blast/.cargo/bin/rustfmt','--edition','2021','--config','skip_children=true','--emit','stdout']
rows=[];started=datetime.datetime.now(datetime.timezone.utc).isoformat()
for n,name in enumerate(paths):
 p=ROOT/name;before=ref(p);source=p.read_text();tests=items(source)
 r=subprocess.run(argv,input=source,text=True,capture_output=True)
 raw=dest/(str(n)+'.stdout');raw.write_text(r.stdout)
 err=dest/(str(n)+'.stderr');err.write_text(r.stderr)
 assert r.returncode==0,(name,r.stderr)
 formatted=r.stdout;other=items(formatted);assert len(tests)==len(other)
 for original,changed in reversed(list(zip(tests,other))):
  formatted=formatted[:changed[0]]+original[2]+formatted[changed[1]:]
 assert [x[2] for x in items(formatted)]==[x[2] for x in tests]
 p.write_text(formatted)
 rows.append({'path':name,'before':before,'after':ref(p),'cfg_test_items':len(tests),'cfg_test_bytes_unchanged':True,'stdin_path':name,'stdout':ref(raw),'stderr':ref(err),'exit_code':r.returncode})
result={'label':label,'command':argv,'actual_started_at_utc':started,'actual_closed_at_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'files':rows,'exit_code':0,'test_module_recursion':False}
(dest/'RESULT.json').write_text(json.dumps(result,indent=2)+'\n')
print(json.dumps({'result':ref(dest/'RESULT.json'),'exit_code':0,'owned_paths':len(paths)}))
