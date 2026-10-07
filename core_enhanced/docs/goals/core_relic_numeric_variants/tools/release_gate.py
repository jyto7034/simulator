"""Bind freshly built executables to immutable current-source build receipts."""
import json
import os
from pathlib import Path
import sys
from run import ROOT, HERE, identity, sha

NAMES = {'release': ['author_content', 'package_official_content', 'consume_content_pack', 'game_server'],
         'debug': ['author_content', 'measure_runtime', 'game_server']}

def binding_path(profile, label):
    assert profile in NAMES and label.isdigit()
    return HERE / 'evidence' / f'{profile}_binding_{label}.json'

def bind(profile, label):
    path = binding_path(profile, label)
    assert not path.exists(), path
    receipt_path = HERE / 'evidence' / f'{profile}_build_{label}.json'
    receipt = json.loads(receipt_path.read_text())
    current = identity()
    assert receipt['exit_code'] == 0 and receipt['source_unchanged']
    assert receipt['source_before'] == current == receipt['source_after']
    assert receipt['log_sha256'] == sha(receipt_path.with_suffix('.log'))
    command = receipt['command']
    assert command[:2] == ['cargo', 'build'] and '--locked' in command and '--offline' in command
    assert 'game_server/runtime-measurement' in command
    assert ('--release' in command) == (profile == 'release')
    target = Path(receipt['environment']['CARGO_TARGET_DIR'])
    binaries = {str(target / profile / name): sha(target / profile / name) for name in NAMES[profile]}
    value = {'profile':profile, 'label':label, 'source_sha256':current['sha256'],
             'build_receipt':str(receipt_path), 'build_receipt_sha256':sha(receipt_path),
             'binaries':binaries, 'passed':True}
    with path.open('x') as stream:
        json.dump(value, stream, indent=2)
        stream.write('\n')
    return value

def require(profile):
    label = os.environ.get('U08_' + profile.upper() + '_LABEL', '')
    assert label.isdigit(), f'U08_{profile.upper()}_LABEL must identify a completed fresh build'
    path = binding_path(profile, label)
    value = json.loads(path.read_text())
    assert value['passed'] and value['profile'] == profile and value['label'] == label
    assert value['source_sha256'] == identity()['sha256'], 'source differs from the fresh build'
    receipt_path = Path(value['build_receipt'])
    assert value['build_receipt_sha256'] == sha(receipt_path)
    receipt = json.loads(receipt_path.read_text())
    assert receipt['log_sha256'] == sha(receipt_path.with_suffix('.log'))
    target = Path(os.environ.get('CARGO_TARGET_DIR', ROOT / '.cache/d07-runtime-amount-product-target'))
    assert set(value['binaries']) == {str(target / profile / name) for name in NAMES[profile]}
    assert all(sha(Path(p)) == digest for p, digest in value['binaries'].items()), 'binary changed after build'
    return value

if __name__ == '__main__':
    action, profile, label = sys.argv[1:]
    assert action == 'bind'
    print(json.dumps(bind(profile, label)))
