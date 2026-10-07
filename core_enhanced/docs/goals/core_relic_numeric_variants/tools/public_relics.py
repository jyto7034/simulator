"""Independent U08 release CLI checks over the immutable successful E009 export."""
import json
import re
import subprocess
import sys
from pathlib import Path
from run import ROOT, HERE, identity, sha
from release_gate import require


def exact_json(actual, expected, location):
    """Keep integer recipe fields distinct from JSON floats and booleans."""
    assert type(actual) is type(expected), (location, type(actual).__name__, type(expected).__name__)
    if isinstance(expected, dict):
        assert actual.keys() == expected.keys(), (location, actual.keys(), expected.keys())
        for key in expected:
            exact_json(actual[key], expected[key], location + '.' + key)
    elif isinstance(expected, list):
        assert len(actual) == len(expected), (location, len(actual), len(expected))
        for index, (left, right) in enumerate(zip(actual, expected)):
            exact_json(left, right, location + '[' + str(index) + ']')
    else:
        assert actual == expected, (location, actual, expected)


def relic_definition_oracle():
    # Independently transcribed authored E009 RON constants, not expected.json
    # or another API result. Serde's external enum tags come from RelicPenalty.
    selector = {'target':'AnyDamage', 'all_tags':[], 'damage_type':None,
                'equipment_id':None, 'fragment_id':None, 'form_id':None}
    return {
        'id':'u08_trial_blend', 'uuid':'a508e100-0000-0000-0000-000000000001',
        'name':'External trial blend', 'description':'Public grant, event, save and damage',
        'rarity':'probe', 'allow_duplicates':False, 'incompatible_with':[],
        'effects':[{'id':'first', 'selector':dict(selector), 'flat':20, 'percent':20},
                   {'id':'second', 'selector':dict(selector), 'flat':0, 'percent':30}],
        'corruption':{'effects':[{'id':'corrupt', 'selector':dict(selector), 'flat':40, 'percent':50}],
                      'penalties':[{'DefenseFlat':{'amount':-15}}, {'MagicResistFlat':{'amount':-10}}]},
        'dismantle_equipment_dust':7,
    }


def check_definition_header(document, generation, key, source):
    exact_json(document['identity'], generation, 'explain.identity')
    exact_json(document['key'], key, 'explain.key')
    assert document['present'] is True
    assert document['representation'] == 'resolved_definition' and document['index_space'] is None
    exact_json(document['source'], source, 'explain.source')
    exact_json(document['writers'], [{'target':key, 'pack_id':'official', 'operation':'Add',
                                      'after':[], 'selected':True}], 'explain.writers')
    assert isinstance(document['value'], dict) and document['value']


def check_static_public(validation, description, skill_explanation, relic_explanation,
                        generation, expected_source_receipt):
    assert validation['status'] == 'validated'
    exact_json(validation['identity'], generation, 'validate.identity')
    exact_json(validation['source_receipt'], expected_source_receipt, 'validate.source_receipt')
    exact_json(description['identity'], generation, 'describe.identity')
    exact_json(description['source_receipt'], expected_source_receipt, 'describe.source_receipt')
    safety = validation['runtime_safety_sha256']
    assert isinstance(safety, str) and re.fullmatch(r'[0-9a-f]{64}', safety)
    assert description['runtime_safety_sha256'] == safety
    assert description['catalog_kinds'].count('Relics') == 1
    assert description['definition_kinds'].count('Relic') == 1
    key = {'kind':'Relic', 'id':'u08_trial_blend'}
    definition_source = {'pack_id':'official', 'source_file':'official/data/relics/base.ron',
                         'field_path':'relics[0]'}
    relic_members = [row for row in description['definitions'] if row[0]['kind'] == 'Relic']
    exact_json(relic_members, [[key, definition_source]], 'describe.relic_membership')
    assert any(row['pack_id'] == 'official' and row['path'] == definition_source['source_file']
               for row in expected_source_receipt['files'])
    check_definition_header(skill_explanation, generation,
        {'kind':'Skill', 'id':'u08_trial_damage'},
        {'pack_id':'official', 'source_file':'official/data/skills/base.ron', 'field_path':'skills[0]'})
    check_definition_header(relic_explanation, generation, key, definition_source)
    expected_definition = relic_definition_oracle()
    exact_json(relic_explanation['value'], expected_definition, 'explain.relic.value')
    normal = expected_definition['effects']
    corrupt = expected_definition['corruption']['effects']
    # Separate hand arithmetic corroborates the recipes; battle consumption is
    # still independently checked below against actual HP and full trace parity.
    normal_amount = (100 + sum(row['flat'] for row in normal)) * (100 + sum(row['percent'] for row in normal)) // 100
    corrupt_amount = (100 + sum(row['flat'] for row in corrupt)) * (100 + sum(row['percent'] for row in corrupt)) // 100
    assert (normal_amount, corrupt_amount) == (180, 210)
    return {'identity':generation, 'runtime_safety_sha256':safety,
            'source_file_count':len(expected_source_receipt['files']),
            'relic_key':key, 'relic_source':definition_source, 'relic_writers':relic_explanation['writers'],
            'resolved_relic_definition':relic_explanation['value'],
            'independent_recipe_amounts':{'base':100,'normal':180,'corrupt':210},
            'oracle_basis':'Immutable E009 authored RON constants and documented typed serialization; expected.json is not a static-definition oracle.'}


