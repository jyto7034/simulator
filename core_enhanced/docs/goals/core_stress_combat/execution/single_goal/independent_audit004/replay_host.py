"""New approved-host stores; supported commands only, preserved raw evidence."""
from pathlib import Path
import json
import shutil
import sys
import tempfile
import uuid

A = Path(__file__).resolve().parent
E = A.parent
sys.path.insert(0, str(E / 'test_parts'))
import completion_host as h
h.E = A

root = Path(tempfile.mkdtemp(prefix='u164-audit004-host-'))
receipts = []
assets = lambda r: [r['run'][k] for k in ['resources', 'roster', 'inventory']] + [r['inventory']]
employees = lambda r: {e['employee_definition_id']: e for e in r['run']['roster']['employees']}

def close(host):
    receipts.append({'name': host.name, **host.close()})
    print(json.dumps(receipts[-1]), flush=True)

host = h.Host('prepare', root / 'base')
try:
    h.start(host)
    h.event_assets(host)
    h.enter(host, 5)
    host.command('complete_node', fail=True)
    host.command('complete_node')
    before = host.last
    assert before['run']['game_state'] == 'viewing_map'
finally:
    close(host)

for name in ['extraction', 'unfinished']:
    shutil.copytree(root / 'base', root / name)
    host = h.Host(name, root / name)
    try:
        host.snapshot()
        assert assets(host.last) == assets(before)
        node = h.enter(host, 8)
        h.deploy(host)
        host.tick(60000)
        choice = h.stress(host)
        assert choice['completed_rounds'] == 1 and choice['awaiting_choice']
        assert assets(host.last) == assets(before)
        checkpoint = host.reconnect()['result']['payload']['battle_update']['checkpoint']
        host.tick(60000)
        assert host.reconnect()['result']['payload']['battle_update']['checkpoint'] == checkpoint
        host.command('stress_combat_act', battle_uuid=str(uuid.UUID(int=1)), completed_rounds=1, extract=True, reject=True)
        host.command('stress_combat_act', battle_uuid=choice['battle_uuid'], completed_rounds=0, extract=True, reject=True)
        host.command('retreat_battle', reject=True)
        if name == 'extraction':
            h.act(host, extract=True, fail=True)
            h.act(host, extract=True)
            result = host.last
            assert result['run']['game_state'] == 'combat_result'
            assert result['run']['resources']['enkephalin'] - before['run']['resources']['enkephalin'] == 10
            for employee in ['scout_runner', 'breach_guard', 'field_medic']:
                assert employees(result)[employee]['experience'] - employees(before)[employee]['experience'] == (0 if employee == 'field_medic' else 11)
            for material in ['equipment_dust', 'u15_part_a']:
                value = lambda r: next(x['amount'] for x in r['inventory']['equipment_materials'] if x['material_id'] == material)
                assert value(result) - value(before) == 1
            host.command('stress_combat_act', battle_uuid=choice['battle_uuid'], completed_rounds=1, extract=True, reject=True)
            assert assets(host.last) == assets(result)
    finally:
        close(host)
    fresh = h.Host(name + '_fresh', root / name)
    try:
        fresh.snapshot()
        expected = result if name == 'extraction' else before
        assert assets(fresh.last) == assets(expected)
        assert fresh.last['run']['map'] == expected['run']['map']
        assert fresh.last['run']['game_state'] == expected['run']['game_state']
        assert fresh.last['run']['run_progression'] == expected['run']['run_progression']
        if name == 'extraction':
            assert fresh.last['run']['selected_event'] == result['run']['selected_event']
            fresh.command('complete_combat_result')
            assert fresh.last['run']['game_state'] == 'viewing_map'
    finally:
        close(fresh)

(A / 'HOST_REPLAY.json').write_text(json.dumps({'passed': True, 'root': str(root), 'processes': receipts,
    'scope': ['new public preparation', 'SavePoint actual IO retry', 'actual first round', 'choice clock/reconnect',
              'wrong UUID/round rejection', 'extraction IO retry', 'reward/XP exact once',
              'fresh unconfirmed result and acknowledgement', 'fresh incomplete entry rollback']}, indent=2) + '\n')
