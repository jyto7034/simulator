#!/usr/bin/env python3
import hashlib
import json
import subprocess
import tarfile
import tempfile
from pathlib import Path
from initialize import HERE, ROOT
from run import identity, sha

assert (HERE/'P2G_60_RESULT_001.json').is_file()
d = HERE/'execution/single_goal'
assert not d.exists()
d.mkdir(parents=True)
for name in ['baseline','evidence']:
    (d/name).mkdir()
idx = json.loads((HERE/'SOURCE_INDEX_002.json').read_text())
for row in idx['files']:
    assert sha(ROOT/row['path']) == row['sha256'], row['path']
b = identity()
for rel in ['core_enhanced/docs/core_runtime_contract.md','core_enhanced/docs/data_loading_contract.md',
            'core_enhanced/docs/README.md','core_enhanced/docs/roadmaps/core_first/ROADMAP.ko.md',
            'game_server/README.md','game_server/docs/unity_bridge_protocol.md']:
    path=ROOT/rel
    if path.is_file() and not any(row['path']==rel for row in b['files']):
        b['files'].append({'path':rel,'bytes':path.stat().st_size,'sha256':sha(path)})
b['files'].sort(key=lambda row:row['path'])
b['sha256']=hashlib.sha256(json.dumps(b['files'],sort_keys=True).encode()).hexdigest()
(d/'baseline/SOURCE_BEFORE.json').write_text(json.dumps(b,indent=2)+'\n')
with tarfile.open(d/'baseline/SOURCE_BEFORE.tar.gz','x:gz') as archive:
    for row in b['files']:
        archive.add(ROOT/row['path'],arcname=row['path'],recursive=False)
with tempfile.TemporaryDirectory(prefix='u04-before-restore-') as temporary:
    with tarfile.open(d/'baseline/SOURCE_BEFORE.tar.gz') as archive:
        archive.extractall(temporary,filter='data')
    for row in b['files']:
        assert sha(Path(temporary)/row['path']) == row['sha256']
