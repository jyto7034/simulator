import fs from 'node:fs';
import path from 'node:path';
import crypto from 'node:crypto';
import {execFileSync} from 'node:child_process';
const root='/mnt/f/work/simulator';
const here=path.dirname(new URL(import.meta.url).pathname);
const predecessor=root+'/core_enhanced/docs/goals/content_authoring_value_expressions_reassessment/execution/single_goal/segments/wp02_effect_execution_prefix_001/target/effect_execution_prefix_001/MANIFEST.json';
const sha=b=>crypto.createHash('sha256').update(b).digest('hex');
const meta=p=>{const b=fs.readFileSync(p);return {path:path.relative(root,p),bytes:b.length,sha256:sha(b)}};
const old=JSON.parse(fs.readFileSync(predecessor));
const files=old.source.files.map(f=>f.path);
if(process.argv[2]==='baseline'){
  if(meta(predecessor).sha256!=='f8d854b8ea257a2ea19045caf61434d0592ee2333cfe827edfb5eaac7530b52f')throw Error('predecessor mismatch');
  for(const f of old.source.files)if(meta(root+'/'+f.path).sha256!==f.after.sha256)throw Error('source drift '+f.path);
  const dir=here+'/baseline/entry_001';fs.mkdirSync(dir,{recursive:true});
  if(fs.existsSync(dir+'/MANIFEST.json'))throw Error('baseline already frozen');
  const archive=dir+'/SOURCE_BEFORE.tar.gz';
  execFileSync('tar',['-czf',archive,...files],{cwd:root});
  const record={state:'pre_correction_actual_bytes',created_at:new Date().toISOString(),predecessor:meta(predecessor),source:{files:files.map(p=>meta(root+'/'+p)),archive:meta(archive)}};
  fs.writeFileSync(dir+'/MANIFEST.json',JSON.stringify(record,null,2)+'\n',{flag:'wx'});
  console.log(JSON.stringify({baseline:meta(dir+'/MANIFEST.json'),source_count:files.length}));
}
