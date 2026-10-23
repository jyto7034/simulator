import fs from 'node:fs';
import path from 'node:path';
import crypto from 'node:crypto';
import os from 'node:os';
import {spawn, spawnSync} from 'node:child_process';
const root='/mnt/f/work/simulator';
const sandbox='/tmp/r06-load-baseline.ZZys4Z';
const out=path.dirname(new URL(import.meta.url).pathname);
const mode=process.argv[2];
const hash=p=>crypto.createHash('sha256').update(fs.readFileSync(p)).digest('hex');
const save=(name,data)=>fs.writeFileSync(path.join(out,name),JSON.stringify(data,null,2)+'\n',{flag:'wx'});
function walk(p) {return fs.readdirSync(p,{withFileTypes:true}).sort((a,b)=>a.name.localeCompare(b.name)).flatMap(d=>d.isDirectory()?walk(path.join(p,d.name)):[path.join(p,d.name)]);}
if(mode==='identity') {
  const names=['Cargo.toml','Cargo.lock','core_enhanced/Cargo.toml','game_server/Cargo.toml',
    ...['core_enhanced/src','core_enhanced/tests','game_server/src','game_resources/data'].flatMap(p=>walk(path.join(root,p)).map(f=>path.relative(root,f)))];
  const files=names.map(p=>({path:p,sha256:hash(path.join(root,p)),bytes:fs.statSync(path.join(root,p)).size}));
  const changed=files.filter(f=>hash(path.join(sandbox,f.path))!==f.sha256).map(f=>({path:f.path,copy_sha256:hash(path.join(sandbox,f.path))}));
  if(changed.length!==2 || !changed.every(f=>['core_enhanced/src/game/battle/runtime_control.rs','core_enhanced/src/game/battle/core/value_expr_runtime.rs'].includes(f.path)))throw Error('unexpected copy delta');
  save('source_identity.json',{created:new Date().toISOString(),root,sandbox,files,changed,
    head:spawnSync('git',['rev-parse','HEAD'],{cwd:root,encoding:'utf8'}).stdout.trim()});
  fs.writeFileSync(path.join(out,'archive_files.txt'),names.join('\n')+'\n',{flag:'wx'});
  const tar=spawnSync('tar',['-czf',path.join(out,'source_before.tar.gz'),'-T',path.join(out,'archive_files.txt')],{cwd:root,encoding:'utf8'});
  if(tar.status!==0)throw Error(tar.stderr);
  save('environment.json',{os:os.platform(),release:os.release(),arch:os.arch(),cpus:os.cpus(),memory:os.totalmem(),
    rustc:spawnSync('rustc',['-Vv'],{encoding:'utf8'}).stdout,
    cargo:spawnSync('cargo',['-V'],{encoding:'utf8'}).stdout,
    relevant_env:Object.fromEntries(Object.entries(process.env).filter(([k])=>/^(CARGO_TARGET_DIR|RUSTFLAGS|CARGO_BUILD_JOBS|CARGO_PROFILE_RELEASE_.*)$/.test(k))),
    archive_sha256:hash(path.join(out,'source_before.tar.gz')),
    note:'WSL/Linux local PC measurements; no pinned affinity; separate release build directory; no mobile or Windows-native measurements'});
  console.log(`Identity: ${files.length} original input files, exactly two instrumentation-only copy deltas.`);
} else if(mode==='verify') {
  const m=JSON.parse(fs.readFileSync(path.join(out,'source_identity.json')));
  const drift=m.files.filter(f=>hash(path.join(root,f.path))!==f.sha256);
  if(drift.length)throw Error(JSON.stringify(drift));
  console.log(`CURRENT_SOURCE_UNCHANGED: ${m.files.length} files`);
} else if(mode==='build'||mode==='measure') {
  const tag=process.argv[3]||mode;
  const args=['test','--manifest-path',`${sandbox}/core_enhanced/Cargo.toml`,'--offline','--release','--lib','pc_load_baseline'];
  if(mode==='build')args.push('--no-run');else args.push('--','--ignored','--exact',
    'game::battle::core::value_expr_runtime::tests::load_baseline::pc_load_baseline','--nocapture','--test-threads=1');
  const log=fs.createWriteStream(path.join(out,`${tag}.log`),{flags:'wx'});
  const started=new Date().toISOString();
  const env={...process.env,CARGO_TARGET_DIR:`${sandbox}/target`,CARGO_BUILD_JOBS:'4',R06_BASELINE_OUTPUT:path.join(out,`${tag}_raw.json`)};
  const child=spawn('cargo',args,{cwd:sandbox,env,stdio:['ignore','pipe','pipe']});
  child.stdout.on('data',b=>{log.write(b);process.stdout.write(b)});
  child.stderr.on('data',b=>{log.write(b);process.stdout.write(b)});
  child.on('error',e=>{log.write(String(e));console.error(e)});
  child.on('close',(code,signal)=>{log.end();save(`${tag}_receipt.json`,{started,finished:new Date().toISOString(),command:['cargo',...args],cwd:sandbox,
    target_dir:env.CARGO_TARGET_DIR,jobs:4,code,signal,probe_sha256:hash(path.join(out,'probe.rs'))});process.exitCode=code??1;});
} else throw Error('identity | build [tag] | measure [tag] | verify');
