"""Publish S1 self-verification outcome without claiming an independent verdict."""
import json
import run

H = run.HERE
S = H / 'execution/s1'
E = H / 'evidence'
coverage = json.loads((S / 'COVERAGE.json').read_text())
assert coverage['source_sha256'] == run.identity()['sha256']
ledger = S / 'EXPERIMENTS.md'
old = ledger.read_text()
lines = []
for path in sorted(E.glob('*.json')):
    value = json.loads(path.read_text())
    if not isinstance(value, dict) or 'command' not in value or 'exit_code' not in value:
        continue
    name = path.stem
    if f'`{name}`:' not in old:
        lines.append(f"- `{name}`: exit {value['exit_code']}, {value['duration_seconds']:.2f}s, source_unchanged={value['source_unchanged']}; `../../evidence/{name}.json` / `.log`.")
with ledger.open('a') as stream:
    stream.write('\n\nFinal evidence additions (raw failures retained):\n\n' + '\n\n'.join(lines) + '\n')

result = '''# GOAL-CRSO-S1 구현·자체 검증 결과

GEX-10의 CONFIG, CORE, PORTS, S1VERIFY 구현과 자체 검증을 마쳤다. 독립 GEX-20은 아직 수행하지 않았으므로 Goal Complete를 주장하지 않는다. 다음 대상은 `target/s1_001/MANIFEST.json`이며 감사 출력은 Goal root의 `post_audit/s1_20261004_001/`에만 쓴다. S2 구현은 S1 Complete / High 이후에 시작한다.

선택한 runtime safety profile이 content publication 전 parse/compile, immutable generation, authoring/trial, 첫 native operation, restore, history window, 실제 CLI/server로 전달된다. 운영 기본 수치38개는 단일 embedded RON에서 읽으며 기존 값과 일치한다. raw 파일 SHA와 normalized effective SHA를 분리한다. 지정 파일 오류는 path/field를 보고하고 startup/publication을 거절한다. content semantic identity는 profile과 분리한다.

CoreApplication의 public opaque persistence rollback token은 owner/revision/one-shot 권한을 확인하고 monotonic profile peak를 보존한다. 호스트는 private GameCore/BehaviorResult를 사용하지 않고 public ApplicationCommand/Result를 소비한다. terminal event-log attachment는 선행 금지 계약대로 제공하지 않는다. 이력은 선택된 수량/바이트/span을 적용하고 스트리밍 import, dirty public edits, cursor 만료를 검사한다. 작은 history span은 battle 종료 시간을 제한하지 않는다.

최종 Core 기본/test-fixtures 각각1700 tests, 서버76 tests가 실패·ignored 없이 통과했다. check, event_log_checks, Clippy, fmt와 release binaries, 실제 공개 API/CLI/서버 증거는 COVERAGE.json의 exact receipts에 연결한다. Clippy 경고는 보존하며 zero-warning 주장은 하지 않는다. 공개 profile probe003은24 CLI commands 및5 실제 host configurations(환경 변수 선택 포함)를 실행했다. 경계 probe002는 valid consumer 및 private/Clone/Serde/consumed capability 거부13 cases, 공개 API/CLI probe001은21 commands를 검증했다.

U01/R06/R07의88 inherited conditions와 F01 selectors를 최종 두 Core suite에서 재실행했다. U01 public oracle5/5, trace5/5,9 packs45 CLI commands 및 gameplay/amount parity가 통과했다. PC36 raw576rows는31success/5typed-refusal 조건을 유지한다. 바로 전 U01과 비교하면 모든 non-timing fields가 같고 allocation samples도 같다. R06와의 기존 error requested-detail 차이는80건으로 별도 보존한다.

R05 raw 실행은 concurrent peak guard로 exit101이었다. 기존 분류기는 concurrent peak만 경고로 허용하는 원래 정책에 따라 `warning_concurrent_peak_only`로 판정했다. 측정 peak9354169 bytes가 cap9030188보다323981 높다. 다른 operation/retained/event-heavy guard 및 deterministic/repeatability는 통과했다. raw 실패를 성공으로 바꾸거나 cap을 완화하지 않았다. 임의 latency gate와20분 실행은 추가하지 않았다.

초기 server baseline은 U02 전부터 실패했으며 PORTS에서 현행 공개 계약에 맞췄다. 최초 runner membership에 빠졌던 cfg-test/include 입력3개는 실제 불변 bytes로 보충하고 최종 전체 suite를 다시 실행했다. 그 전에 실행한 release/public probes는 해당 test-only 입력을 compile하지 않았고 runtime source/binaries는 동일하다. COVERAGE.json은 이러한 제한적 재사용을 직접 검사한다. 첫 PC isolate compile failure, profile parity probe의 usage accounting 과잉 비교, default-number receipt의 대소문자 파일 충돌 등 모든 실패 원본을 남겼다.

결정 권한은 CONTINUOUS_REQUEST_20261004 및 frozen policy/TD01–04다. 구현 상세는 기존 의미 안에서 구체화했으며 새로운 사용자 정책을 만들지 않았다. exact acceptance7개, V01–08/V12, required dimensions10개는 COVERAGE.json에, 선행 영향은 PREDECESSOR_IMPACT.json에 연결한다. 감사자는 이 mapping을 주장으로 취급하고 실제 source/consumer와 의미를 독립 확인해야 한다.

배포된 Unity/remote 환경 검증 주장은 없다. 실제 service probe는 격리된 로컬 endpoint와 temp persistence/content를 사용했으며 argv/config/content/binary hashes/output은 evidence에 있다. SOURCE_BEFORE 및 supplemental archives를 합쳐 forward/reverse patch fuzz0, exact membership/bytes 복원을 검증하고 source/records/references를 동결한다. 기존 사용자 변경과 과거 감사 원본은 그대로 보존했다.
'''
with (S / 'RESULT.ko.md').open('x') as stream:
    stream.write(result)
with (S / 'GEX_10_RESULT_001.json').open('x') as stream:
    json.dump(dict(goal='GOAL-CRSO-S1', stage='GEX-10', status='implementation_and_self_validation_complete; independent audit pending', source_sha256=coverage['source_sha256'], coverage='COVERAGE.json', result='RESULT.ko.md', target='target/s1_001/MANIFEST.json', next='Fresh independent implementation-nonparticipant GPT-6 Astra / High GEX-20; restore predecessor impact and S1 conditions before S2'), stream, ensure_ascii=False, indent=2)
    stream.write('\n')
with (S / 'PLAN.md').open('a') as stream:
    stream.write('\nFinal GEX-10 current state: CONFIG/CORE/PORTS/S1VERIFY self-verification complete. RESULT.ko.md and COVERAGE.json carry exact current results and limitations. Freeze target/s1_001 then independent GEX-20 at ../../post_audit/s1_20261004_001; no S2 implementation before Complete/High.\n')
with (S / 'EXPERIMENT_NOTES.md').open('a') as stream:
    stream.write('\nFinal outcome and remaining warning: RESULT.ko.md. PC36 exact non-timing parity to immediate U01; R05 raw101 and warning_concurrent_peak_only are separate. History public-edit accounting gap described above is closed by the dirty-bit correction and final tests, not a remaining claim limitation. No independent verdict exists yet.\n')
print(json.dumps(dict(source_sha256=coverage['source_sha256'], new_ledger_entries=len(lines), next='freeze then independent audit')))
