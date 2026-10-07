"""Bind final migrated public inputs without replacing intermediate history."""
import difflib, json
from pathlib import Path
from prepare_public_inputs import ROOT, HERE, OUT, INPUTS, PACKS, sha
from run import identity

initial = json.loads((OUT / 'ADAPTATION_001.json').read_text())
validation_path = OUT / 'input_validation_003/SUMMARY.json'
validation = json.loads(validation_path.read_text())
assert validation['passed'] and len(validation['rows']) == len(PACKS) == 15
selection = json.loads((HERE / 'execution/single_goal/EVIDENCE_SELECTION.json').read_text())
receipt_path = HERE / 'evidence' / (selection['checks'][-1] + '.json')
receipt = json.loads(receipt_path.read_text())
current = identity()
assert receipt['exit_code'] == 0 and receipt['source_before'] == current == receipt['source_after']
assert validation['final_input_files'] == {
    str(p.relative_to(ROOT)): sha(p) for p in INPUTS.rglob('*') if p.is_file()}
rows, patches = [], []
for row in initial['files']:
    source, destination = ROOT / row['source'], ROOT / row['copy']
    assert sha(source) == row['source_sha256'], row['source']
    rows.append({**row, 'initial_copy_sha256': row['copy_sha256'],
        'copy_sha256': sha(destination),
        'changed_after_initial_adaptation': sha(destination) != row['copy_sha256']})
    patches.extend(difflib.unified_diff(source.read_text().splitlines(keepends=True),
        destination.read_text().splitlines(keepends=True),
        fromfile=row['source'], tofile=row['copy']))
revisions = [OUT / 'ADAPTATION_001.json', OUT / 'CHARGE_ADAPTATION_002.json',
    OUT / 'input_revision_003/ADAPTATION.json']
initial_copies = {row['copy'] for row in initial['files']}
added_origins = {
    'prepare_pc_u05.py': 'prepare_pc.py plus explicit historical nonemployee resonance0/100/1000 in the isolated draft helper; current product unchanged.',
    'compare_pc_exact_u05.py': 'compare_pc.py with raw003 and distinct output paths only; unchanged-cost failure retained as compat_pc_immediate_002.',
    'compare_pc_cost_u05.py': 'U05 explicit state-size/source-traversal accounting followed by equality of every non-timing field; no unchanged-cost claim or changed numeric ceiling.',
    'finish_pc_u05.py': 'Recovery recipe retaining original gameplay comparator and failed strict cost comparison before accounted comparison.',
}
added_harness = []
for path in sorted(OUT.glob('*.py')):
    if str(path.relative_to(ROOT)) in initial_copies:
        continue
    assert path.name in added_origins, path
    added_harness.append({'path': str(path.relative_to(ROOT)), 'sha256': sha(path),
        'origin': added_origins[path.name]})
    patches.extend(difflib.unified_diff([], path.read_text().splitlines(keepends=True),
        fromfile='/dev/null', tofile=str(path.relative_to(ROOT))))
with (OUT / 'FINAL_INPUT_AND_HARNESS.diff').open('x') as f:
    f.writelines(patches)
result = {
    'status': 'final current input reader validation; functional and performance receipts bound separately by COVERAGE',
    'source_sha256': current['sha256'], 'predecessor_sources_unchanged': True,
    'files': rows, 'packs': [str(p) for p in PACKS],
    'added_harness_files': added_harness,
    'revision_records': {str(p.relative_to(ROOT)): sha(p) for p in revisions},
    'validation_summary_sha256': sha(validation_path), 'validation_receipt_sha256': sha(receipt_path),
    'final_diff_sha256': sha(OUT / 'FINAL_INPUT_AND_HARNESS.diff'),
    'semantic_changes': [
        'Catalog schema5, explicit use lifetime and positive per-form maximum; removed Rest resonance field.',
        'Manual Funeral anchor is ReuseCastTarget; a separate awakened clone preserves the original autonomous skill.',
        'Seven manual trial forms use max25/start5 so their original23second wait reaches readiness at1/s; gameplay and error oracles retain their original assertions.',
        'Two R07 unnamed SkillDef tuples receive explicit Action lifetime; their projectile/permanent modifier payloads are unchanged.',
        'Harness destinations, current continuation version11 and source-bound successor receipt names are explicit changes. Intermediate failures and predecessor bytes are retained.',
        'The PC private nonemployee fixture explicitly retains its original0/100/1000 inputs despite changed employee defaults. All original gameplay/error-code hashes pass separately; new state-size and traversal costs are reported and exactly accounted, not claimed unchanged.',
    ],
    'original_oracle_limit': 'New U05 schema/area probes assert new behavior; inherited numeric/performance guards are not relaxed by this adaptation record.',
}
with (OUT / 'ADAPTATION_FINAL.json').open('x') as f:
    json.dump(result, f, ensure_ascii=False, indent=2)
    f.write('\n')
print(json.dumps({'files': len(rows), 'packs': len(PACKS), 'originals_unchanged': True}))
