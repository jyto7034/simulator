"""Summarize actual receipts without converting self-verification into an audit verdict."""
from pathlib import Path
import json
import run

H = run.HERE
E = H / 'evidence'
names = ['all_default_002', 'all_feature_001', 'check_001', 'event_log_check_001', 'clippy_001', 'fmt_001', 'release_001', 'public_oracle_after_001', 'public_cli_001', 'public_trace_001', 'pc_current_001', 'pc_compare_001', 'r05_current_002', 'r05_compare_002']
source = run.identity()
receipts = {}
for name in names:
    p = E / (name + '.json')
    data = json.loads(p.read_text())
    assert data['source_before'] == data['source_after'] == source, name
    assert data['log_sha256'] == run.sha(E / (name + '.log')), name
    assert data['exit_code'] in ((0, 101) if name == 'r05_current_002' else (0,)), name
    receipts[name] = data
public = json.loads((E / 'public_after/SUMMARY.json').read_text())
assert public['passed'] == 5 and public['failed'] == 0
assert len(json.loads((E / 'public_after_trace/SUMMARY.json').read_text())['rows']) == 5
cli = json.loads((E / 'cli_current/CLI_FINAL_SUMMARY_002.json').read_text())
assert len(cli['commands']) == 45 and all(c['exit_code'] == 0 for c in cli['commands'])
r05 = json.loads((E / 'R05_SUMMARY_CURRENT_002.json').read_text())
assert not r05['other_hard_guard_failures']
text = (H / 'EXPERIMENTS.md').read_text()
text += '\n## 교정 후 실제 검증\n\n| 원자료 | exit | 시험 통과/실패 | 초 |\n|---|---:|---:|---:|\n'
for name, data in receipts.items():
    text += f"| [{name}](evidence/{name}.json) | {data['exit_code']} | {data['passed']}/{data['failed']} | {data['duration_seconds']:.2f} |\n"
