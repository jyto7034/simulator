#!/usr/bin/env python3
import hashlib,json,subprocess,tarfile,tempfile
from pathlib import Path
from initialize import HERE,ROOT
from run import identity,sha
assert (HERE/'P2G_60_RESULT_001.json').is_file()
d=HERE/'execution/single_goal';assert not d.exists();d.mkdir(parents=True)
for x in ['baseline','evidence']:(d/x).mkdir()
for r in json.loads((HERE/'evidence/REVIEW_SOURCE_MANIFEST_003.json').read_text())['files']:assert sha(ROOT/r['path'])==r['sha256']
b=identity()
previous=json.loads((ROOT/'core_enhanced/docs/roadmaps/core_first/execution/u05_closure_20261004_001/PUBLICATION_RECORD.json').read_text())['source_after']
assert b==previous, 'Product baseline drift: classify before implementation'
for path in sorted((ROOT/'core_enhanced/docs/game_rules').glob('*.md')):
 rel=str(path.relative_to(ROOT))
 if not any(row['path']==rel for row in b['files']):b['files'].append({'path':rel,'bytes':path.stat().st_size,'sha256':sha(path)})
for rel in ['core_enhanced/docs/core_runtime_contract.md','core_enhanced/docs/data_loading_contract.md','core_enhanced/docs/skill_target_contract.md','core_enhanced/docs/game_rulebook.md','core_enhanced/docs/README.md','core_enhanced/docs/roadmaps/core_first/ROADMAP.ko.md','game_server/README.md','game_server/docs/unity_bridge_protocol.md']:
 p=ROOT/rel
 if p.is_file() and not any(r['path']==rel for r in b['files']):b['files'].append({'path':rel,'bytes':p.stat().st_size,'sha256':sha(p)})
b['files'].sort(key=lambda r:r['path']);b['sha256']=hashlib.sha256(json.dumps(b['files'],sort_keys=True).encode()).hexdigest()
(d/'baseline/SOURCE_BEFORE.json').write_text(json.dumps(b,indent=2)+'\n')
with tarfile.open(d/'baseline/SOURCE_BEFORE.tar.gz','x:gz') as archive:
 for r in b['files']:archive.add(ROOT/r['path'],arcname=r['path'],recursive=False)
with tempfile.TemporaryDirectory(prefix='u06-before-restore-') as tmp:
 with tarfile.open(d/'baseline/SOURCE_BEFORE.tar.gz') as archive:archive.extractall(tmp,filter='data')
 for r in b['files']:assert sha(Path(tmp)/r['path'])==r['sha256']
(d/'baseline/ARCHIVE.json').write_text(json.dumps({'sha256':sha(d/'baseline/SOURCE_BEFORE.tar.gz'),'source_sha256':b['sha256'],'files':len(b['files']),'actual_extracted_bytes_verified':True},indent=2)+'\n')
(d/'baseline/GIT_STATUS.txt').write_text(subprocess.check_output(['git','status','--short'],cwd=ROOT,text=True))

