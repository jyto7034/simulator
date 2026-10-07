"""Read-only prerequisite discovery and preparation baseline; never invokes an audit."""
from pathlib import Path
import hashlib
import json
import os
import re
import subprocess
import tarfile

ROOT = Path('/mnt/f/work/simulator')
HERE = Path(__file__).resolve().parent
OLD = ROOT / 'core_enhanced/docs/goals/content_authoring_r08_r12_runtime/execution/single_goal'

def sha(path):
    with path.open('rb') as f:
        return hashlib.file_digest(f, 'sha256').hexdigest()

def row(name):
    p = ROOT / name
    return dict(path=name, bytes=p.stat().st_size, sha256=sha(p))

def write(name, value):
    with (HERE / name).open('x') as f:
        json.dump(value, f, ensure_ascii=False, indent=2)
        f.write('\n')

def names():
    baseline = json.loads((OLD / 'baseline/MANIFEST.json').read_text())
    result = {r['path'] for r in baseline['files'] if (ROOT/r['path']).is_file()}
    for folder in ('core_enhanced/src', 'core_enhanced/tests', 'game_resources/data',
                   'core_enhanced/tools/runtime_architecture_final_validation',
                   'core_enhanced/tools/runtime_architecture_preservation'):
        for parent, dirs, files in os.walk(ROOT/folder):
            dirs[:] = [d for d in dirs if d not in ('target', '__pycache__')]
            for name in files:
                p = Path(parent)/name
                if p.is_file():
                    result.add(str(p.relative_to(ROOT)))
    result.add('core_enhanced/docs/goals/content_authoring_value_expressions_reassessment/execution/single_goal/evidence/preimplementation/active_run_v7.pre_r06.json')
    return sorted(result)

def main():
    pointer = json.loads((OLD/'FINAL_TARGET.json').read_text())
    manifest_path = ROOT/pointer['manifest_path']
    manifest = json.loads(manifest_path.read_text())
    current = [row(n) for n in names()]
    by_name = {r['path']: r for r in current}
    frozen = {r['path']: r for r in manifest['source_files']}
    changed = [n for n in sorted(frozen.keys() & by_name.keys()) if frozen[n] != by_name[n]]
    digest = hashlib.sha256(json.dumps([(r['path'],r['bytes'],r['sha256']) for r in current], separators=(',',':')).encode()).hexdigest()
    references = json.loads((manifest_path.parent/'REFERENCES.json').read_text())
    reference_delta = [r['path'] for r in references if not (ROOT/r['path']).is_file() or row(r['path']) != r]
    archives = [dict(path=r['path'], matches=(sha(manifest_path.parent/r['path']) == r['sha256'])) for r in manifest['archives_and_patches']]
    write('CURRENTNESS.json', dict(manifest_path=pointer['manifest_path'], manifest_sha256=sha(manifest_path),
        pointer_hash_matches=sha(manifest_path)==pointer['manifest_sha256'], frozen_digest=manifest['source_digest'],
        current_digest=digest, frozen_count=len(frozen), current_count=len(current), changed=changed,
        added=sorted(by_name.keys()-frozen.keys()), missing=sorted(frozen.keys()-by_name.keys()),
        reference_delta=reference_delta, archives_and_patches=archives,
        scope='Identity check only; not semantic review, test execution, or GEX-20 verdict.'))

    docs = sorted((ROOT/'core_enhanced/docs').rglob('*.md'))
    hits = []
    pattern = re.compile(r'R08.{0,8}R12|GOAL-CARX-01|bd89f9c42191217a333d648d35f495e802d13e6fb1420caa9c08da3deb731690')
    for p in docs:
        if HERE in p.parents:
            continue
        lines = p.read_text(errors='replace').splitlines()
        matches = [dict(line=i+1, text=s) for i,s in enumerate(lines) if pattern.search(s)]
        if matches:
            hits.append(dict(path=str(p.relative_to(ROOT)), sha256=sha(p), matches=matches))
    candidates = []
    for p in (ROOT/'core_enhanced/docs/goals').rglob('*'):
        if p.is_file() and p.suffix in ('.json','.md') and ('AUDIT' in p.name or 'CLOSURE' in p.name or p.name=='REVIEW.md'):
            if pattern.search(p.read_text(errors='replace')):
                candidates.append(row(str(p.relative_to(ROOT))))
    write('AUDIT_DISCOVERY.json', dict(markdown_scanned=len(docs), matches=hits, audit_candidates=candidates,
        finding='No R08-R12 completion verdict found in repository documents inspected; R06/R07 audits have narrower scope.',
        limitation='No assertion about external conversations or files outside this repository.'))

    extra = {str(p.relative_to(ROOT)) for p in (ROOT/'core_enhanced/docs/roadmaps/content_authoring/handoffs').glob('R13_R16*')}
    extra |= {r['path'] for r in references}
    extra |= {str(p.relative_to(ROOT)) for p in (ROOT/'core_enhanced/docs/goals/content_authoring_r08_r12_runtime').glob('*') if p.is_file()}
    extra |= {'core_enhanced/docs/game_rules_next/relics.md', 'core_enhanced/docs/game_rules_next/open-decisions.md',
              'core_enhanced/docs/policy_to_goal_pipeline_revised_v2.md'}
    base = ROOT/'core_enhanced/docs/roadmaps/content_authoring/reviews/2026-09-09-dependency-review-v2.2/roadmap_v2.2-review'
    extra |= {str((base/'goals'/f'R{i}.ko.md').relative_to(ROOT)) for i in range(13,17)}
    extra |= {str((base/f'{n}.ko.md').relative_to(ROOT)) for n in ('CONTRACTS','DEPENDENCIES','ACCEPTANCE')}
    for name in ('PLAN.md','REVIEW_HANDOFF.md','FINAL_TARGET.json','COVERAGE.json','R06_R07_IMPACT_COVERAGE.json'):
        extra.add(str((OLD/name).relative_to(ROOT)))
    extra.add(str(manifest_path.relative_to(ROOT)))
    preserved = [row(n) for n in sorted(set(by_name)|extra)]
    with tarfile.open(HERE/'SOURCE_BEFORE.tar.gz', 'x:gz') as archive:
        for r in preserved:
            archive.add(ROOT/r['path'], arcname=r['path'], recursive=False)
    with tarfile.open(HERE/'SOURCE_BEFORE.tar.gz') as archive:
        actual = [dict(path=m.name, bytes=m.size, sha256=hashlib.sha256(archive.extractfile(m).read()).hexdigest()) for m in archive.getmembers() if m.isfile() or m.islnk()]
    assert actual == preserved
    write('BEFORE_MANIFEST.json', dict(files=preserved, source_files=current, archive_sha256=sha(HERE/'SOURCE_BEFORE.tar.gz'), archive_verified=True))
    for name,args in [('git_status.txt',['status','--short']),('worktree.patch',['diff','--binary']),('index.patch',['diff','--cached','--binary'])]:
        with (HERE/name).open('xb') as f:
            p = subprocess.run(['git',*args],cwd=ROOT,stdout=f,stderr=subprocess.PIPE)
        assert p.returncode == 0, p.stderr
    print(json.dumps(dict(source_count=len(current),changed=changed,added=sorted(by_name.keys()-frozen.keys()),missing=sorted(frozen.keys()-by_name.keys()),reference_delta=reference_delta,preserved=len(preserved),audit_candidates=[r['path'] for r in candidates]),ensure_ascii=False))

if __name__ == '__main__':
    main()
