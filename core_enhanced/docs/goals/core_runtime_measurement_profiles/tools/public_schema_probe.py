"""Actual release-reader negative cases for the U05 cutover, with positive controls."""
import json, shutil, subprocess, sys
from prepare_public_inputs import ROOT, HERE, OUT, INPUTS, E, sha, spans

label=sys.argv[1] if len(sys.argv)>1 else '001'
destination=HERE/('evidence/public_schema_'+label)
destination.mkdir(exist_ok=False)
source=INPUTS/E/'r12_public_cli_003/protection'
binary=ROOT/'.cache/d07-runtime-amount-product-target/release/author_content'
skills=source/'content/official/data/skills/base.ron'
fragments=source/'content/official/data/skill_fragments/base.ron'
equipment=source/'content/official/data/equipments/base.ron'
manifest=source/'content/official/pack.ron'
descriptor='ActionAndEffects(effects: [(step: "hit", effect: "heal"), (step: "hit", effect: "personal"), (step: "hit", effect: "parry")])'
assert descriptor in skills.read_text()
area='TileArea(footprint: (include_anchor_tile: true, rows: ["XXX", "X@X", "XXX"]), area: (anchor: Caster, tile_origin: Caster, duration_ms: 1000, tick_interval_ms: Some(100)))'
def persistent_finite(text):
    # Avoid the older, earlier shared-shield/Parry Instant-only producer checks
    # so this variant reaches the new independent-producer lifetime boundary.
    text=text.replace('delivery: Instant','delivery: '+area,1).replace('shared_radius_units: Some(30000000)','shared_radius_units: None',1)
    old='Parry(duration_ms: 4500, window_ms: 3200, invulnerable_ms: 1000, miss_recovery_ms: 20, reduction_percent: 0, counter_skill: "r12_counter")'
    assert old in text
    return text.replace(old,'Modifiers(duration_ms: 4500, modifiers: [(stat: Attack, kind: Flat, value: 1)])',1).replace('HealthCost(amount: 10, allow_lethal: false)','Heal(amount: 1)',1)
cases=[
    ('positive',None,None,None),
    ('old_schema',manifest,lambda s:s.replace('catalog_schema:5','catalog_schema:4'), 'catalog'),
    ('missing_lifetime',skills,lambda s:s.replace('use_lifetime: '+descriptor+', ', '',1), 'use_lifetime'),
    ('missing_capacity',fragments,lambda s:s.replace(', resonance_max: 25','',1), 'resonance_max'),
    ('zero_capacity',fragments,lambda s:s.replace('resonance_max: 25','resonance_max: 0',1), 'resonance_max'),
    ('removed_rest',equipment,lambda s:s.replace('hp_recovery_per_tick: 3,','hp_recovery_per_tick: 3, resonance_recovery_per_tick: 5,',1), 'resonance_recovery_per_tick'),
    ('unlisted_finite',skills,lambda s:s.replace(descriptor,'Action',1),'requires an explicit finite use effect reference'),
    ('empty_effect_list',skills,lambda s:s.replace(descriptor,'ActionAndEffects(effects: [])',1),'requires at least one finite effect'),
    ('hostile_effect_owned',skills,lambda s:s.replace('(step: "hit", effect: "parry")','(step: "taunt", effect: "taunt")',1),'must uniquely reference a finite beneficial effect'),
    ('persistent_finite_producer',skills,persistent_finite,'finite use effect cannot be produced by an independent persistent area'),
    ('manual_automatic_retarget',skills,lambda s:s.replace('id: "fragment_quiet_lament",','id: "fragment_quiet_lament",',1).replace('targeting: ReuseCastTarget,\n                    when: IfPreviousStepDealtDamage','targeting: RetargetOnStep,\n                    when: IfPreviousStepDealtDamage'), 'manual employee fragment cannot retarget after admission'),
]
rows=[]
for name,path,change,expected in cases:
    fixture=destination/name
    shutil.copytree(source/'content',fixture/'content')
    shutil.copyfile(source/'selection.json',fixture/'selection.json')
    if path:
        copied=fixture/path.relative_to(source)
        original=copied.read_text();adapted=change(original)
        assert adapted!=original,(name,'no mutation')
        copied.write_text(adapted)
    command=[binary,'validate',fixture/'content',fixture/'selection.json']
    result=subprocess.run(list(map(str,command)),cwd=ROOT,capture_output=True)
    (fixture/'stdout').write_bytes(result.stdout);(fixture/'stderr').write_bytes(result.stderr)
    text=(result.stdout+result.stderr).decode(errors='replace')
    passed=result.returncode==0 if expected is None else result.returncode!=0 and expected.lower() in text.lower()
    rows.append({'case':name,'exit_code':result.returncode,'expected_diagnostic':expected,'passed':passed,
                 'command':list(map(str,command)),'stdout_sha256':sha(fixture/'stdout'),'stderr_sha256':sha(fixture/'stderr'),
                 'mutated_input':str(copied.relative_to(ROOT)) if path else None,'mutated_sha256':sha(copied) if path else None})
    print(json.dumps(rows[-1]),flush=True)
summary={'passed':all(r['passed'] for r in rows),'rows':rows,'binary_sha256':sha(binary)}
(destination/'SUMMARY.json').write_text(json.dumps(summary,indent=2)+'\n')
assert summary['passed'],'Review the exact public diagnostics; do not accept an unrelated refusal'
