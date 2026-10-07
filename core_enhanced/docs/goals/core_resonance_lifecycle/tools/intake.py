#!/usr/bin/env python3
import hashlib,json,subprocess,tarfile,tempfile
from pathlib import Path
from initialize import HERE,ROOT
from run import identity,sha
assert (HERE/'P2G_60_RESULT_001.json').is_file()
d=HERE/'execution/single_goal';assert not d.exists();d.mkdir(parents=True)
for x in ['baseline','evidence']:(d/x).mkdir()
for r in json.loads((HERE/'evidence/REVIEW_SOURCE_MANIFEST_002.json').read_text())['files']:assert sha(ROOT/r['path'])==r['sha256']
b=identity()
for rel in ['core_enhanced/docs/core_runtime_contract.md','core_enhanced/docs/data_loading_contract.md','core_enhanced/docs/skill_target_contract.md','core_enhanced/docs/game_rulebook.md','core_enhanced/docs/README.md','core_enhanced/docs/roadmaps/core_first/ROADMAP.ko.md','game_server/README.md','game_server/docs/unity_bridge_protocol.md']:
 p=ROOT/rel
 if p.is_file() and not any(r['path']==rel for r in b['files']):b['files'].append({'path':rel,'bytes':p.stat().st_size,'sha256':sha(p)})
b['files'].sort(key=lambda r:r['path']);b['sha256']=hashlib.sha256(json.dumps(b['files'],sort_keys=True).encode()).hexdigest()
(d/'baseline/SOURCE_BEFORE.json').write_text(json.dumps(b,indent=2)+'\n')
with tarfile.open(d/'baseline/SOURCE_BEFORE.tar.gz','x:gz') as archive:
 for r in b['files']:archive.add(ROOT/r['path'],arcname=r['path'],recursive=False)
with tempfile.TemporaryDirectory(prefix='u05-before-restore-') as tmp:
 with tarfile.open(d/'baseline/SOURCE_BEFORE.tar.gz') as archive:archive.extractall(tmp,filter='data')
 for r in b['files']:assert sha(Path(tmp)/r['path'])==r['sha256']
