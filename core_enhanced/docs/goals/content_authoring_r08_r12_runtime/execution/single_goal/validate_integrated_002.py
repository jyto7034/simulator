"""Run the required final matrix; retain the known environment refusal separately."""
import json, re, subprocess, sys
from pathlib import Path
H=Path(__file__).resolve().parent
SKIP='game::data::content_pack::snapshot::tests::unix::invalid_utf8_directories_and_socket_inputs_return_errors'
def run(name,args):
    code=subprocess.run([sys.executable,str(H/'run.py'),name,'--',*args]).returncode
    receipt=json.loads((H/'evidence'/f'{name}.json').read_text())
    assert receipt['source_before']['digest']==receipt['source_after']['digest']
    return code
for name,feature in [('integrated_all_002',[]),('integrated_feature_002',['--features','test-fixtures'])]:
    args=['cargo','test','-p','game_core','--all-targets',*feature,'--offline','--locked']
    if run(name,args):
        log=(H/'evidence'/f'{name}.log').read_text()
        failed=re.findall(r'^test (\S+) \.\.\. FAILED$',log,re.M)
        assert failed==[SKIP], f'Unexpected failure: {failed}'
        assert 'Operation not permitted' in log
        assert run(name+'_environment_excluded',args+['--','--skip',SKIP])==0
for name,args in [('integrated_check_002',['check','--all-targets']),('integrated_event_check_002',['check','--all-targets','--features','event_log_checks']),('integrated_release_002',['build','--release','--bins']),('integrated_clippy_002',['clippy','--all-targets'])]:
    command=['cargo',args[0],'-p','game_core',*args[1:],'--offline','--locked']
    if args[0]=='clippy':command+=['--','-A','clippy::never_loop','-A','clippy::eq_op']
    assert run(name,command)==0, name
assert run('integrated_fmt_002',['cargo','fmt','--all','--','--check'])==0
assert run('integrated_public_export_002',['env',f'R12_PUBLIC_CLI_EXPORT={H}/evidence/r12_public_cli_002','cargo','test','-p','game_core','--lib','--offline','--locked','r12_public_'])==0