label = sys.argv[1]
assert label.isdigit()
binding = require('release')
source = identity()
fixture = HERE / 'evidence/public_trial_cli009'
out = HERE / ('evidence/public_relics_' + label)
out.mkdir()
binary = ROOT / '.cache/d07-runtime-amount-product-target/release/author_content'
manifest = json.loads((fixture / 'export_manifest.json').read_text())

def verify_inputs():
    expected_paths = {r['path'] for r in manifest}
    assert len(expected_paths) == len(manifest)
    assert expected_paths == {str(p.relative_to(fixture)) for p in fixture.rglob('*')
                              if p.is_file() and p.name != 'export_manifest.json'}
    for row in manifest:
        path = fixture / row['path']
        assert path.stat().st_size == row['bytes'] and sha(path) == row['sha256']

verify_inputs()
generation = json.loads((fixture / 'generation.json').read_text())
scenario = json.loads((fixture / 'scenario.json').read_text())
assert json.loads((fixture / 'selection.json').read_text()) == {'packs':['official'], 'winners':[]}
pack = (fixture / 'content/official/pack.ron').read_text()
assert re.search(r'pack_id:\s*"official"', pack) and re.search(r'operations:\s*\[\s*\]', pack)
assert re.search(r'kind:\s*Relics\s*,\s*path:\s*"data/relics/base.ron"', pack)
content_manifest = json.loads((fixture / 'content_manifest.json').read_text())
exported_content = [{'path':row['path'][len('content/'):], 'bytes':row['bytes'], 'sha256':row['sha256']}
                    for row in manifest if row['path'].startswith('content/')]
exact_json(sorted(content_manifest, key=lambda row:row['path']),
           sorted(exported_content, key=lambda row:row['path']), 'immutable content manifest')
assert len(content_manifest) == 27 and len({row['path'] for row in content_manifest}) == 27
expected_source_receipt = {'files':[{'pack_id':'official', 'path':row['path'],
    'byte_count':row['bytes'], 'sha256':row['sha256']} for row in sorted(content_manifest, key=lambda row:row['path'])]}
# Independent authored constants and arithmetic: these do not come from expected.json.
skills = (fixture / 'content/official/data/skills/base.ron').read_text()
relics = (fixture / 'content/official/data/relics/base.ron').read_text()
assert re.search(r'id:\s*"u08_trial_damage".*?DamageExpr\(amount:\s*\(moment:\s*Snapshot,\s*root:\s*Const\(\(value:\s*100,\s*unit:\s*Magnitude\)\)\),\s*damage_type:\s*True', skills, re.S)
assert 'id:"corrupt"' in relics and 'flat:40,percent:50,' in relics
base, flat, percent = 100, 40, 50
effective = (base + flat) * (100 + percent) // 100
assert effective == 210 and effective != base
site = {'action':{'action_id':'hit', 'definition_id':'u08_trial_damage'}, 'effect_id':'amount'}
origin = {'field_path':'skills[0].steps[0].effects[0]', 'origin':'Original',
          'origin_definition_id':'u08_trial_damage', 'site':site, 'source_file':'official/data/skills/base.ron'}
request = out / 'definition.request.json'
request.write_text(json.dumps({'query':'definition', 'key':{'kind':'Skill', 'id':'u08_trial_damage'}}))
relic_request = out / 'relic_definition.request.json'
with relic_request.open('x') as stream:
    json.dump({'query':'definition', 'key':{'kind':'Relic', 'id':'u08_trial_blend'}}, stream)
