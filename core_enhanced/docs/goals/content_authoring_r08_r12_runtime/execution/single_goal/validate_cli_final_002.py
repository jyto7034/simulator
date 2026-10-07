"""Public release CLI; exact sources and unmodified predecessor packs are retained."""
import hashlib,json,subprocess,sys
from pathlib import Path
H=Path(__file__).resolve().parent;E=H/'evidence';R=Path('/mnt/f/work/simulator');B=R/'.cache/d07-runtime-amount-product-target/release/author_content'
OLD=R/'core_enhanced/docs/goals/content_authoring_fragment_copies/execution/single_goal/evidence'
PACKS=[('r06',E/'cli_predecessor_current_001/r06','fragment_freischutz_black_round'),('r07_snapshot',E/'cli_predecessor_current_001/r07_snapshot','r07_awakened'),('r07_live',E/'cli_predecessor_current_001/r07_live','r07_awakened'),('r10_snapshot',E/'r10_public_cli_002/snapshot','fragment_freischutz_black_round'),('r10_live',E/'r10_public_cli_002/live','fragment_freischutz_black_round')]+[(f'r12_{name}',E/'r12_public_cli_003'/name,'r08_strike' if name in ['released','effectssettled'] else 'fragment_freischutz_black_round') for name in ['hello','protection','released','effectssettled']]
commands=[];comparisons=[]
def run(name,args):
    assert not (E/f'{name}.json').exists()
    code=subprocess.run([sys.executable,str(H/'run.py'),name,'--',*map(str,args)]).returncode
    commands.append({'name':name,'exit_code':code})
    assert code==0,(name,code)
    return (E/f'{name}.log').read_text()
def normalize(value):
    if isinstance(value,dict):
        out={k:normalize(v) for k,v in value.items()}
        # Inherited R06/R07 audit OBS001: only the presentation employee list.
        if {'employees','physical_damage_unchanged','replaces_run_checkpoint'} <= out.keys():
            out['employees']=sorted(out['employees'],key=lambda e:e['employee_uuid'])
        return out
    if isinstance(value,list): return [normalize(v) for v in value]
    return value
for tag,root,skill in PACKS:
    explain=E/f'cli_{tag}_definition_002.request.json';assert not explain.exists();explain.write_text(json.dumps({'query':'definition','key':{'kind':'Skill','id':skill}}))
    for operation in ['validate','describe','explain']:
        run(f'cli_final_{tag}_{operation}_002',[B,operation,root/'content',root/'selection.json',*([explain] if operation=='explain' else [])])
    results=[]
    for trace in [False,True]:
        text=run(f'cli_final_{tag}_{"on" if trace else "off"}_002',[B,'simulate',root/'content',root/'selection.json',root/'scenario.json','--amount-observation',*(['--trace-output',E/f'cli_final_{tag}_002.trace.jsonl'] if trace else [])])
        rows=[json.loads(line) for line in text.splitlines() if line.strip()]
        public=[normalize(row['payload']['public_result']) for row in rows if row.get('payload',{}).get('kind')=='step']
        amounts=rows[-4:]
        assert all(row.get('kind') in ['amount_preview','amount_explanation'] for row in amounts), (tag,[row.get('kind') for row in amounts])
        results.append((public,amounts))
    assert results[0]==results[1], f'{tag}: trace changed gameplay/amount'
    assert any(row.get('status')=='evaluated' for row in results[0][1]),tag
    comparisons.append({'pack':tag,'public_results':len(results[0][0]),'amount_documents':4,'trace_gameplay_and_amount_equal':True,'normalization':'only inherited trauma recovery employees sorted by UUID'})
with (E/'CLI_FINAL_SUMMARY_002.json').open('x') as f:json.dump({'commands':commands,'comparisons':comparisons},f,indent=2);f.write('\n')
