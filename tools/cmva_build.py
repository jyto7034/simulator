#!/usr/bin/env python3
"""Prepare and execute the supported CMVA release matrix from actual Cargo input.

The output is a trusted build input, not a mod-supplied assertion. Unbound Core
builds remain diagnostic and cannot issue approval. The command collector owns
execution receipts and before/after source protection around this tool.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import subprocess

ROOT = Path(__file__).resolve().parent.parent
PACKAGES = ['game_core', 'game_server', 'u11_independent_public_probe', 'u12_independent_lifecycle_probe']
MANIFESTS = ['Cargo.toml', 'Cargo.lock', 'core_enhanced/Cargo.toml', 'game_server/Cargo.toml',
 'core_enhanced/docs/goals/core_mod_validation_apply/execution/single_goal/test_parts/runtime001/probe/Cargo.toml',
 'core_enhanced/docs/goals/core_expedition_lifecycle/execution/single_goal/test_parts/runtime001/probe/Cargo.toml']
ENVIRONMENT = ['RUSTFLAGS', 'CARGO_ENCODED_RUSTFLAGS', 'RUSTC', 'RUSTC_WRAPPER', 'RUSTC_WORKSPACE_WRAPPER', 'RUSTUP_TOOLCHAIN']
def sha(raw):
    return hashlib.sha256(raw).hexdigest()
def core_graph(tree):
    # Cargo separates the selected package roots with blank lines. Keep every
    # dependency/feature row; only those formatting separators are discarded.
    lines = [line for line in tree.splitlines() if line.strip()]
    found = set()
    for index, line in enumerate(lines):
        depth = len(line) - len(line.lstrip('0123456789'))
        if not depth:
            raise ValueError('Cargo tree depth prefix missing')
        level = int(line[:depth])
        if not line[depth:].startswith('game_core v'):
            continue
        graph = []
        for child in lines[index+1:]:
            prefix = len(child) - len(child.lstrip('0123456789'))
            if not prefix:
                raise ValueError('Cargo tree depth prefix missing')
            child_level = int(child[:prefix])
            if child_level <= level:
                break
            graph.append(str(child_level-level) + child[prefix:])
        found.add('\n'.join(graph))
    if len(found) != 1:
        raise ValueError('actual Core dependency graph is missing or inconsistent')
    return found.pop()
def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--mode', choices=['plain','instrumented'], required=True)
    parser.add_argument('--kind', choices=['bins','lib-tests'], required=True)
    parser.add_argument('--out', type=Path, required=True)
    parser.add_argument('--target-dir', type=Path, required=True)
    parser.add_argument('--prepare-only', action='store_true')
    parser.add_argument('--worker-only', action='store_true',
        help='Build only mod_launcher while retaining the supported package/feature graph')
    parser.add_argument('--with-worker', action='store_true',
        help='Build mod_launcher with selected plain probes in the same prepared invocation')
    parser.add_argument('--probe-bin', choices=['native_fault_gap','worker_boundaries','approved_lifecycle','public_lifecycle','normal_host_lifecycle','u14_deterrence_host','u15_economy_host','u164_stress_host'], action='append',
        help='Build only the required independent diagnostic probe with the supported plain graph')
    parser.add_argument('--package-official', action='store_true',
        help='Build package_official_content alone or with selected probes using the supported plain graph')
    parser.add_argument('--server-lib-only', action='store_true',
        help='Build only the server library test harness after proving its effective Core graph matches the supported matrix')
    parser.add_argument('--with-probe-integration', action='store_true',
        help='Compile the three required probe integration harnesses with the library tests')
    args = parser.parse_args()
    if args.worker_only and args.kind != 'bins':
        parser.error('--worker-only requires --kind bins')
    if args.with_worker and (args.kind != 'bins' or args.mode != 'plain' or args.worker_only or not args.probe_bin):
        parser.error('--with-worker requires plain bins with --probe-bin and cannot combine with --worker-only')
    if args.probe_bin and (args.kind != 'bins' or args.mode != 'plain' or args.worker_only):
        parser.error('--probe-bin requires plain bins and cannot combine with --worker-only')
    if args.package_official and (args.kind != 'bins' or args.mode != 'plain' or args.worker_only):
        parser.error('--package-official requires plain bins and cannot combine with --worker-only')
    if args.server_lib_only and (args.kind != 'lib-tests' or args.mode != 'plain' or args.with_probe_integration):
        parser.error('--server-lib-only requires plain library tests and cannot combine with probe integration')
    if args.with_probe_integration and args.kind != 'lib-tests':
        parser.error('--with-probe-integration requires --kind lib-tests')
    out = args.out.resolve()
    if out.exists():
        raise ValueError('new exclusive prepared build directory required')
    out.mkdir(parents=True, mode=0o700)
    # The rustup Cargo launcher otherwise injects its chosen toolchain only
    # after this process has recorded the environment. Resolve it once and
    # explicitly use that same selection for every compiler/tree/build call.
    env = os.environ.copy()
    selected = subprocess.check_output(['rustup','show','active-toolchain'],
        cwd=ROOT, env=env, text=True).strip().rsplit(' (', 1)[0]
    if not selected or '\n' in selected:
        raise ValueError('actual rustup toolchain selection is unavailable')
    env['RUSTUP_TOOLCHAIN'] = selected
    compiler = subprocess.check_output([env.get('RUSTC','rustc'),'-Vv'], cwd=ROOT, env=env, text=True)
    target = next(line.split(': ',1)[1] for line in compiler.splitlines() if line.startswith('host: '))
    selection = [arg for package in PACKAGES for arg in ['-p',package]]
    features = [] if args.mode == 'plain' else ['--features','game_server/cmva-lab-measurement,u11_independent_public_probe/instrumented']
    tree_command = ['cargo','tree','--locked','--offline','--target',target,'--prefix','depth','--no-dedupe','--format','{p}|{f}','--edges', 'normal,build' if args.kind == 'bins' else 'normal,build,dev'] + selection + features
    tree = subprocess.check_output(tree_command,cwd=ROOT,env=env,text=True)
    (out/'cargo-tree.txt').write_text(tree)
    graph = core_graph(tree)
    targets = (['--bin','mod_launcher'] if args.worker_only else ['--bins']) if args.kind == 'bins' else ['--no-run','--lib']
    if args.probe_bin:
        targets = [arg for name in args.probe_bin for arg in ['--bin',name]]
    if args.package_official:
        targets = (targets if args.probe_bin else []) + ['--bin','package_official_content']
    if args.with_worker:
        targets += ['--bin', 'mod_launcher']
    if args.with_probe_integration:
        for name in ('registry_contract','required_configuration','calibration_authority'):
            targets += ['--test',name]
    build_selection = ['-p','game_server'] if args.server_lib_only else selection
    selected_tree_command = tree_command if not args.server_lib_only else tree_command[:-len(selection)] + build_selection
    selected_tree = tree if not args.server_lib_only else subprocess.check_output(selected_tree_command,cwd=ROOT,env=env,text=True)
    if core_graph(selected_tree) != graph:
        raise ValueError('selected target effective Core dependency graph differs from supported matrix')
    (out/'selected-cargo-tree.txt').write_text(selected_tree)
    command = ['cargo', 'build' if args.kind == 'bins' else 'test','--release','--locked','--offline','-j','1','--target',target,'--target-dir',str(args.target_dir.resolve())] + build_selection + features + targets
    proof = {'schema':'cmva.prepared-build/v1','workspace_root':str(ROOT), 'compiler':compiler,
      'target':target,'profile':'release','mode':args.mode,'kind':args.kind,'packages':PACKAGES,
      'core_features': '' if args.mode == 'plain' else 'RUNTIME_OBSERVATION',
      'files': {path:sha((ROOT/path).read_bytes()) for path in MANIFESTS},
      'environment':{name:env.get(name,'') for name in ENVIRONMENT},
      'tree_command':tree_command,'tree_path':str(out/'cargo-tree.txt'),'tree_sha256':sha(tree.encode()),
      'core_dependency_graph_sha256':sha(graph.encode()),'build_command':command,'build_packages':['game_server'] if args.server_lib_only else PACKAGES,
      'selected_tree_command':selected_tree_command,'selected_tree_path':str(out/'selected-cargo-tree.txt'),
      'selected_tree_sha256':sha(selected_tree.encode())}
    proof_path = out/'prepared-build.json'
    proof_path.write_text(json.dumps(proof,indent=2)+'\n')
    env['CMVA_PREPARED_BUILD'] = str(proof_path)
    if args.prepare_only:
        print(json.dumps({'prepared_build':str(proof_path),'sha256':sha(proof_path.read_bytes()),'command':command,'build_executed':False}))
        return 0
    result = subprocess.run(command,cwd=ROOT,env=env)
    print(json.dumps({'prepared_build':str(proof_path),'sha256':sha(proof_path.read_bytes()),'command':command,'exit_code':result.returncode}))
    return result.returncode
if __name__ == '__main__':
    raise SystemExit(main())