(d/'baseline/ARCHIVE.json').write_text(json.dumps({'sha256':sha(d/'baseline/SOURCE_BEFORE.tar.gz'),'source_sha256':b['sha256'],'files':len(b['files']),'actual_extracted_bytes_verified':True},indent=2)+'\n')
(d/'baseline/GIT_STATUS.txt').write_text(subprocess.check_output(['git','status','--short'],cwd=ROOT,text=True))
(d/'baseline/CURRENTNESS.json').write_text(json.dumps({'predecessor':'../../evidence/PREDECESSOR_BASELINE_001.json','predecessor_receipt_sha256':sha(HERE/'evidence/PREDECESSOR_BASELINE_001.json'),'review_source_index_sha256':sha(HERE/'SOURCE_INDEX_002.json'),'reviewed70_sources_unchanged':True,'baseline':'U03 independent audit003 Complete/High;363 product unchanged; authorized ROADMAP status-only successor separately bound. Initial tests reused only; actual U04 changes invalidate affected seams.','production_mutation':False},indent=2)+'\n')
(d/'PLAN.md').write_text('''# GOAL-CRRP-01 실행

GEX revision2026-09-13-gex-r7; P2G r7 quality2. Core-first U04/S01–08.
Mode goal-run: original continuous user request ../../P2G_00_INITIAL_001.json IN-001;
CF-D06 policy delegation and CF-D07 command evolution. No requested intermediate stop.
Exact member GS-CRRP-001 / GOAL-CRRP-01, PHASE-CRRP-01,
AG-CRRP-RESULT-PERSISTENCE + WP-CRRP-VERIFY.
Accepted upstream ../../P2G_60_HANDOFF_001.json HA-CRRP-001,
../../P2G_50_GOAL_SET.ko.md, FPF-CRRP-001 / PP-CRRP-001 / PVR-CRRP-001.
Exact references and digests are owned by the Handoff and P2G40/50/60 result records.

## Current execution state

GEX-00 complete; GEX-10 next. U03 independent audit003 closed Complete/High before U04.
No prior exact member execution records existed: bounded PLAN/EXPERIMENTS/NOTES/AUDIT search and
new member/directory creation history checked. This directory is the sole execution-document owner.
Role paths: this PLAN.md=current scope/basis/design/progress; EXPERIMENTS.md=actual attempts/evidence;
EXPERIMENT_NOTES.md=technical choices/failures; independent audit output is a fresh post_audit child,
referenced here when produced. No independent verdict exists for U04 yet.

baseline/SOURCE_BEFORE.tar.gz and SOURCE_BEFORE.json preserve actual dirty before bytes;
ARCHIVE.json binds the archive and successful exact extraction. No reset/stash/reformat of user work.
70 reviewed sources remain exact. Initial full baseline is the exact U03 final Core1717/1717/1726,
host81/88+allocator1 and actual16 public consumers/PC576, fmt/check/event-log/clippy/release,
bound through ../../evidence/PREDECESSOR_BASELINE_001.json. One status-only ROADMAP successor
is explicitly attributed to U03 closure; product363 unchanged. This is not post-U04 validation.

## Implementation and readiness

Scope/meaning and current source design: ../../CODE_REVIEW_001.ko.md RQ01–08 TD01–08 V01–15;
exact Goal and Freeze15 elements. Internally implement RESULT→RESTORE→DURABLE→HOST→VERIFY.
All four producer/schema/file/consumer packages are one atomic delivery unit; partial code is not
independent completion. Existing GameCore prepared root and common grant/employee/node planners
remain sole owners. Existing standard completion journal IO is expanded, not a second domain engine.

Before changing each owner, refine its concrete receipt/phase validation, nonrecursive immutable
resume root, retreat overlay versus explicit checkpoint rewind, exclusive prepare API, journal
revision/tombstone and lock/publication order. Preserve native capture/work/poll and U03 origin gates.
Fallible work finishes before durable intent; intent acceptance is the irreversible result boundary
and later IO/transport failure is recovery-pending for the same accepted result. No fake domain defeat.
Compact receipt stores current result/choice summary, not full runtime/event log. New codecs include
only current state and strict references; V4/V9/unsupported originals are preserved and refused.

Ready inputs: Core source/adapters/actual stock host/tests and compiler available in current workspace;
local configured persistent_data directories absent, fixtures use isolated temp roots. No operational
file mutation/deployment is needed. Existing Rust build cache/toolchain may be reused. No new policy
question or required inaccessible source. Concrete helper and fixture names remain GEX local choices.

## Validation and completion

Own V01–15 durable fixtures before independent audit: actual manual/tick immediate result and ack
restart, fixed ChooseOne2/0/context/retry, success retreat2 and third challenge after restart, every
current unfinished node's pre-entry restoration, first-SavePoint-independent Continue, trauma/load3
and backup monotonicity, employee once/death/consumable, same-process reconnect/U03 faults, real file
subprocess crash every publication stage, concurrent claims/stale revision/terminal ABA, strict legacy
original protection and malformed import, public Core/stock websocket/observer parity.
Full appropriate default/test-fixtures/observation Core and host matrix, allocator,fmt/check/event-log/
clippy/release and affected inherited112 + separate R06 semantic31. Ten dimensions required.
Retain raw failed/zero runs and the approved prior R05 concurrent-peak exception; no new arbitrary cap.
No performance/native work-accounting regression is assumed away because source tests stayed unchanged.

After self-verification, capture exact source/data/doc/evidence target and exercise fuzz0 forward and
reverse restoration. Changed actual sources must be archived; unchanged predecessor proof can be
external exact path+digest/member bindings to avoid recursive archive growth. All original audits stay.
Fresh implementation-nonparticipant GPT-6 Astra / High GEX20 must close Complete/High; implement and
re-audit findings under existing authority. No U05 dependent implementation before independent pass.
Single-phase Goal has no Master/GEX30. Stop only for user stop, approved roadmap completion, or an
actual unresolvable current authority/environment blocker. Intermediate reports do not stop work.
''')
(d/'EXPERIMENTS.md').write_text('# U04 실행·검증 기록\n\nP2G40 receipt와 P2G50 quality2 표준 검증/의미 검토 PASS; P2G60 exact 인계 완료.\nGEX00 before actual archive와 추출 hash 확인 완료. Production mutation 없음.\n초기 baseline은 exact U03 closed evidence 재사용이며 U04 새 acceptance 실행은 아직 미수행.\n')
(d/'EXPERIMENT_NOTES.md').write_text('# U04 기술 판단·실패\n\nP2G20 initial reviewed refs의 PE-* 이름을 표준 Goal reader PER-*에 맞춰 동결 전에 immutable P2G10/20 successor로 교정했다. 원본 보존, 정책/결정/acceptance 의미 불변.\nBaseline367 모두 불변 가정은 기존 U03 ROADMAP status publication을 검출하여 authorized successor를 명시 결속했다. 제품363은 불변.\n구체 구현 선택은 CODE_REVIEW TD01–08의 owner/수명/원자성/검증 경계 안에서 아래에 기록한다.\n')
with (HERE/'WORKING_POLICY.ko.md').open('a') as out:
    out.write('\n[GEX-00 PLAN](execution/single_goal/PLAN.md)과 실제 before archive 인수 완료. 현재 GEX-10 구현 시작; 새 독립 감사 완료 전 U05 구현 없음.\n')
print('U04 GEX00 complete:',len(b['files']),'actual before files archived and extracted/verified; GEX10 next')
