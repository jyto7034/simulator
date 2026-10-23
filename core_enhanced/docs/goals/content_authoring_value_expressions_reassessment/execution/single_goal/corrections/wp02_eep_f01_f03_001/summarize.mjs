import fs from 'node:fs';
import path from 'node:path';
import crypto from 'node:crypto';
const root='/mnt/f/work/simulator';
const here=path.dirname(new URL(import.meta.url).pathname);
const dir=here+'/evidence';
const hash=b=>crypto.createHash('sha256').update(b).digest('hex');
const read=p=>JSON.parse(fs.readFileSync(p));
const preserve=(name,data)=>{const p=dir+'/'+name;if(fs.existsSync(p)){if(!fs.readFileSync(p).equals(Buffer.from(data)))throw Error('existing evidence differs '+name);}else fs.writeFileSync(p,data,{flag:'wx'});};
const write=(name,value)=>preserve(name,JSON.stringify(value,null,2)+'\n');
const identity=read(dir+'/isolate_identity.json');
const adapter=identity.instrumented[0],native=identity.instrumented[1];
for(const f of identity.source){
 if(hash(fs.readFileSync(root+'/'+f.path))!==f.sha256)throw Error('current drift '+f.path);
 if(!identity.instrumented.includes(f.path)&&hash(fs.readFileSync(identity.dir+'/'+f.path))!==f.sha256)throw Error('isolate drift '+f.path);
}
const expectedAdapter=fs.readFileSync(root+'/'+adapter,'utf8').trimEnd().slice(0,-1)+`\n#[path = "${identity.dir}/review_probes.rs"] mod review_probes;\n#[path = "${identity.dir}/pc_probe.rs"] mod load_baseline;\n}\n`;
if(fs.readFileSync(identity.dir+'/'+adapter,'utf8')!==expectedAdapter)throw Error('adapter hook drift');
const expectedNative=fs.readFileSync(root+'/'+native,'utf8')+'\n#[cfg(test)]\npub(crate) fn baseline_usage() -> Option<(u64, usize, Option<usize>)> {\n ACTIVE.with(|slot| slot.borrow().as_ref().map(|b| (b.work, b.clone_slots, b.retained_admission)))\n}\n';
if(fs.readFileSync(identity.dir+'/'+native,'utf8')!==expectedNative)throw Error('native hook drift');
const hooks={};
for(const name of ['review_probes.rs','pc_probe.rs']){
 const data=fs.readFileSync(identity.dir+'/'+name);preserve(name,data);hooks[name]=hash(data);
}
write('isolate_final_identity.json',{state:'333_SOURCE_FILES_MATCH_EXCEPT_EXACT_TWO_TEST_HOOKS',source:identity.source,hooks,initial_probe_fixture_unwrap_corrected:true});
for(const name of ['all_tests_002','check_final','clippy_final','changed_format','review_probes_002','pc_001','r05_guard_001']){
 const receipt=read(dir+'/'+name+'.json');if(receipt.exit_code!==0)throw Error('validation failed '+name);
 if(JSON.stringify(receipt.source_before)!==JSON.stringify(receipt.source_after))throw Error('source changed during '+name);
 for(const f of receipt.source_after)if(hash(fs.readFileSync(root+'/'+f.path))!==f.sha256)throw Error('stale '+name);
}
const raw=read(dir+'/pc_raw.json');
const key=r=>JSON.stringify({family:r.family,n:r.enemies??r.members??r.target_equivalent,chunk:r.chunk_ms,sources:r.burst_sources??r.source_equivalent??r.repetitions,reads:r.reads_per_expr,grouped:r.grouped_per_tile});
function summarize(data){
 const groups=new Map();for(const r of data.rows.filter(r=>!r.warmup)){const k=key(r);if(!groups.has(k))groups.set(k,[]);groups.get(k).push(r);}
 return [...groups].map(([k,rows])=>({...JSON.parse(k),errors:[...new Set(rows.map(r=>r.error??r.setup_error).filter(Boolean))],example:rows[0]}));
}
const predecessor=here+'/../../segments/wp02_effect_execution_prefix_001/evidence/performance/pc_current_002/pc_current_raw.json';
const before=summarize(read(predecessor)),after=summarize(raw);
const signature=rows=>rows.filter(r=>r.errors.length).map(r=>JSON.stringify({...r,example:undefined}));
const comparison={conditions:after.length,before:{completed:before.filter(r=>!r.errors.length).length,refused:before.filter(r=>r.errors.length).length},after:{completed:after.filter(r=>!r.errors.length).length,refused:after.filter(r=>r.errors.length).length},refusal_identity_equal:JSON.stringify(signature(before))===JSON.stringify(signature(after)),summary:after};
if(comparison.conditions!==36||comparison.after.completed!==31||!comparison.refusal_identity_equal)throw Error('PC regression '+JSON.stringify(comparison));
write('PC_COMPARISON.json',comparison);
const r05=read(dir+'/r05_guard_raw.json');
const guard={original_concurrent_cap:9030188,allocation_observer_enabled:r05.allocation_observer_enabled};
for(const k of ['event_heavy','concurrent_sessions']){
 const v=r05[k];guard[k]={peak:Math.max(...v.allocation_raw_samples.map(x=>x.heap_allocation.peak_delta_bytes)),guard:v.allocation_retained_peak_guard_pass,parity:v.deterministic_baseline_parity,repeatable:v.repeatable};
 if(!guard[k].guard||!guard[k].parity||!guard[k].repeatable)throw Error('R05 guard failed');
}
guard.headroom=guard.original_concurrent_cap-guard.concurrent_sessions.peak;
if(!guard.allocation_observer_enabled||guard.headroom<0)throw Error('R05 cap exceeded');
write('R05_SUMMARY.json',guard);
console.log(JSON.stringify({source_count:identity.source.length,pc:{before:comparison.before,after:comparison.after,same_refusals:comparison.refusal_identity_equal},r05:guard},null,2));
