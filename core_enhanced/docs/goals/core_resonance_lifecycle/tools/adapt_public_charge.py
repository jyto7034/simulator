"""Keep the seven manual public trials' original 23-second setup and all oracles.

Match the current in-tree R06/R07/R10/R12 trial fixture: form maximum 25,
initial floor(25/5)=5, ordinary 1/s pulses fill it during the existing wait.
Only successor inputs are edited; autonomous enemy profiles are untouched.
"""
import json, re
from prepare_public_inputs import HERE, ROOT, OUT, INPUTS, PACKS, sha, spans

rows=[]
for relative in PACKS:
    scenario=INPUTS/relative/'scenario.json'
    if not scenario.exists(): continue
    value=json.loads(scenario.read_text())
    skill_ids={s['command']['skill_id'] for s in value['steps']
               if s.get('command',{}).get('type')=='activate_skill'}
    if not skill_ids: continue
    assert skill_ids <= {'fragment_freischutz_black_round','r07_awakened'}
    path=scenario.parent/'content/official/data/skill_fragments/base.ron'
    before=sha(path)
    text=path.read_text()
    candidates=[(a,b) for a,b in spans(text,'SkillFragmentMetadata')
                if re.search(r'id:\s*"authoring_preview_fragment"',text[a:b])]
    assert len(candidates)==1
    a,b=candidates[0]
    block=text[a:b]
    for skill_id in skill_ids | {'fragment_freischutz_black_round'}:
        block,count=re.subn(r'(skill_id:\s*"'+skill_id+r'", resonance_max:) 100',r'\1 25',block)
        assert count==1,(path,skill_id,count)
    text=text[:a]+block+text[b:]
    path.write_text(text)
    rows.append({'path':str(path.relative_to(ROOT)),'previous_sha256':before,'sha256':sha(path),'manual_skills':sorted(skill_ids),'maximum':25,'initial':5,'existing_wait_ms':23000,'scenario_sha256':sha(scenario)})
assert len(rows)==7
with (OUT/'CHARGE_ADAPTATION_002.json').open('x') as f:
    json.dump({'reason':__doc__,'files':rows,'original_scenarios_unchanged':True},f,indent=2)
print(json.dumps({'adapted_manual_trials':len(rows),'original_oracles_unchanged':True}))