text += f'''\n모든 위 명령의 실행 전후 source는 388개·`{source['digest']}`로 동일하다.
기본/feature 각 1684 PASS는 기존 1677개와 추가 7개를 포함하며 제외/ignored는 없다.
중단된 `all_default_001`은 완료 receipt가 없어 PASS로 사용하지 않았고 별도 interruption 기록을 남겼다.
`focused_002`의 1실패는 초기화 이전에 분기한 Debug HashMap 표시 순서 시험을 정정한 것으로, 원 실패를 보존한다.

공개 F01 입력은 before 2 PASS/3 FAIL에서 after **5 PASS/0 FAIL**로 바뀌었다.
다섯 case 모두 marker 도착·정확 projectile HP 시간·후속 area 허용/차단 oracle을 확인했고 trace on/off event가 같다.
공개 9팩45명령은 모두 exit0이며 팩별 trace on/off gameplay·amount 동일성을 확인했다.
R06 31수용→21 integration과 R07 F01–F12의 지정 selector는 양쪽 전체 실행에서 모두 발견되고 통과했다.
55개 Goal 조건과 새 회귀의 연결은 [COVERAGE_CURRENT](COVERAGE_CURRENT.json)이며 의미 판정은 독립 감사에 남긴다.

PC36은 현재 제품 source의 격리 사본에 기존 probe만 부착하여 다시 실행했다. 사본 일치, 계측 변경 2파일과
보조 probe 원자료는 `pc_isolate_current_001.json` 및 hooks archive에 있다. 기존 비교기의 원 기준을 유지했다.
R05 현재 raw 결과는 `{r05['classification']}`, concurrent peak 초과 `{r05['concurrent_peak_excess_bytes']}` bytes다.
기존 합의의 concurrent peak 단독 경고만 적용하며 allocation count/retained/event-heavy peak/parity/repeatability
기준을 완화하지 않았다. latency gate는 원 계약상 적용하지 않는다. 20분 시험은 수행하지 않았다.
clippy는 기존의 never_loop/eq_op 허용 옵션을 유지했고, GameError 크기 등 경고를 0개라고 주장하지 않는다.

독립 감사는 아직 미수행이다. 이 자체 검증으로 후속 작업의 선행 통과를 선언하지 않는다.
'''
(H / 'EXPERIMENTS.md').write_text(text)
plan = (H / 'PLAN.md').read_text().replace('현재: GEX-00 인수 완료 / GEX-10 baseline·교정 진행. 감사 미수행.', '현재: GEX-10 교정·자체 검증 완료 / fresh GEX-20 인계 준비. 감사 판정은 별도.')
(H / 'PLAN.md').write_text(plan)
(H / 'AUDIT_HANDOFF.md').write_text('''# U00 인수 확인 / U01 F01 fresh GEX-20 인계

2026-10-04 · 구현자 자체 검증 완료, 독립 verdict 미발행.

## 정확 대상과 권한

- Goal `GS-CARX-001 / GOAL-CARX-01`, Handoff `HA-CARX-001`.
- 원 Goal: `../../../../P2G_50_GOAL_SET.ko.md`; 원 계획 인계 `../../../../PLAN_HANDOFF.ko.md`.
- 이전 Not Complete/F01: `../../../../post_audit/20261003_001/COMPLETION_AUDIT.md`.
- 새 target: `target/f01_action_flow_001/MANIFEST.json`. 전후 archive, scoped FORWARD.patch,
  fuzz0 정·역 복원, membership/bytes 검증, records/references 원자료를 포함한다.
- GEX-10 결정과 실험: PLAN.md / EXPERIMENTS.md / EXPERIMENT_NOTES.md / COVERAGE_CURRENT.json.
- 현재 source digest: ''' + source['digest'] + ''' (388개).
- 사용자 연속 실행 위임: `../../../../../../roadmaps/core_first/execution/CONTINUOUS_REQUEST_20261004.ko.md`.
  최신 요청은 독립 감사·실패 교정·재감사·통과 후 다음 작업을 포함한다. 옛 인계의 감사 제외 문구는
  당시 요청 범위이며 이번 명시적 요청을 취소하지 않는다.
- U00: `../../../../../../roadmaps/core_first/execution/u00_20261004_001/INTAKE.ko.md` 및
  `../DECISIONS_20261004.ko.md`(U00 디렉터리 기준). 현재성, 정책 권한, 범위 연결만이며 제품 완료가 아니다.

## 변경과 검토할 경계

기존 387개 중 제품 2파일 변경(boss_runtime.rs, queue.rs), 회귀 파일 1개 추가다.
정확 actor+opaque ActionHandle의 pending/active cast와 미실행 SkillStep/FlowWake만 회수한다.
준비 중 취소의 소유 lock만 풀고 이동/basic/다른 action을 actor-wide로 취소하지 않는다.
이미 방출된 projectile/area의 source/resource lease/완료 event는 보존한다. terminal과 queue 변경의
오류는 Result/poll로 전달하며 후보 transaction이 복원한다. Detach/정상 Released는 취소 경로에 들어가지 않는다.

필수 직접 영향은 A08-03/A08-04/A10-04/A10-06 및 X10이다. 나머지 55조건과 R06/R07 current 영향은
기존 condition reasoning과 새 양쪽 suite/공개 CLI/PC36/R05 증거를 연결해 판정한다.
기존 snapshot/live/source-copy/resource provenance와 이후 state marker를 포함한다.
추가 7개 시험에는 legacy delayed step, stale wake/추출 step, same-timestamp, exact capability 격리,
released persistent area, 취소 중 fault 6지점과 retry가 있다. 격리 시험의 synthetic cast는 내부 경계용이며
공개 제작 지원 증거로 대신하지 않는다. 5개 공개 팩은 기존 감사 입력 bytes 그대로 실행했다.

## 독립 감사 요청

구현 비참여 GPT-6 Astra / High reviewer가 `goal_execution_pipeline(1).md`의 GEX-20을 수행한다.
현재 target과 실제 경로를 읽고 모든 조건의 적용성·증거·currentness를 확인한다. source-level 의미 검토를
테스트 수로 대체하지 않는다. 유효한 명령 증거는 검증 후 재사용하고 부족한 안전 probe만 새 출력 경로에서 수행한다.
U00 인수 문서도 최신 범위/정책 위임/과거 증거 적용성을 점검한다.

출력은 새 `../../../../post_audit/20261004_f01_001/` 아래에 작성한다. target·구현·과거 감사는 수정하지 않는다.
COMPLETION_AUDIT.md에 정확 target, condition matrix/currentness, Complete / High 또는 Not Complete / Blocked,
finding의 교정 owner와 영향 범위를 명시한다. 제품 수정 필요시 finding을 반환하고 이번 read-only 감사를 닫는다.
후속 U02 구현은 해당 fresh 감사 통과 뒤 시작한다.
''')
print(json.dumps({'source_digest':source['digest'],'commands':len(names),'public_oracles':5,'public_cli_commands':45,'independent_audit':False}))