commands = []

def run(name, arguments):
    require('release')
    command = list(map(str, [binary, *arguments]))
    result = subprocess.run(command, cwd=ROOT, capture_output=True)
    stdout, stderr = out / (name + '.stdout'), out / (name + '.stderr')
    with stdout.open('xb') as stream: stream.write(result.stdout)
    with stderr.open('xb') as stream: stream.write(result.stderr)
    commands.append({'command':command, 'exit_code':result.returncode,
                     'stdout_sha256':sha(stdout), 'stderr_sha256':sha(stderr)})
    assert result.returncode == 0, (name, result.returncode, result.stderr[-2000:])
    return [json.loads(line) for line in result.stdout.splitlines() if line.strip()]

static_documents = {}
for operation in ['validate', 'describe', 'explain']:
    documents = run(operation, [operation, fixture/'content', fixture/'selection.json',
                                *([request] if operation == 'explain' else [])])
    assert len(documents) == 1, (operation, len(documents))
    static_documents[operation] = documents[0]
documents = run('explain_relic', ['explain', fixture/'content', fixture/'selection.json', relic_request])
assert len(documents) == 1, ('explain_relic', len(documents))
static_public_proof = check_static_public(static_documents['validate'], static_documents['describe'],
    static_documents['explain'], documents[0], generation, expected_source_receipt)

def normalize(value):
    if isinstance(value, dict):
        result = {key:normalize(item) for key, item in value.items()}
        # The sole inherited presentation normalization, also used by U07-M.
        if {'employees', 'physical_damage_unchanged', 'replaces_run_checkpoint'} <= result.keys():
            result['employees'] = sorted(result['employees'], key=lambda item:item['employee_uuid'])
        return result
    if isinstance(value, list): return [normalize(item) for item in value]
    return value