(d/'baseline/ARCHIVE.json').write_text(json.dumps({'sha256':sha(d/'baseline/SOURCE_BEFORE.tar.gz'),'source_sha256':b['sha256'],'files':len(b['files']),'actual_extracted_bytes_verified':True},indent=2)+'\n')
(d/'baseline/GIT_STATUS.txt').write_text(subprocess.check_output(['git','status','--short'],cwd=ROOT,text=True))
(d/'PLAN.md').write_text('''# GOAL-CRSL-01 실행

GEX revision2026-09-13-gex-r7; P2G current r7/quality2. Core-first U05/B02,B03,B04,B09.
Mode goal-run: 원 연속 요청 ../../P2G_00_INITIAL_001.json IN-001, CF-D06/07 위임.
명시적 독립 감사·finding 교정·다음 owner 전환까지 승인됨. 중간 단계검토 opt-in 없음.
Exact member GS-CRSL-001/GOAL-CRSL-01; PHASE-CRSL-01;
AG-CRSL-RESONANCE + WP-CRSL-VERIFY. Accepted HA-CRSL-001/FPF-CRSL-001/PP-CRSL-001/PVR-CRSL-001.
Exact refs/digests는 ../../P2G_60_HANDOFF_001.json과 P2G40/50/60 result가 소유한다.

## Current execution state

GEX-00 complete; GEX-10 next. U01/U04 independent CompleteHigh를 인수했다.
현재 member에 기존 PLAN/EXPERIMENT/AUDIT가 없는 신규 실행임을 좁힌 경로 검색과 생성 이력으로 확인.
이 디렉터리 하나가 실행 문서 owner다. PLAN=current scope/basis/design/progress,
EXPERIMENTS=실제 시도·검증, EXPERIMENT_NOTES=선택·실패/후속 위치.
독립 감사는 구현 후 별도 post_audit 디렉터리에서 작성한다. U05 감사 verdict는 아직 없다.

baseline SOURCE_BEFORE archive/index와 ARCHIVE는 실제 dirty before bytes를 보존하고 추출 hash까지 확인했다.
기존 사용자 변경·선행 구현·과거 감사는 reset/stash/일괄정리하지 않는다. 150 reviewed source unchanged.
초기 검증은 ../../evidence/PREDECESSOR_BASELINE_001.json의 U04 final367 product unchanged 근거를 재사용:
Core1734/1734/1743,host89/97+heap1,fmt/check/event-log/clippy/release,16public consumer/PC576와
actual continuation/rewind/fault 증거. U05 변경 후 검증으로 주장하지 않는다.

## Implementation and readiness

정책 원본은 Freeze14요소, 구조/actual/검증 seam은 ../../CODE_REVIEW_002.ko.md RQ01–07 TD01–11 V01–14.
DEFINITION→ADMISSION→LIFETIME→PROJECTION→VERIFY를 root가 순차 구현한다.
네 구현 WP는 최소 atomic 전환이며 중간 코드를 독립 완료로 게시하지 않는다.
직원 base20%와 유효 InitialSkillCharge/배치 효과, 파편별max와 적 autonomous 별도 owner를 유지한다.
입력 최신검증→소비/자격→기존 start/corrosion timing→actual-use 종료→all-positive gain→query/restore를
한 owner에 연결한다. effects settlement·resource scope와 use lock을 합치지 않는다.

첫 product mutation 전에 구체 descriptor/compiled future-control 조건, credential state key와 consume fact,
finite buff/channel 실제 lease, pulse scheduling/CC overlap, cancellation/released payload와 extent/clone hooks,
strict authoring/public projection의 타입·함수·순서·시험 기대를 이 PLAN에 추가하고 준비 점검한다.
공개 source/host/local fixture와 Rust toolchain/cache가 제공되며 operational store/배포 mutation은 필요 없다.
필수 policy/source gap 없음. exact 내부 실현 선택은 TD01–11 안의 GEX 책임이다.

## Validation and completion

V01–14와10 required dimensions를 actual public RON→normal command/query 및 kernel boundary로 검증한다.
공식25개+Flow/sustained 공개 fixture 전종, slow projectile/combo/timedbuff/channel/independent 생성,
Pause input/0잔량start/중복/취소무환불/stale preview/empty tile/accepted anchor/corrosion actual timing,
양수gate/기본gain제거/RestHP/초기충전효과/적주기, clone rollback·same actor reconnect·U04 Continue가 포함된다.
선행127조건+별도31R06의 changed/reuse/focused map, full Core/host/native/observer/strict/public release
검증을 완료한다. 실패/zero/skip은 보존하고 필요한 수정·재실행만 수행한다. R05 approved peak exception과
20분 제외를 유지하되 새 실패를 숨기지 않는다. 실행 후 source 변경은 해당 evidence를 invalid 처리한다.

완료 후 exact source/data/docs/evidence target과 before/after fuzz0 forward/reverse 복원을 고정한다.
변경없는 선행 증거는 exact external archive/member/digest로 인수해 중복 아카이브를 피한다.
제품 mutation을 멈춘 뒤 구현 비참여 GPT-6 Astra/High가 독립 GEX20을 수행한다.
finding을 교정·재감사하여 CompleteHigh 전에는 U06 종속 구현을 시작하지 않는다.
Single Phase이므로 Master/GEX30 없음. 사용자 stop/승인 로드맵 완료/현 권한·환경으로 해결불가 실제 장애에서만 멈춘다.
''')
(d/'EXPERIMENTS.md').write_text('# U05 실행·검증\n\nP2G40/50 표준 구조/quality2 PASS, 의미 검토와 P2G60 인계 완료.\nGEX00 actual dirty before archive와 실제 추출 hash 확인. Product mutation 없음.\n초기 full baseline은 exact U04 closed evidence 재사용; U05 acceptance 실행은 아직 없음.\n')
(d/'EXPERIMENT_NOTES.md').write_text('# U05 기술 판단과 실패\n\nP2G20 initial candidate001은 base 초기량+InitialSkillCharge 및 직원/적 autonomous 적용 범위 발견으로 abort했다. P2G10 successor에서3요소 보완/11exact 보존 후 complete review002를 고정했다. 원본 보존.\n현재 지원 snapshot은 인메모리 candidate/같은 actor reconnect 및 U04 confirmed disk state다. 새 live battle disk 저장을 추가하지 않는다.\n구체 구현 선택은 PLAN에 준비 점검과 함께 추가한다.\n')
p=HERE/'WORKING_POLICY.ko.md';p.write_text(p.read_text()+'\n[GEX-00 인수](execution/single_goal/PLAN.md) 완료. actual before archive 추출 검증. 현재 GEX-10 상세 설계/구현 준비; U05 독립 감사 전 U06 구현 없음.\n')
print('U05 GEX00 complete:',len(b['files']),'actual before files archived and extraction verified;GEX10 next')
