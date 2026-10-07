#!/usr/bin/env python3
"""Create a new-file-only review patch and ZIP in the local artifact workspace.

Does not access the connected simulator repository. Local patch verification uses
an isolated temporary directory; none of the copied scripts are executed.
"""
from __future__ import annotations
import argparse
import difflib
import hashlib
import json
import subprocess
import tempfile
import zipfile
from pathlib import Path

ROOT=Path(__file__).resolve().parents[1]
DEST='core_enhanced/docs/roadmaps/content_authoring/reviews/2026-09-09-dependency-review-v2.2'
ZIP=ROOT.parent/'simulator_content_authoring_review_v2.2-review.1_20260909.zip'


def files():
    return sorted(p for p in ROOT.rglob('*') if p.is_file()
                  and '__pycache__' not in p.parts and p.suffix!='.pyc'
                  and p.name not in {'SHA256SUMS.json','CREATE_ONLY.patch'})


def write_patch():
    chunks=[]; included=files()
    for p in included:
        rel=p.relative_to(ROOT).as_posix(); target=DEST+'/'+rel
        a=json.dumps('a/'+target,ensure_ascii=False)
        b=json.dumps('b/'+target,ensure_ascii=False)
        chunks.extend([f'diff --git {a} {b}\n','new file mode 100644\n'])
        lines=p.read_bytes().decode('utf-8').splitlines(keepends=True)
        diff=list(difflib.unified_diff([],lines,fromfile='/dev/null',tofile='b/'+target+'\t',n=3))
        for line in diff:
            chunks.append(line if line.endswith('\n') else line+'\n\\ No newline at end of file\n')
    patch=ROOT/'CREATE_ONLY.patch'
    patch.write_text(''.join(chunks),encoding='utf-8')
    return patch,included


def main():
    ap=argparse.ArgumentParser();ap.add_argument('--patch-only',action='store_true');args=ap.parse_args()
    patch,included=write_patch()
    if args.patch_only:
        print(f'Created actual create-only diff for {len(included)} new files. No repository application.')
        return
    receipt=json.loads((ROOT/'evidence/DOCUMENT_VALIDATION.json').read_text(encoding='utf-8'))
    if receipt.get('status')!='passed':raise RuntimeError('Document validation must pass before packaging.')
    # Apply only to an empty, disposable local directory, never the remote workspace.
    with tempfile.TemporaryDirectory(prefix='simulator_review_patch_check_') as tmp:
        run=subprocess.run(['git','-C',tmp,'apply','--whitespace=nowarn',str(patch)],capture_output=True,text=True,timeout=30)
        if run.returncode:raise RuntimeError('Local new-file patch validation failed: '+run.stderr)
        for p in included:
            restored=Path(tmp)/DEST/p.relative_to(ROOT)
            if restored.read_bytes()!=p.read_bytes():raise RuntimeError('Patch byte mismatch: '+str(p))
    manifest_items=included+[patch]
    manifest={'kind':'distribution_file_sha256_not_repository_source_hashes',
              'version':'2.2-review.1','algorithm':'SHA-256',
              'files':{p.relative_to(ROOT).as_posix():hashlib.sha256(p.read_bytes()).hexdigest() for p in manifest_items}}
    mp=ROOT/'SHA256SUMS.json';mp.write_text(json.dumps(manifest,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
    with zipfile.ZipFile(ZIP,'w',compression=zipfile.ZIP_DEFLATED,compresslevel=9) as z:
        for p in sorted(manifest_items+[mp]):z.write(p,arcname=ROOT.name+'/'+p.relative_to(ROOT).as_posix())
    with zipfile.ZipFile(ZIP) as z:
        bad=z.testzip()
        if bad:raise RuntimeError('CRC failure: '+bad)
        for rel,digest in manifest['files'].items():
            actual=hashlib.sha256(z.read(ROOT.name+'/'+rel)).hexdigest()
            if actual!=digest:raise RuntimeError('ZIP manifest mismatch: '+rel)
        count=len(z.namelist())
    result={'status':'passed','archive':str(ZIP),'archive_sha256':hashlib.sha256(ZIP.read_bytes()).hexdigest(),
            'archive_file_count':count,'manifest_verified_files':len(manifest['files']),
            'new_file_patch_verified_files':len(included),'patch_test':'git apply in isolated local temporary directory; restored bytes all equal',
            'zip_crc':'passed','zip_manifest':'all listed SHA-256 values match',
            'repository_applied':False,'game_tests_executed':False,'independent_audit':False,
            'document_validation':{'checks':receipt['check_count'],'passed':receipt['passed_count'],'failed':receipt['failed_count']}}
    output=ROOT.parent/'simulator_content_authoring_review_packaging_validation.json'
    output.write_text(json.dumps(result,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
    print(json.dumps(result,ensure_ascii=False,indent=2))


if __name__=='__main__':main()
