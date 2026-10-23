"""New processes consume exact published checkpoints through public Core API."""
from pathlib import Path
import base64
import hashlib
import json
import subprocess
import time

A = Path(__file__).resolve().parent
E = A.parent
S = E/'storage_refactor'
R = next(p for p in E.parents if (p/'game_server/Cargo.toml').is_file())
sha = lambda p: hashlib.sha256(p.read_bytes()).hexdigest()
binary = S/'artifacts/diagnostic001/restore_cost'
binding = json.loads((S/'runs/current001/cost/BINDINGS.json').read_bytes())
assert sha(binary) == binding['normal'][str(binary)] == binding['expanded'][str(binary)]
out = A/'restore_probe'
out.mkdir()
results = []
for name, mode, source, milestone in [
    ('normal-savepoint','normal',S/'runs/current001/normal-forward','save_before_hatch'),
    ('normal-selected','normal',S/'runs/current001/normal-forward','Stage3_choice_selected'),
    ('expanded-pending','expanded',S/'runs/current001/expanded-forward','Stage3_choice_pending'),
    ('old-writer','normal',E/'single_goal/runs/current001/normal','Stage2_hatched'),
]:
    case = out/name
    case.mkdir()
    active = source/'milestones'/milestone/'active/active_run.json'
    profile = source/'milestones'/milestone/'profile/player_profile.json'
    expected = source/(milestone+'.json')
    saved = json.loads(active.read_bytes())
    (case/'core_payload.json').write_bytes(base64.b64decode(saved['core_payload_base64'],validate=True))
    (case/'metadata.json').write_text(json.dumps(saved['monotonic']))
    (case/'profile.json').write_text(json.dumps(json.loads(profile.read_bytes())['profile']))
    config = S/'test_parts'/(mode+'_inputs001/admission.json')
    result_path = case/'RESULT.json'
    argv = [str(binary),str(config),str(case),str(expected),str(result_path)]
    started = time.monotonic()
    with (case/'stdout').open('xb') as stdout, (case/'stderr').open('xb') as stderr:
        p = subprocess.run(argv,cwd=R,stdout=stdout,stderr=stderr,timeout=600)
    receipt = dict(name=name,argv=argv,exit_code=p.returncode,seconds=time.monotonic()-started,
        inputs={str(f):sha(f) for f in [active,profile,expected,config,binary,case/'core_payload.json']},
        conversion='base64 decode and envelope extraction only; no payload/checksum/semantic edits')
    (case/'RECEIPT.json').write_text(json.dumps(receipt,indent=2)+'\n')
    assert p.returncode == 0, receipt
    result = json.loads(result_path.read_bytes())
    assert result['passed'] and result['matches_published_progress'] and result['exact_requested_bytes_recovered']
    results.append(dict(name=name,exit_code=p.returncode,passed=True,
        public_continue_seconds=result['seconds'],native_work=result['native']['work'],
        exact_requested_bytes_recovered=True))
    print(json.dumps(results[-1]),flush=True)
(out/'SUMMARY.json').write_text(json.dumps(dict(passed=True,processes=len(results),results=results),indent=2)+'\n')
