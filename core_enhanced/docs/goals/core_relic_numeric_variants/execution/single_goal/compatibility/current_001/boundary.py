import sys
from pathlib import Path
sys.path.insert(0, str(Path(__file__).resolve().parents[4] / 'tools'))
from release_gate import require
require('release')
import hashlib
import json
from pathlib import Path
import subprocess
import sys

C = Path(__file__).resolve().parent
R = Path('/mnt/f/work/simulator')
label = sys.argv[1] if len(sys.argv) > 1 else '001'
out = C / ('boundary_' + label)
out.mkdir(exist_ok=False)
sha = lambda path: hashlib.sha256(path.read_bytes()).hexdigest()
# Use the precise cargo artifact set from the just-built public host probe.
build = C / ('public_probe_' + label) / 'build.jsonl'
libraries = {}
for line in build.read_text().splitlines():
    item = json.loads(line)
    if item.get('reason') == 'compiler-artifact' and item['target']['name'] in ('game_core', 'serde_json', 'uuid'):
        for name in item['filenames']:
            if name.endswith('.rlib'): libraries[item['target']['name']] = Path(name)
assert len(libraries) == 3
cases = {
 'valid': (None, "fn check(p: PreparedNonBattleApplication<'_>) { let _ = p.result(); let _ = p.checkpoint(); let _ = p.enters_battle(); let _ = p.commit(); }"),
 'private': ('E0616', "fn check(p: PreparedNonBattleApplication<'_>) { let _ = p.prepared; }"),
 'clone': ('E0277', "fn need<T: Clone>() {} fn check() { need::<PreparedNonBattleApplication<'static>>(); }"),
 'serialize': ('E0277', "fn check(p: PreparedNonBattleApplication<'_>) { let _ = serde_json::to_string(&p); }"),
 'deserialize': ('E0277', "fn check() { let _ = serde_json::from_str::<PreparedNonBattleApplication<'static>>(\"{}\"); }"),
 'reuse': ('E0382', "fn check(p: PreparedNonBattleApplication<'_>) { let _ = p.commit(); let _ = p.commit(); }"),
 'interleave': ('E0499', "fn check(a: &mut CoreApplication, c: ApplicationCommand) { let p = a.prepare_non_battle_command(uuid::Uuid::nil(), c.clone(), ApplicationExecutionContext::default()).unwrap(); let _ = a.execute(uuid::Uuid::nil(), c, ApplicationExecutionContext::default()); p.commit(); }"),
 'foreign': ('E0616', "fn check<'a>(mut p: PreparedNonBattleApplication<'a>, a: &'a mut CoreApplication) { p.application = a; }"),
}
rows = []
for name, (error, body) in cases.items():
    src = out / (name + '.rs')
    src.write_text('use game_core::application::*;\n' + body + '\nfn main() {}\n')
    cmd = ['rustc', '--edition=2021', str(src), '-o', str(out / name)]
    for crate, lib in libraries.items(): cmd += ['--extern', crate + '=' + str(lib)]
    cmd += ['-L', 'dependency=' + str(libraries['game_core'].parent)]
    result = subprocess.run(cmd, capture_output=True)
    log = out / (name + '.log'); log.write_bytes(result.stdout + result.stderr)
    assert result.returncode == (1 if error else 0), (name, result.stderr)
    if error: assert ('error[' + error + ']').encode() in result.stderr, (name, result.stderr)
    rows.append(dict(name=name, command=cmd, exit_code=result.returncode, expected_error=error, source_sha256=sha(src), log_sha256=sha(log)))
(out / 'RESULT.json').write_text(json.dumps(dict(passed=True, cases=rows, libraries={str(p):sha(p) for p in libraries.values()}, build_receipt_sha256=sha(build)), indent=2) + '\n')
print(json.dumps(dict(passed=True, cases=len(rows))))
