#!/usr/bin/env python3
import hashlib,json,subprocess,tarfile,tempfile
from pathlib import Path
from initialize import HERE,ROOT
from run import identity,sha
assert json.loads((HERE/'P2G_60_RESULT_001.json').read_text())['status']=='complete'
d=HERE/'execution/single_goal';assert not d.exists();d.mkdir(parents=True)
for x in ['baseline','evidence']:(d/x).mkdir()
for r in json.loads((HERE/'evidence/REVIEW_SOURCE_MANIFEST_003.json').read_text())['files']:assert sha(ROOT/r['path'])==r['sha256']
b=identity()
previous=json.loads((ROOT/'core_enhanced/docs/roadmaps/core_first/execution/u06_closure_20261005_001/PUBLICATION_RECORD.json').read_text())['source_after']
assert b==previous,'Product baseline drift: classify before implementation'
(d/'baseline/PRODUCT_BEFORE.json').write_text(json.dumps(b,indent=2)+'\n')
extra=[str(p.relative_to(ROOT)) for p in sorted((ROOT/'core_enhanced/docs/game_rules').glob('*.md'))]
extra+=['core_enhanced/docs/core_runtime_contract.md','core_enhanced/docs/data_loading_contract.md','core_enhanced/docs/skill_target_contract.md','core_enhanced/docs/game_rulebook.md','core_enhanced/docs/README.md','core_enhanced/docs/roadmaps/core_first/ROADMAP.ko.md','game_server/README.md','game_server/docs/unity_bridge_protocol.md']
for rel in extra:
 p=ROOT/rel
 if p.is_file() and not any(r['path']==rel for r in b['files']):b['files'].append({'path':rel,'bytes':p.stat().st_size,'sha256':sha(p)})
b['files'].sort(key=lambda r:r['path']);b['sha256']=hashlib.sha256(json.dumps(b['files'],sort_keys=True).encode()).hexdigest()
(d/'baseline/SOURCE_BEFORE.json').write_text(json.dumps(b,indent=2)+'\n')
with tarfile.open(d/'baseline/SOURCE_BEFORE.tar.gz','x:gz') as archive:
 for r in b['files']:archive.add(ROOT/r['path'],arcname=r['path'],recursive=False)
with tempfile.TemporaryDirectory(prefix='u07-before-restore-') as tmp:
 with tarfile.open(d/'baseline/SOURCE_BEFORE.tar.gz') as archive:archive.extractall(tmp,filter='data')
 for r in b['files']:assert sha(Path(tmp)/r['path'])==r['sha256']
