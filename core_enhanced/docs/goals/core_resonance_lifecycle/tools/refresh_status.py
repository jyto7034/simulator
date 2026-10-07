"""Keep mutable summaries current; preserve every previous summary verbatim."""
import hashlib, json, sys
from initialize import ROOT, HERE

mode = sys.argv[1]
assert mode in ('verification', 'self_verified')
execution = HERE / 'execution/single_goal'
destination = execution / 'status_history' / mode
destination.mkdir(parents=True, exist_ok=False)
paths = [HERE / 'WORKING_POLICY.ko.md', execution / 'PLAN.md', execution / 'EXPERIMENT_NOTES.md']
start, end = '<!-- U05 CURRENT STATUS BEGIN -->', '<!-- U05 CURRENT STATUS END -->'
if mode == 'verification':
    status = ('P2G-00–60과 GEX-00 인수를 마쳤고, 현재 GEX-10 통합 자체 검증 중이다. '
        'Core 기본1758/fixture1758/관측1767, 호스트 기본91, 공개 기능·재시작·되감기·다섯 fault 모드가 현재 소스에서 통과했다. '
        '호스트 관측·최종 정적/heap 검사와 성능 측정, 최종 증거 연결·정역 복원·독립 감사는 아직 남아 있다. '
        'U05 독립 판정은 없으며 Complete/High 전 U06 종속 구현을 시작하지 않는다.')
else:
    result = json.loads((execution / 'GEX_10_RESULT.json').read_text())
    assert result['self_awarded_independent_verdict'] is False
    status = ('P2G-00–60과 GEX-00 인수를 마쳤고, GEX-10 구현·자체 검증을 완료했다. '
        'GEX_10_RESULT.json, COVERAGE.json, PREDECESSOR_IMPACT.json이 최종 소스·실제 실행·14요소/10차원/127선행조건/별도31 R06을 연결한다. '
        '과거 실패와 수치 경고는 원자료·분류를 보존했다. 다음은 target/u05_001의 실제 bytes·정역 복원을 고정하고 '
        '구현 비참여 GPT-6 Astra/High의 GEX-20 독립 감사를 수행하는 것이다. '
        'U05 독립 완료 판정은 아직 없으며 Complete/High 전 U06 종속 구현을 시작하지 않는다.')
block = start + '\n\n## 현재 상태\n\n' + status + '\n\n' + end
rows = []
sha = lambda value: hashlib.sha256(value).hexdigest()
for path in paths:
    before = path.read_bytes()
    (destination / path.name).write_bytes(before)
    text = before.decode()
    if start in text:
        a, b = text.index(start), text.index(end) + len(end)
        text = text[:a] + block + text[b:]
    else:
        title, rest = text.split('\n', 1)
        text = (title + '\n\n' + block + '\n\n## 시점별 계획·판단 기록\n\n'
            '아래의 준비·미완료·다음 단계 표현은 해당 기록 시점의 상태다. 최신 실행 상태는 위 요약과 최종 실행 증거가 소유한다. '
            '정책·Goal의 동결 원본은 각 P2G artifact를 참조한다.\n' + rest)
        text = text.replace('## Current execution state', '### GEX-00 intake checkpoint')
        text = text.replace('### Current implementation checkpoint', '### Early implementation checkpoint')
    after = text.encode()
    path.write_bytes(after)
    rows.append({'path': str(path.relative_to(ROOT)), 'before_sha256': sha(before),
        'after_sha256': sha(after), 'before_archive': str((destination / path.name).relative_to(ROOT))})
(destination / 'RECORD.json').write_text(json.dumps({'mode': mode, 'files': rows,
    'frozen_policy_or_goal_changed': False}, ensure_ascii=False, indent=2) + '\n')
print(json.dumps({'mode': mode, 'summaries': len(rows)}))