(d/'PLAN.md').write_text("""# GOAL-CBST-01 실행

적용 GEX2026-10-04-gex-r8/P2G2026-10-04-r8/quality2/usagev2.1.3.
이전 r7/gex-r7/v2.1.2에서의 운영 변경은 ../../WORKFLOW_INTAKE_001.json과 core_first workflow_efficiency_update_20261004_001에 인수했다. 원래 scope/단계/독립성/완료 조건은 유지한다.
Mode goal-run. 원 연속 요청은 ../../P2G_00_INITIAL_001.json IN-001; 승인 core_first 활성 로드맵의 U06.
P2G+GEX구현/자체검증/비참여GPT-6AstraHigh독립감사/finding교정·재감사/선행통과뒤다음owner까지 명시 승인.
중간 단계검토 opt-in 없음. 감사 인계마다 멈추지 않는다는 사용자 지시가 기본 인계 checkpoint보다 우선한다.
Selected exact member GS-CBST-001/GOAL-CBST-01, PHASE-CBST-01.
HA-CBST-001/FPF-CBST-001/PP-CBST-001/PVR-CBST-001 exact chain은 ../../P2G_60_HANDOFF_001.json과P2G40/50/60결과 소유.

## Current execution state

GEX-00 complete. GEX-10 상세 설계/준비가 다음이다. 제품 변경/새 acceptance 실행/독립 감사는 아직 없다.
현재 member에 기존 execution history가 없음을 좁은경로 확인과 exclusive생성으로 확인했다.
이 디렉터리의 PLAN=currentbasis/scope/design/progress, EXPERIMENTS=실제검증, EXPERIMENT_NOTES=선택/실패/재개위치.
독립 감사는 구현자가 완료 target을 고정한 뒤 별도 post_audit에 기록한다.

baseline SOURCE_BEFORE.json/tar.gz, ARCHIVE.json은 기존 dirty actual bytes와 추출hash를 고정한다.
U05product368 identity를 publicationsource와 재대조했고149reviewsource도 unchanged다.
원래 사용자 변경·선행 구현·감사·저장 원본은 reset/stash/일괄정리하지 않는다.
초기 fullbaseline은 U05 CompleteHigh/Core1758/1758/1767,host91/99+heap1,43receipts/publicPC576와 실제public31success5typedrefusal 근거를 재사용한다. U06 성공 근거가 아니다.
운영 문서7개 delta는 중앙 인수에서 proven non-impacting으로 분류했고 선행 verdict를 소급 재발행하지 않았다.

## Implementation and readiness

원본 의미는 Freeze20요소. ../../CODE_REVIEW_001 RQ01–08/TD01–08/V01–20 및 exactPhasePlan6node가 구현경계다.
root 혼자 BASIC→STARTER, TACTICS→BLOCK, TIME, VERIFY를 순차 수행한다.
각 WP는 자기 schema/state/query/clone 전환을 내부에 포함하며 기존 unit/action/session owner 사이 stable seam을 사용한다.
기본 Heal은 순수아군HP와 피해뒤resolve, activeAllySingle은 명시selector, starter는 정상copygrant/equip.
전술은 같은unit·양쪽원자교대·강제Defense1500→Attack·즉시work/exitAttack; 저지는 실제segment접촉0.75와현재위치/basic적대예외.
선택/설정은 ActiveBattleSession의 단일restore owner와manualPause/disconnect/terminal우선; U05강제침식hostclock예외 유지.
첫 제품 변경 전 실제 type/field/queue carrier/cloneextent/targetselection/strictauthoring hooks와 작은검증selector를 이 PLAN에 상세화한다.
source/host/localRON/fixture와 기존 Rust cache/runner가 제공된다. 운영서비스/개인DB/remote배포 mutation은 필요없다.
필수 정책/source gap 없음. exactenum/계수/sweep/DTO선택은 TD01–08내 GEX책임.

## Validation and completion

GEXr8에 따라 strictfixture/공통runner준비→새state크기/clone/queue/rollback 작은실행→focused기능→필수최종통합.
V01–20과10required차원, 정상6세트headless CoreApplication/실제host command-query 인수를 완성한다.
단일/영역치유와6active, TP/live8/출전우선/철수/재배치, 태세/Rest/이동/교대/work/CC/Pause/stale timer,
다방향빠른접촉/현재위치/용량/사거리예외, 일반·배치·스킬focus/settings/disconnect/terminal이 포함된다.
선행141조건+별도31R06의reuse/affected/intentionalchange map과 actualcurrentevidence를 연결한다.
새starter에영향받은legacyfixture는 원래 입력과oracle을 보존하며 새정상grant검증은 별도로 수행한다.
실패/zero/skip/도구오류 raw를 보존하고 필요한 변경뒤 해당검증을 재실행한다. 기존R05peak예외/20분stress제외 외 새예외없음.
최종Core(default/fixtures/observation)/host(default/observation/heap),fmt/check/event-log/clippy/release와publicPC/authoring/native/fault가 필수다.
완료후 exactsource/data/docs/evidence와before-after복원/patch를 고정해 mutation을 멈춘 뒤 비참여GPT-6AstraHigh 독립GEX20.
finding교정·재감사CompleteHigh 전 U07종속구현금지. SinglePhase여서MasterGEX30없음.
사용자stop/승인로드맵완료/현권한환경으로해결불가실제장애에서만중단한다.
""")
(d/'EXPERIMENTS.md').write_text('# U06 실행·검증\n\nP2G40 구조와 P2G50 quality2 표준검증 PASS, 의미검토/P2G60 인계완료.\nGEX00 실제 dirty baseline archive 추출hash확인. 초기U05 baseline은현재성재사용; U06 productmutation/새acceptance실행없음.\n')
(d/'EXPERIMENT_NOTES.md').write_text('# U06 기술 판단과 실패\n\nP2G source plan001/002의발견된실제소비자를003에보완했다. 정식complete review001은149개소스를소유한다.\nP2G40 result조립도중 receipt_ref오추측은미발행상태에서 actual phase_plan_validation_receipt_ref를읽어수정했다. Plan/receipt/검증결과는변경없음. 제품결함아님.\n다음은 BASIC typedheal/allytarget의exactstate/queue/clone/authoring준비와작은검증이다.\n')
p=HERE/'WORKING_POLICY.ko.md';p.write_text(p.read_text()+'\n[GEX-00 인수](execution/single_goal/PLAN.md) 완료. 기존 dirty actual baseline과 추출hash 확인. 다음은 GEX-10 상세설계/구현. U06 독립감사 전 U07 종속구현 없음.\n')
print('U06 GEX00 complete:',len(b['files']),'before files archived and extraction verified;GEX10 next')
