from pathlib import Path
import shutil,subprocess,json
H=Path(__file__).resolve().parent;E=H/'evidence';P=E/'public_cli_001';B=Path('/mnt/f/work/simulator/.cache/d07-runtime-amount-product-target/release')
root=P/'resolved_groups';shutil.copytree(P/'content',root)
p=root/'official/data/skills/base.ron';s=p.read_text();lines=s.splitlines();index=next(i for i,l in enumerate(lines) if '1125899906842624' in l)
parts=[]
for source,owner in [('ResolvedTargets','Side'),('ResolvedTargetsWhere([IsActualEmployee])','ActualEmployee'),('ResolvedTargetsWhere([IsActualEmployee,HasEquippedSlot(Weapon)])','EquippedSlot(Weapon)'),('ResolvedTargetsWhere([IsActualEmployee,HasEquippedSlot(Armor)])','EquippedSlot(Armor)')]:
 parts.append(f'Aggregate(source:{source},grouping:ByResolvedOwner({owner}),reduction:Sum,outer_reduction:Some(Sum),member:Const((value:4,unit:Magnitude)))')
lines[index]='(id:"effect_01",op:DamageExpr(amount:(moment:Live,root:Add(['+','.join(parts)+'])),damage_type:True)),],'
p.write_text('\n'.join(lines)+'\n')
cmd=[str(B/'author_content'),'simulate',str(root),str(P/'selection.json'),str(P/'scenario.json'),'--amount-observation']
with (E/'cli_resolved_groups_001.log').open('xb') as f:r=subprocess.run(cmd,stdout=f,stderr=subprocess.STDOUT)
assert r.returncode==0,(r.returncode,(E/'cli_resolved_groups_001.log').read_text()[-2000:])
docs=[json.loads(l) for l in (E/'cli_resolved_groups_001.log').read_text().splitlines()[-4:]]
evaluated=[d for d in docs if d['status']=='evaluated'];assert len(evaluated)==2
assert all(d['typed_base_amount']['value']==4 for d in evaluated),[(d['typed_base_amount']) for d in evaluated]
with (E/'CLI_AGGREGATE_SUMMARY.json').open('x') as f:json.dump({'command':cmd,'exit':r.returncode,'evaluated_documents':2,'four_resolved_owner_views':True,'expected_and_actual':4,'semantics':'one real enemy in Side view contributes4; explicit actual-employee/slot filters are empty and contribute0'},f,indent=2)
print('Public resolved owner aggregate: actual runtime amount4 in preview and explain')
