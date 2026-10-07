"""Strict current release package inputs: positive control and named U07 refusals."""
import json,shutil,subprocess,sys
from initialize import ROOT,HERE
from run import sha,identity
label=sys.argv[1];out=HERE/('evidence/public_object_schema_'+label);out.mkdir();binary=ROOT/'.cache/d07-runtime-amount-product-target/release';base=out/'positive';(base/'content').mkdir(parents=True)
r=subprocess.run([str(binary/'package_official_content'),str(ROOT/'game_resources'),str(base/'content/official')],capture_output=True)
(out/'package.stdout').write_bytes(r.stdout);(out/'package.stderr').write_bytes(r.stderr);assert r.returncode==0,r.stderr
(base/'selection.json').write_text('{"packs":["official"],"winners":[]}\n')
obj='content/official/data/summoned_units/base.ron';skill='content/official/data/skills/base.ron'
cases=[
 ('positive',None,None,None,None),
 ('ttl_zero',obj,'ttl_ms:30000','ttl_ms:0','ttl_ms'),
 ('zero_target_limit',obj,'max_targets:Some(1)','max_targets:Some(0)','max_targets'),
 ('unknown_capability',obj,'inherit_source_defenses:false','inherit_source_defenses:false,unknown_u07:true','unknown_u07'),
 ('missing_definition',skill,'definition_id:"laetitia_small_friend"','definition_id:"u07_missing_object"','unknown independent object'),
 ('empty_placements',skill,'offsets:[(x:0,y:0)],allow_same_source_evolution:true','offsets:[],allow_same_source_evolution:true','nonempty offset'),
 ('duplicate_placements',skill,'definition_id:"common_ice_wall",anchor:CastTile,offsets:[(x:0,y:0)]','definition_id:"common_ice_wall",anchor:CastTile,offsets:[(x:0,y:0),(x:0,y:0)]','offsets must be unique'),
 ('zero_hp_cost',skill,'health_cost_percent:20,wall_health_percent:200','health_cost_percent:0,wall_health_percent:200','positive cost'),
 ('trap_wall_cost',skill,'definition_id:"melting_love_meat_wall",anchor:CastTile','definition_id:"laetitia_big_friend",anchor:CastTile','HP wall'),
 ('evolution_cycle',obj,'display_name:"Great Witch Friend",','display_name:"Great Witch Friend", evolves_to:Some("laetitia_small_friend"),','evolution cycle'),
]
rows=[]
for name,path,old,new,expected in cases:
 fixture=out/name
 if name!='positive':shutil.copytree(base,fixture)
 if path:
  p=fixture/path;s=p.read_text();assert old in s,(name,old);p.write_text(s.replace(old,new,1))
 command=[binary/'author_content','validate',fixture/'content',fixture/'selection.json'];r=subprocess.run(list(map(str,command)),capture_output=True)
 (fixture/'stdout').write_bytes(r.stdout);(fixture/'stderr').write_bytes(r.stderr);diagnostic=(r.stdout+r.stderr).decode(errors='replace')
 passed=r.returncode==0 if expected is None else r.returncode!=0 and expected.lower() in diagnostic.lower()
 rows.append({'case':name,'passed':passed,'exit_code':r.returncode,'expected_diagnostic':expected,'stdout_sha256':sha(fixture/'stdout'),'stderr_sha256':sha(fixture/'stderr')});print(name,passed,flush=True)
summary={'passed':all(r['passed'] for r in rows),'source':identity(),'rows':rows,'binary_sha256':sha(binary/'author_content'),'inputs':{str(p.relative_to(ROOT)):sha(p) for p in out.rglob('*') if p.is_file() and p.suffix in ('.ron','.json')}}
(out/'SUMMARY.json').write_text(json.dumps(summary,indent=2)+'\n');assert summary['passed'],'Inspect each diagnostic; unrelated refusals cannot pass.'
