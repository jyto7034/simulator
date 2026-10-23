"""Supported RON kind alias, normal packaging/admission, public manual route."""
from pathlib import Path
import hashlib
import json
import shutil
import subprocess
import sys
import tempfile
import time

A = Path(__file__).resolve().parent
E = A.parent
R = next(p for p in E.parents if (p/'game_server/Cargo.toml').is_file())
S = E/'storage_refactor'
OLD = E/'single_goal/test_parts'
sys.path.insert(0, str(OLD))
import host_driver as driver
from full_flow import SOURCE, materials, fragments, equipment

sha = lambda p: hashlib.sha256(p.read_bytes()).hexdigest()


def run(name, argv):
    started = time.monotonic()
    with (A/(name+'.stdout')).open('xb') as out, (A/(name+'.stderr')).open('xb') as err:
        p = subprocess.run(list(map(str, argv)), cwd=R, stdout=out, stderr=err, timeout=1200)
    (A/(name+'.receipt.json')).write_text(json.dumps(dict(argv=list(map(str,argv)),
        exit_code=p.returncode, seconds=time.monotonic()-started), indent=2)+'\n')
    print(name, p.returncode, flush=True)
    assert p.returncode == 0, name


def prepare():
    inputs = A/'kind_id_inputs'
    inputs.mkdir()
    source = inputs/'source'
    shutil.copytree(OLD/'normal_inputs002/source', source)
    definitions = source/'data/map/node_definitions.ron'
    text = definitions.read_text()
    start = text.index('        (')
    end = text.index('        ),', start)+len('        ),')
    ordinary = text[start:end]
    assert 'kind_id: "combat_monster"' in ordinary and 'category: Combat' in ordinary
    alias = ordinary.replace('kind_id: "combat_monster"', 'kind_id: "combat_elite_training"')
    definitions.write_text(text[:end]+'\n'+alias+text[end:])
    layout = source/'data/map/facility_templates.ron'
    text = layout.read_text()
    start = text.index('(id:"ordinary",')
    end = text.index('payload:', start)
    prefix = text[start:end]
    assert prefix.count('kind_id:"combat_monster"') == 1
    layout.write_text(text[:start]+prefix.replace('kind_id:"combat_monster"', 'kind_id:"combat_elite_training"')+text[end:])
    changed = []
    for p in source.rglob('*'):
        if p.is_file():
            previous = OLD/'normal_inputs002/source'/p.relative_to(source)
            if sha(p) != sha(previous):
                changed.append(dict(path=str(p.relative_to(source)),before_sha256=sha(previous),after_sha256=sha(p)))
    assert len(changed) == 2, changed
    (inputs/'INPUT_DELTA.json').write_text(json.dumps(changed,indent=2)+'\n')
    content = inputs/'content'
    content.mkdir()
    run('kind-package001', [S/'artifacts/normal001/package_official_content', source, content/'official'])
    config = json.loads((S/'test_parts/normal_inputs001/admission.json').read_bytes())
    private = Path(tempfile.mkdtemp(prefix='u17-audit-kind-'))
    config.update(content_root=str(content), issuer_root=str(private/'issuer_private'),
                  profile_revision='u17-audit-kind-id001')
    path = inputs/'admission.json'
    path.write_text(json.dumps(config,indent=2)+'\n')
    host = S/'artifacts/normal001/u164_stress_host'
    for name in ['calibration','adoption']:
        (private/name).mkdir(mode=0o700)
    run('kind-calibration001', [host,'--cmva-calibrate',path,private/'calibration'])
    reviewed = S/'test_parts/normal_inputs001/reviewed_limits.json'
    review = R/'core_enhanced/docs/goals/core_expedition_deterrence_rewards/execution/single_goal/test_parts/review_calibration.py'
    run('kind-limits001', ['python3',review,A/'kind-calibration001.stdout',reviewed,A/'kind-limits.json'])
    run('kind-adoption001', [host,'--cmva-adopt-calibration',path,private/'adoption','u17-audit-kind-id001',reviewed])
    return path


def probe(config):
    driver.HOST = S/'artifacts/normal001/u164_stress_host'
    root = Path(tempfile.mkdtemp(prefix='u17-audit-kind-route-'))
    host = driver.Host('kind-id-route',root/'store',root/'host',config)
    try:
        driver.start(host)
        for slot in [8,4]:
            driver.enter(host,slot)
            driver.win(host)
            host.command('complete_combat_result')
        copies = fragments(host.last,'fragment_censored_trace')
        assert len(copies) == 1
        driver.enter(host,9)
        host.command('dismantle_skill_fragment',copy_uuid=copies[0]['copy_uuid'],confirmed=True)
        host.command('complete_node')
        driver.enter(host,5)
        driver.win(host)
        host.command('complete_combat_result')
        assert not fragments(host.last,'fragment_censored_trace')
        driver.enter(host,1)
        driver.win(host)
        assert host.last['run']['revival']['eggs'] == {SOURCE:'Stage2'}
        host.command('complete_combat_result')
        driver.enter(host,2)
        offer = next(o for o in host.last['run']['selected_event']['offers'] if o['merchandise']['type']=='revival_catalyst')
        host.command('purchase_item',offer_id=offer['offer_id'])
        host.command('exit_shop')
        assert materials(host.last).get('revival_catalyst') == 4
        driver.enter(host,3)
        host.command('complete_node')
        node_id = driver.select(host,6)
        before = host.last
        quote = next(q for q in before['run']['revival']['hatch_options'] if q['reward_source_id']==SOURCE)
        selected = driver.node(host,6)
        assert selected['kind_id'] == 'combat_elite_training'
        assert selected['state'] == 'Available'
        assert quote['stage']=='Stage2' and quote['catalyst_required']==2
        (root/'BEFORE.json').write_text(json.dumps(before,indent=2)+'\n')
        rejected = host.command('hatch_encounter',node_id=node_id,reward_source_id=SOURCE,stage='Stage2',
            payment=[dict(material_id='revival_catalyst',carried=2,warehouse=0)],reject=True)
        assert rejected['run']==before['run'] and rejected['inventory']==before['inventory']
        (root/'AFTER.json').write_text(json.dumps(rejected,indent=2)+'\n')
        result = dict(reproduced=True,classification='in scope: supported authored input and public commands',
            node=selected,quote=quote,expected_can_hatch=True,actual_can_hatch=quote['can_hatch'],
            command_accepted=rejected['accepted'],rejection=rejected.get('result'),
            roots_unchanged=True,config_sha256=sha(config),host_sha256=sha(driver.HOST))
        assert not result['actual_can_hatch'] and not result['command_accepted']
        (root/'OBSERVATION.json').write_text(json.dumps(result,ensure_ascii=False,indent=2)+'\n')
    finally:
        host.close()
        shutil.copytree(root,A/'kind_id_route')
    print(json.dumps(dict(reproduced=True,output=str(A/'kind_id_route'),operations=host.count)),flush=True)


if __name__=='__main__':
    probe(prepare())
