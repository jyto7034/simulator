import fs from 'node:fs';
import path from 'node:path';
const base=path.dirname(new URL(import.meta.url).pathname);
const input=process.argv[2]||'measure_001_raw.json';
const data=JSON.parse(fs.readFileSync(path.join(base,input)));
const groups=new Map();
function key(r){return JSON.stringify({family:r.family,n:r.enemies??r.members??r.target_equivalent,
  chunk:r.chunk_ms,sources:r.burst_sources??r.source_equivalent??r.repetitions,
  reads:r.reads_per_expr,grouped:r.grouped_per_tile});}
for(const r of data.rows.filter(r=>!r.warmup)) {const k=key(r);if(!groups.has(k))groups.set(k,[]);groups.get(k).push(r);}
const quantile=(a,p)=>a.length?a.slice().sort((x,y)=>x-y)[Math.ceil(a.length*p)-1]:null;
const summary=[];
for(const [k,rows] of groups){
  const timed=rows.filter(r=>!(r.measurement??r.setup).allocation_observer);
  const alloc=rows.filter(r=>(r.measurement??r.setup).allocation_observer);
  const ms=timed.map(r=>(r.measurement??r.setup).elapsed_ns/1e6);
  const hashes=[...new Set(rows.map(r=>r.after_log_sha256).filter(Boolean))];
  const errors=[...new Set(rows.map(r=>r.error??r.setup_error).filter(Boolean))];
  const usages=rows.flatMap(r=>Array.isArray(r.native_usage?.[0])||r.native_usage?.[0]===null?r.native_usage:(r.native_usage?[r.native_usage]:[])).filter(Boolean);
  summary.push({...JSON.parse(k),samples:timed.length,allocation_samples:alloc.length,errors,
    elapsed_ms:{p50:quantile(ms,.5),p95:quantile(ms,.95),max:Math.max(...ms)},
    requested_bytes:{min:alloc.length?Math.min(...alloc.map(r=>(r.measurement??r.setup).requested_bytes_sum)):null,
      max:alloc.length?Math.max(...alloc.map(r=>(r.measurement??r.setup).requested_bytes_sum)):null},
    native_admitted_work_max:usages.length?Math.max(...usages.map(x=>x[0])):null,
    native_clone_slots_max:usages.length?Math.max(...usages.map(x=>x[1])):null,
    native_retained_admission_max:usages.length?Math.max(...usages.map(x=>x[2]??0)):null,
    result_hash_variants:hashes.length,example:rows[0]});
}
const output=input.replace(/_raw\.json$/,'_summary.json');
fs.writeFileSync(path.join(base,output),JSON.stringify({input,summary},null,2)+'\n',{flag:'wx'});
for(const s of summary)console.log(JSON.stringify({family:s.family,n:s.n,chunk:s.chunk,sources:s.sources,
  reads:s.reads,grouped:s.grouped,p50:s.elapsed_ms.p50,p95:s.elapsed_ms.p95,
  bytes:s.requested_bytes.max,work:s.native_admitted_work_max,errors:s.errors,
  hp:s.example.hp_loss,hashes:s.result_hash_variants}));
