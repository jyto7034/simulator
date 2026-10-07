"""Verify the documentation-only F01 correction and exact execution reuse."""
import datetime
import hashlib
import json
import re
import subprocess
import sys
import tarfile
from pathlib import Path

C = Path(__file__).resolve().parent
S = C.parents[1]
H = S.parents[1]
sys.path.insert(0, str(H / 'tools'))
import run

R = run.ROOT
A = H / 'post_audit/u04_20261004_001'
T = S / 'target/u04_001'
DOC = R / 'game_server/docs/unity_bridge_protocol.md'


def bound(path):
    return {'path': str(path.relative_to(R)), 'sha256': run.sha(path), 'bytes': path.stat().st_size}


def main():
    product = run.identity()
    assert product['sha256'] == 'cb033469ed897c2bb38cdd5cd2bc4561b8b820acd0b680d45c5381abf4b3c629'
    assert run.sha(T / 'MANIFEST.json') == 'f1b9bd51c71fe415fccc79f8f073462915b7baa79b3440f20238d41f35a65f5f'
    manifest = json.loads((T / 'MANIFEST.json').read_text())
    assert manifest['product_source'] == product
    for row in manifest['archives_and_patches']:
        assert run.sha(T / row['path']) == row['sha256'], row['path']
    assert run.sha(A / 'AUDIT_ARTIFACTS.json') == 'a7dd51b1920b5d79b0211648520de01c040220bf0614449a2c20d5407520bcab'
    audit = json.loads((A / 'AUDIT_ARTIFACTS.json').read_text())
    assert audit['verdict'] == 'Not Complete'
    for row in audit['files']:
        assert run.sha(A / row['path']) == row['sha256'], row['path']
    before = json.loads((S / 'baseline/HOST_PROTOCOL_BEFORE.json').read_text())[0]
    with tarfile.open(S / 'baseline/HOST_PROTOCOL_BEFORE.tar.gz') as archive:
        assert archive.getnames() == [before['path']]
        raw = archive.extractfile(before['path']).read()
    assert hashlib.sha256(raw).hexdigest() == before['sha256']
    assert raw == (A / 'HOST_PROTOCOL_AT_AUDIT.md').read_bytes()
    doc = DOC.read_text()
    assert run.sha(DOC) != before['sha256']
    subprocess.run(['git', 'diff', '--check', '--', str(DOC.relative_to(R))], cwd=R, check=True)
    for link in re.findall(r'\]\(([^)]+)\)', doc):
        if not link.startswith(('http:', 'https:', '#')):
            assert (DOC.parent / link.split('#')[0]).resolve().is_file(), link
    # This validates exact vocabulary only; the source/semantic review is recorded
    # separately and remains subject to independent re-audit.
    behavior = (R / 'core_enhanced/src/game/behavior.rs').read_text()
    commands = {
        'CompleteCombatResult': 'complete_combat_result',
        'SelectReward': 'select_reward', 'ClaimReward': 'claim_reward',
        'ContinueRun': 'continue_run', 'LoadRunCheckpoint': 'load_run_checkpoint',
        'RetreatBattle': 'retreat_battle', 'RetryPlayerProfile': 'retry_player_profile',
        'StartNewGame': 'start_new_game',
    }
    for variant, wire in commands.items():
        assert re.search(r'\b' + variant + r'\b', behavior), variant
        assert '`' + wire + '`' in doc or '"type": "' + wire + '"' in doc, wire
    state = (R / 'game_server/src/game/player_game_actor/state.rs').read_text()
    for field in ['player_profile_persistence', 'application_recovery_pending']:
        assert field in state and field in doc, field
    assert all(text not in doc for text in [
        'strict envelope version 4', 'Core payload version 9',
        'RetryEncounterCompendium', 'ResetEncounterCompendium',
        'retry_encounter_compendium_persistence', 'encounter_compendium_persistence',
        'finish_combat_replay', 'backup rotation keeps the previous Core payload',
    ])
    coverage = json.loads((S / 'COVERAGE.json').read_text())
    assert coverage['source_sha256'] == product['sha256']
    reused = []
    for name, binding in coverage['receipt_bindings'].items():
        receipt_path = H / 'evidence' / (name + '.json')
        log_path = H / 'evidence' / (name + '.log')
        assert run.sha(receipt_path) == binding['receipt_sha256'], name
        assert run.sha(log_path) == binding['log_sha256'], name
        receipt = json.loads(receipt_path.read_text())
        assert receipt['source_before'] == product == receipt['source_after'], name
        assert receipt['exit_code'] == binding['raw_exit_code'], name
        for item in receipt.get('command_inputs', []):
            assert run.sha(H / item['archive']) == item['sha256'], item
        assert run.sha(H / 'evidence/runner_versions' / (receipt['runner_sha256'] + '.py')) == receipt['runner_sha256']
        reused.append({'id': name, **binding})
    assert len(reused) == 40
    # The original product identity is byte-for-byte unchanged. The only revised
    # canonical product-facing file in this correction is the required host doc.
    source_paths = [
        'core_enhanced/src/application.rs', 'core_enhanced/src/game/behavior.rs',
        'core_enhanced/src/game/world/combat.rs', 'core_enhanced/src/game/world/reward.rs',
        'core_enhanced/src/game/world/active_run/continuation.rs',
        'game_server/src/game/player_game_actor/handlers.rs',
        'game_server/src/game/player_game_actor/messages.rs',
        'game_server/src/game/player_game_actor/state.rs',
        'game_server/src/persistence/active_run_store.rs',
        'game_server/src/persistence/player_profile_store.rs',
        'game_server/src/persistence/player_profile_store/application_intent.rs',
    ]
    result = {
        'recorded_at_utc': datetime.datetime.now(datetime.timezone.utc).isoformat(),
        'goal': 'GOAL-CRRP-01', 'finding': 'F01', 'stage': 'correction GEX-10',
        'passed': True, 'product_unchanged': True, 'source_sha256': product['sha256'],
        'before_document': before, 'after_document': bound(DOC),
        'original_audit': bound(A / 'AUDIT_ARTIFACTS.json'),
        'original_target': bound(T / 'MANIFEST.json'),
        'original_audit_and_target_artifacts_unchanged': True,
        'source_review_bindings': [bound(R / p) for p in source_paths],
        'document_command_vocabulary': commands,
        'markdown_local_links_exist': True, 'diff_whitespace_check_passed': True,
        'reused_current_executions': reused,
        'reuse_reason': 'Only canonical host documentation and correction records/status changed; every product/input/receipt/log/runner byte for all40 final executions remains exact. No product test rerun is claimed or needed for this documentation correction.',
        'affected_completion': ['PE15', 'V12 canonical consumer contract', 'V14 required surface/restoration'],
        'required_next': 'Successor target including host protocol, exact restoration, fresh independent GEX-20; no self-awarded F01 closure.',
    }
    with (C / 'VALIDATION.json').open('x') as stream:
        json.dump(result, stream, ensure_ascii=False, indent=2)
        stream.write('\n')
    print(json.dumps({'passed': True, 'source_sha256': product['sha256'], 'unchanged_executions': len(reused), 'document_sha256': run.sha(DOC)}))


if __name__ == '__main__':
    main()