(d/'baseline/ARCHIVE.json').write_text(json.dumps({'sha256':sha(d/'baseline/SOURCE_BEFORE.tar.gz'),'source_sha256':b['sha256'],'files':len(b['files']),'actual_extracted_bytes_verified':True},indent=2)+'\n')
(d/'baseline/GIT_STATUS.txt').write_text(subprocess.check_output(['git','status','--short'],cwd=ROOT,text=True))
(d/'PLAN.md').write_text('''# GOAL-CIOB-01 실행

적용 GEX2026-10-04-gex-r8/P2G2026-10-04-r8/quality2/usagev2.1.3; 기존 운영 인수와 현 연속 요청을 유지한다.
Mode goal-run. 원 연속 요청은 ../../P2G_00_INITIAL_001.json의 현 요청이며 core_first U07/B13·C01의 P2G/GEX·검증·비참여AstraHigh감사·교정·다음eligible작업까지 승인했다.
중간 단계검토 opt-in 없음. 단계/감사 인계마다 재승인을 요청하지 말라는 사용자 지시가 기본 checkpoint보다 우선한다.
Selected exact GS-CIOB-001/GOAL-CIOB-01/PHASE-CIOB-01. HA-CIOB-001/FPF-CIOB-001/PP-CIOB-001/PVR-CIOB-001 exact chain은 ../../P2G_60_HANDOFF_001.json 및 P2G40/50/60 결과가 소유한다.

## Current execution state and roles

GEX-00 complete; GEX-10 상세화/구현이 다음이다. 제품 mutation/새 acceptance 실행/독립 감사는 아직 없다.
본 세션에서 새 policy identity를 도입해P2G00→60을 직접 작성했고 exclusive execution directory는 이전 생성되지 않았다. 선행U06의 실행·감사와 구별한다.
Canonical roles: core_enhanced/docs/goals/core_independent_objects/execution/single_goal/PLAN.md=current exactbasis/scope/design/progress; 같은 디렉터리 EXPERIMENTS.md=실제검증, EXPERIMENT_NOTES.md=선택/실패/재개위치. 최종독립감사는 동결 뒤 별도 post_audit 아래 기록한다.
진행중cargo/rustc/runner없음; 기존 u06_audit는 완료상태다. 구현은 root 혼자 순차 수행하며 기존 비참여 감사자는 독립성을 유지한다.

## Baseline and authorization

baseline/PRODUCT_BEFORE.json은U06 finalproduct375파일 b33a8a019087f3b6766c8cf611636f46246fa3d895464b5a439f6d176694a401과 exact동일하다. SOURCE_BEFORE.json/tar.gz는 canonicaldocs포함현재dirtybytes이고 실제추출hash확인했다.146reviewcapture도불변이다.
PREDECESSOR_INTAKE_001은U06 final002 Complete/High와현황2문서delta의비영향을인수했다. 과거감사/실패/사용자변경을reset/stash/일괄정리하지않는다.
초기 fullbaseline은exactU06 Core1794/1794/1803,host92/100+heap1와45selectedreceipts(15fresh/30명시재사용)의유효근거를인수한다. U07의새실행성공으로표시하지않는다. 기존R05 peak-only경고/20분stress제외를유지한다.

## Implementation readiness

Freeze20와CODE_REVIEW_001 RQ01–08/TD01–08/Element01–20 및PhasePlan6nodes가정의한경계를따른다.
DEFINITIONS→CREATION→SPATIAL/SEGMENTS→PUBLIC→VERIFY. 새strictschema/objectsource/epoch/queue/pool은처음부터clone/retained/work/admission을포함한다.
정상tilecast→commonSpawnKernel candidate→cost/object/ID/source/queue원자commit; genericactor는실제side/독립행동권한을사용한다.
trap은별도object,wall은명시HPadapterrole로시설목표/employee/heal/deathproc권한을분리하고원래path접촉→공격→재개한다.
field는기존fixedTileArea를재사용한다. 친구는snapshot피해/유효공격성공소모/같은copy진화epoch; sourcePersist와부모사용잠금을분리한다.
분절pool은기존sustainedremaining하나를truth로하고stack/top/blockbonus를파생한다. partialdetach또는nonlethalHPfallback과wall생성은같은candidate다.
Core sharedeligibility→read-onlypreview/query→normalWorld/trial/host; liveclone/pausedreconnect와U04pre-entryContinue를유지한다.
구현전exactpayload/state/sourcecredentials/queue/accounting와초기authored값/작은검증을아래에상세화한다. 제공된localrepo/Rustcache/정상RON/host로진행가능하고운영서비스/개인DB/remote변경필요없다. 현재필수policy/sourcegap0.

## Validation and completion

strict입력→새상태크기/clone/queue/atomicfault작은검사→기능별→공식/외부정상CoreApplication/trial/host→필수최종통합.
V01–20 exactacceptance와10required차원,161선행조건+별도31R06각각impact/currentevidence/reuse를연결한다.
실제actor/trap/field/wall/진화/pool/HPcost의nonzero와경계·latefailure·duplicate·CC/Pause·source제거·동시각·clone·재접속·저장을검증한다.
최종Core(default/fixtures/observation),host(default/observation/heap),fmt/check/event-log/clippy/release/publicPC/nativefault/observer/allocator및affectedrestore를현재integratedtarget에서닫는다. 원래숫자oracle/실패raw를보존하고영향이분리된선행근거만재사용한다.
자체완료후exactsource/data/docs/evidence와정역복원patch를고정해제품변경을멈추고비참여GPT-6AstraHigh가GEX20전체감사한다. finding교정·재감사Complete/High/open0전U08종속구현금지. single_phase이므로MasterGEX30없음.
사용자stop/승인범위완료/현재권한환경에서더진행불가실제장애에서만중단한다.
''')
(d/'EXPERIMENTS.md').write_text('# U07 실행·검증\n\nP2G40:20members/6nodes/11edges/1phase/20seams 구조PASS. P2G50quality2:20traceability/10dimensions PASS. 의미검토/P2G60완료.\nGEX00:exactU06product375현재성과146reviewcapture확인;canonicaldoc포함beforearchive실제추출hash확인. 제품테스트새실행없음;선행baseline은명시재사용.\n')
(d/'EXPERIMENT_NOTES.md').write_text('# U07 기술 판단·실패·재개\n\nsourceplan001→002는기본치유/전투진입stat/Worldinteraction,003은value-expressioncatalog/resultcarrier의역소비경계를보완했다.146파일capture는전체행검토주장이아니다. 정식review001은boundedowner/producer/consumer정역검토다.\nP2G40/50표준검증은첫실행PASS;제품실패/실행없음. 다음은DEFINITIONS의exacttypedobject/creation/poolschema와clone/accounting준비다.\n')
p=HERE/'WORKING_POLICY.ko.md'
p.write_text(p.read_text()+'\n[GEX-00 인수](execution/single_goal/PLAN.md) 완료. U06 exactproduct/146reviewsource와beforearchive추출hash확인. 다음GEX-10 상세화/구현;U07독립감사전U08종속구현없음.\n')
print('U07 GEX00 complete:',len(b['files']),'archived source/canonical paths; actual restoration verified; GEX10 next')
