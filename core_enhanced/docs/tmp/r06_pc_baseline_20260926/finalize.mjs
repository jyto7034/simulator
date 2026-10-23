import fs from 'node:fs';
import path from 'node:path';
import crypto from 'node:crypto';
import {spawnSync} from 'node:child_process';
const dir=path.dirname(new URL(import.meta.url).pathname);
const m=JSON.parse(fs.readFileSync(path.join(dir,'source_identity.json')));
const hashBytes=b=>crypto.createHash('sha256').update(b).digest('hex');
const hook='\n    #[path = "/mnt/f/work/simulator/core_enhanced/docs/tmp/r06_pc_baseline_20260926/probe.rs"]\n    mod load_baseline;\n';
const getter='\n#[cfg(test)]\npub(crate) fn baseline_usage() -> Option<(u64, usize, Option<usize>)> {\n    ACTIVE.with(|slot| slot.borrow().as_ref().map(|b| (b.work, b.clone_slots, b.retained_admission)))\n}\n';
let patch='';
for(const f of m.files){
  const original=fs.readFileSync(path.join(m.root,f.path));
  if(hashBytes(original)!==f.sha256)throw Error('working source drift: '+f.path);
  const copy=fs.readFileSync(path.join(m.sandbox,f.path));
  if(hashBytes(copy)!==f.sha256){
    const addition=f.path.endsWith('/value_expr_runtime.rs')?hook:f.path.endsWith('/runtime_control.rs')?getter:null;
    if(!addition)throw Error('unexpected copy change: '+f.path);
    const text=copy.toString();
    if(text.split(addition).length!==2 || hashBytes(text.replace(addition,''))!==f.sha256)throw Error('non-instrumentation delta: '+f.path);
    const d=spawnSync('diff',['-u','--label',`a/${f.path}`,'--label',`b/${f.path}`,path.join(m.root,f.path),path.join(m.sandbox,f.path)],{encoding:'utf8'});
    if(d.status!==1)throw Error('diff failed');patch+=d.stdout;
  }
}
fs.writeFileSync(path.join(dir,'instrumentation.patch'),patch,{flag:'wx'});
const bin=fs.readdirSync(path.join(m.sandbox,'target/release/deps')).filter(p=>/^game_core-[a-f0-9]+$/.test(p));
const entries=fs.readdirSync(dir).filter(p=>fs.statSync(path.join(dir,p)).isFile()).sort().map(p=>({path:p,bytes:fs.statSync(path.join(dir,p)).size,sha256:hashBytes(fs.readFileSync(path.join(dir,p)))}));
fs.writeFileSync(path.join(dir,'ARTIFACTS.json'),JSON.stringify({date:new Date().toISOString(),
  statement:'Current 294 inputs unchanged; temporary copy differs only by two exact diagnostic additions. Not a stage-review PASS.',
  verified_files:m.files.length,binaries:bin.map(p=>({path:path.join(m.sandbox,'target/release/deps',p),sha256:hashBytes(fs.readFileSync(path.join(m.sandbox,'target/release/deps',p)))})),files:entries},null,2)+'\n',{flag:'wx'});
console.log(`Verified ${m.files.length} current/copy input files and saved artifact hashes.`);
