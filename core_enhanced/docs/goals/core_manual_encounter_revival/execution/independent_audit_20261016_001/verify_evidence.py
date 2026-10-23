"""Independent raw-result, process/trace/binding and cost arithmetic checks."""
from pathlib import Path
import hashlib
import json
import re
import statistics
from functools import lru_cache

A=Path(__file__).resolve().parent
E=A.parent
S=E/'storage_refactor'
O=E/'single_goal'
load=lambda p:json.loads(p.read_bytes())
@lru_cache(None)
def sha(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def tests(path):
    text=path.read_text()
    summaries=re.findall(r'test result: (\w+)\. (\d+) passed; (\d+) failed; (\d+) ignored;',text)
    names=dict(re.findall(r'^test (\S+) \.\.\. (ok|FAILED|ignored)\s*$',text,re.M))
    return summaries,names


checked_tests=[]
for row in load(S/'CURRENT_RESULTS.json')['tests']:
    p=S/row['log']
    summaries,names=tests(p)
    assert len(summaries)==1,p
    status,passed,failed,ignored=summaries[0]
    assert [int(passed),int(failed),int(ignored)]==[row['passed'],row['failed'],row['ignored']],p
    if row['failed']:
        assert [n for n,s in names.items() if s=='FAILED']==['game::data::content_pack::snapshot::tests::unix::socket_inputs_return_errors']
    checked_tests.append(row)
_,socket=tests(S/'runs/socket-009.log')
assert socket['game::data::content_pack::snapshot::tests::unix::socket_inputs_return_errors']=='ok'
fresh_summaries,fresh_names=tests(A/'focused003.log')
assert fresh_summaries==[('ok','31','0','0')] and len(fresh_names)==31
baseline=load(O/'BASELINE_REUSE.json')
closed={}
for name in baseline['failure_closure']['logs']:
    _,names=tests(O/name)
    closed.update(names)
for name in baseline['failure_closure']['original_failed_names']:
    assert closed.get(name)=='ok',name

process_groups={}
for group in ['normal-forward','expanded-forward','normal-fresh','expanded-fresh','normal-io','expanded-io','old-writer-fresh','terminal-terminal']:
    directory=S/'runs/current001'/group
    assertion=load(directory/'ASSERTIONS.json')
    assert assertion['passed'],group
    receipts=sorted(directory.rglob('PROCESS.json'))
    assert receipts,group
    total,io_refusals,rejections=0,0,0
    for receipt in receipts:
        p=load(receipt)
        assert p['exit_code']==0,receipt
        assert sha(p['argv'][0])==p['binary_sha256'],receipt
        assert sha(p['argv'][1])==p['config_sha256'],receipt
        trace=[json.loads(line) for line in (receipt.parent/'trace.jsonl').read_text().splitlines()]
        requests=[json.loads(line) for line in (receipt.parent/'requests.jsonl').read_text().splitlines()]
        assert len(trace)==len(requests)==p['operations'],receipt
        terminal=load(receipt.parent/'RESULT.json')
        assert terminal['protocol_completed'] and terminal['operations']==p['operations'],receipt
        for request,row in zip(requests,trace):
            assert request==row['request'],receipt
            fail=request.get('fail',False)
            reject=request.get('reject',False)
            assert row['response']['accepted']==(not fail and not reject),receipt
            if fail:
                assert 'os error 21' in row['response']['error'],receipt
            io_refusals+=bool(fail)
            rejections+=bool(reject)
        total+=len(trace)
    process_groups[group]=dict(processes=len(receipts),operations=total,actual_io_refusals=io_refusals,explicit_rejections=rejections)
    if group.endswith('-forward'):
        assert total==333
    if group in ['normal-fresh','expanded-fresh','old-writer-fresh']:
        assert len(assertion['results'])==14
    if group in ['normal-io','expanded-io']:
        assert len(assertion['failures'])==17 and io_refusals==17
    if group=='terminal-terminal':
        assert len(assertion['results'])==2 and io_refusals==2

cost=load(S/'COST_REVIEW.json')
binding=load(S/'runs/current001/cost/BINDINGS.json')
for mode,files in binding.items():
    for path,digest in files.items():
        assert sha(path)==digest,path
count=0
for row in cost['measurements']:
    mode=row['mode']
    for i in range(3):
        forward=load(S/f'runs/current001/cost/{mode}.forward.{i}.json')
        restore=load(S/f'runs/current001/cost/{mode}.restore.{i}.json')
        assert forward['passed'] and forward['operations']==333 and forward['exact_requested_bytes_recovered']
        assert restore['passed'] and restore['matches_published_progress'] and restore['exact_requested_bytes_recovered']
        assert row['forward'][i]['operation_seconds']==sum(s['seconds'] for s in forward['stages'] if s['kind']=='operation')
        assert row['public_continue'][i]['seconds']==restore['seconds']
        assert row['public_continue'][i]['heap_window_growth_bytes']==restore['heap']['peak_requested_bytes']-restore['heap']['baseline']['live_requested_bytes']
        count+=2
prior=load(O/'COST_REVIEW.json')
for comparison,old,new in zip(cost['comparison'],prior['measurements'],cost['measurements']):
    assert comparison['mode']==old['mode']==new['mode']
    for family in ['forward','public_continue']:
        for key,values in comparison[family].items():
            assert values['before_median']==statistics.median(x[key] for x in old[family])
            assert values['after_median']==statistics.median(x[key] for x in new[family])
assert count==12
restore=load(A/'restore_probe/SUMMARY.json')
assert restore['passed'] and restore['processes']==4
result=dict(passed=True,new_focused=dict(passed=31,failed=0,ignored=0),reused_tests=checked_tests,
    original_failed_tests_closed=len(baseline['failure_closure']['original_failed_names']),
    public_groups=process_groups,cost_processes_reused=count,new_restore_processes=4,
    authored_rejection=dict(path='kind-package001.stderr',gameplay_executed=False,product_finding=False),
    scope='raw summaries/names, process exits, command/response traces, IO error21, binary/config hashes and cost arithmetic; implementation assertions independently read in audit')
with (A/'VERIFIED_RESULTS.json').open('x') as f:
    json.dump(result,f,ensure_ascii=False,indent=2)
    f.write('\n')
print(json.dumps(result,ensure_ascii=False,indent=2))