variants = []
proofs = []
for traced in [False, True]:
    documents = run('simulate_' + ('on' if traced else 'off'),
        ['simulate',fixture/'content',fixture/'selection.json',fixture/'scenario.json','--amount-observation',
         *(['--trace-output',out/'trace.jsonl'] if traced else [])])
    rows = [row['payload'] for row in documents if row.get('payload',{}).get('kind') == 'step']
    steps = {row['index']:row for row in rows}
    assert len(steps) == len(rows) == len(scenario['preparation_steps']) + len(scenario['steps'])
    assert set(steps) == set(range(len(rows)))
    assert all(row['content'] == generation for row in rows)
    amounts = documents[-4:]
    assert [row['kind'] for row in amounts] == ['amount_preview','amount_explanation'] * 2
    variants.append(([normalize(row['public_result']) for row in rows], amounts))
    payload = lambda index: steps[index]['public_result']['payload']
    granted = payload(15)
    assert len(granted['copies']) == 1
    copy = granted['copies'][0]
    assert copy['definition_id'] == 'u08_trial_blend' and copy['variant'] == 'Normal'
    assert granted['resources']['relic_corruption_catalysts'] == 3
    for index, amount in [(17,180),(26,effective),(29,180),(35,effective)]:
        targets = [t for t in payload(index)['targets'] if t['kind'] == 'skill_damage' and t['site'] == site]
        assert len(targets) == 1
        assert targets[0]['evaluation'] == {'status':'available','base_amount':base,'effective_amount':amount}
    assert payload(28)['copies'][0] == copy
    assert payload(36)['copies'][0]['copy_uuid'] == copy['copy_uuid']
    assert payload(36)['copies'][0]['variant'] == 'Corrupt'
    history = [{'source_scene_id':'first','copy_uuid':copy['copy_uuid']},
               {'source_scene_id':'second','copy_uuid':None}]
    assert payload(24)['event']['relic_corruption_selections'] == history
    assert payload(33)['event']['relic_corruption_selections'] == history
    preview, explanation = amounts[:2]
    for document in [preview, explanation]:
        assert document['status'] == 'evaluated' and document['basis'] == 'recorded_evaluation'
        assert document['selector'] == {'battle':'active','selector':'latest'}
        assert document['moment'] == 'Snapshot' and document['provenance'] == 'AuthoredExpression'
        assert document['site'] == site
        assert document['typed_base_amount'] == {'unit':'Magnitude','value':base}
        assert document['narrowed_base_amount'] == {'kind':'damage','value':base}
        assert document['effective_relic_amount'] == {'kind':'damage','value':effective}
        assert document['identity']['content_generation'] == generation
        numeric, runtime_source = document['relic_numeric'], document['source']
        assert numeric['generation'] == generation and numeric['target'] == {'kind':'skill_damage','site':site}
        assert numeric['contributions'] == [dict(copy_uuid=copy['copy_uuid'], definition_id='u08_trial_blend',
            effect_id='corrupt', flat=flat, percent=percent, variant='Corrupt')]
        assert numeric['actor'] == runtime_source['actor'] == runtime_source['cast']['caster_instance_id']
        assert numeric['action_ordinal'] == runtime_source['action_ordinal']
        assert numeric['employee_uuid'] == payload(9)['employee_uuid']
        fragment = runtime_source['fragment']
        grants = [r for r in payload(13)['skill_fragment_diffs'] if r['fragment_id'] == 'u08_trial_fragment']
        assert len(grants) == 1
        assert fragment == dict(copy_uuid=grants[0]['copy_uuid'], definition_id='u08_trial_fragment',
                                form_id='base', skill_id='u08_trial_damage')
        assert numeric['fragment'] == fragment and numeric['damage_type'] == 'True'
        assert document['release_target'] == document['consumption_target']
        assert document['release_time_ms'] == document['consume_time_ms'] == 8001
    plan = explanation['static_plan']
    assert plan['source'] == origin and plan['semantic_fingerprint'] == explanation['semantic_fingerprint']
    assert preview['semantic_fingerprint'] == explanation['semantic_fingerprint']
    assert plan['segments'] == [{'index':0,'ops':[{'PushConst':{'node_index':0,'value':{'unit':'Magnitude','value':base}}}]}]
    assert {k:v for k,v in explanation.items() if k not in ['kind','static_plan']} == {k:v for k,v in preview.items() if k != 'kind'}
    target = preview['consumption_target']
    before_update, after_update = payload(48)['battle_update'], payload(49)['battle_update']
    unit = lambda update: next(u for u in update['checkpoint']['units'] if u['unit_instance_id'] == target)
    hp_before, hp_after = unit(before_update)['stats']['current_health'], unit(after_update)['stats']['current_health']
    assert hp_before > effective and hp_before - hp_after == effective
    hits = [e for e in after_update['events_delta']['events'] if e['event']['type'] == 'HpChanged'
            and e['event'].get('target_instance_id') == target and e['event'].get('damage_source') == 'Ability']
    assert len(hits) == 1
    hit = hits[0]['event']
    assert hit['raw_damage'] == hit['final_damage'] == effective and hit['delta'] == -effective
    assert hit['hp_after'] == hp_after and hit['source_instance_id'] == preview['source']['actor']
    # Export is corroboration only, after arithmetic, actual HP and public membership checks.
    expected = json.loads((fixture/'expected.json').read_text())
    assert expected['copy_uuid'] == copy['copy_uuid'] and expected['generation'] == generation
    assert expected['consumed_damage']['hp_before'] == hp_before and expected['consumed_damage']['hp_after'] == hp_after
    assert expected['consumed_damage']['source']['authored'] == origin
    proofs.append({'hp_before':hp_before,'hp_after':hp_after,'effective':effective,'copy_uuid':copy['copy_uuid'],
                   'generation':generation,'amount_identity':preview['identity'],'hit':hits[0]})
assert variants[0] == variants[1], 'trace changed a public result or any of four amount documents'
assert proofs[0] == proofs[1]
verify_inputs()
assert source == identity() and binding == require('release')
with (out/'SUMMARY.json').open('x') as stream:
    json.dump({'passed':True,'source_sha256':source['sha256'],'release_binding':binding,'commands':commands,
               'input_export_manifest_sha256':sha(fixture/'export_manifest.json'), 'proof':proofs[0],
               'static_public_proof':static_public_proof,
               'public_result_count':len(variants[0][0]),'amount_document_count':4,
               'trace_full_public_and_amount_equal':True,
               'normalization':'only inherited trauma recovery employees sorted by UUID',
               'limits':'Fresh-process Continue remains the separate E009 test evidence; CLI consumes the exact normal scenario.'},stream,indent=2)
print('PASS fresh release U08 exact validate/describe/relic explain, external grant/savepoint/event, independent 210 non-clamped damage and full trace parity')
