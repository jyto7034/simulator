from release_gate import require
require('release')
"""Publish current official U06 content and exercise exact negative reader boundaries."""
import json,shutil,subprocess,sys
from initialize import ROOT,HERE
from run import sha
OUT=HERE/('evidence/public_starter_schema_'+sys.argv[1]);OUT.mkdir()
B=ROOT/'.cache/d07-runtime-amount-product-target/release'
base=OUT/'positive';(base/'content').mkdir(parents=True)
command=[B/'package_official_content',ROOT/'game_resources',base/'content/official']
r=subprocess.run(list(map(str,command)),cwd=ROOT,capture_output=True)
(OUT/'package.stdout').write_bytes(r.stdout);(OUT/'package.stderr').write_bytes(r.stderr)
assert r.returncode==0,r.stderr
(base/'selection.json').write_text('{"packs":["official"],"winners":[]}')
equipment='content/official/data/equipments/base.ron'
employees='content/official/data/employees/definitions.ron'
skills='content/official/data/skills/base.ron'
cases=[
 ('positive',None,None,None,None),
 ('zero_single_heal',equipment,'Heal(flat: 8, spell_power_percent: 100','Heal(flat: 0, spell_power_percent: 0',None),
 ('zero_area_heal',equipment,'Heal(flat: 4, spell_power_percent: 50','Heal(flat: 0, spell_power_percent: 0',None),
 ('stat_dependent_single_heal',equipment,'Heal(flat: 8, spell_power_percent: 100','Heal(flat: 0, spell_power_percent: 100',None),
 ('missing_active',employees,'active_skill_fragment_id: Some("training_single_heal")','active_skill_fragment_id: Some("missing_training_heal")','ActiveSkill'),
 ('wrong_kind_active',employees,'active_skill_fragment_id: Some("training_single_heal")','active_skill_fragment_id: Some("starter_basic_attack_enhancement")','ActiveSkill'),
 ('incompatible_active',employees,'active_skill_fragment_id: Some("training_single_heal")','active_skill_fragment_id: Some("training_berserker")','compatible'),
 ('negative_heal',equipment,'action: Heal(flat: 8','action: Heal(flat: -1','Expected integer'),
 ('unknown_heal_field',equipment,'action: Heal(flat: 8','action: Heal(unknown_u06: 1, flat: 8','unknown'),
 ('spatial_basic_heal',equipment,'interval_ms: 1800, windup_ms: 200, delivery: Instant','interval_ms: 1800, windup_ms: 200, delivery: Projectile(speed_units_per_ms: 6000)','Heal requires Instant'),
 ('invalid_reduction',skills,'damage_reduction_percent: 50','damage_reduction_percent: 101','100'),
]
rows=[]
for name,path,old,new,expected in cases:
 fixture=OUT/name
 if name!='positive':shutil.copytree(base,fixture)
 if path:
  target=fixture/path;text=target.read_text();assert old in text,(name,old);target.write_text(text.replace(old,new,1))
 cmd=[B/'author_content','validate',fixture/'content',fixture/'selection.json']
 r=subprocess.run(list(map(str,cmd)),cwd=ROOT,capture_output=True)
 (fixture/'stdout').write_bytes(r.stdout);(fixture/'stderr').write_bytes(r.stderr)
 diagnostic=(r.stdout+r.stderr).decode(errors='replace')
 passed=(r.returncode==0) if expected is None else (r.returncode!=0 and expected.lower() in diagnostic.lower())
 if name=='negative_heal':
  detail=json.loads(r.stderr)['diagnostic']
  passed=passed and detail['code']=='ron_parse' and detail['field_path']=='items[91].weapon_profile.action.Heal.flat' and detail['parser_reason']=='ExpectedInteger'
 row={'case':name,'passed':passed,'exit_code':r.returncode,'expected_diagnostic':expected,'command':list(map(str,cmd)),'stdout_sha256':sha(fixture/'stdout'),'stderr_sha256':sha(fixture/'stderr')}
 rows.append(row);print(json.dumps(row),flush=True)
summary={'passed':all(r['passed'] for r in rows),'rows':rows,'binary_sha256':sha(B/'author_content'),'inputs':{str(p.relative_to(ROOT)):sha(p) for p in OUT.rglob('*') if p.is_file() and p.suffix in ('.ron','.json')}}
(OUT/'SUMMARY.json').write_text(json.dumps(summary,indent=2)+'\n')
assert summary['passed'],'Inspect exact diagnostics; do not count unrelated parse failure'
