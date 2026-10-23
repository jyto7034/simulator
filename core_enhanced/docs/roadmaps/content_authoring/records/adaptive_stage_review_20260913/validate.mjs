// Read-only document checks. Run from any cwd; emits JSON, writes no files.
import { readFileSync, readdirSync, existsSync } from 'node:fs';
import { createHash } from 'node:crypto';
import { dirname, resolve, relative, join } from 'node:path';
import { fileURLToPath } from 'node:url';

const record = dirname(fileURLToPath(import.meta.url));
const repo = resolve(record, '../../../../../..');
const before = join(record, 'before');
const read = file => readFileSync(file, 'utf8');
const sha = file => createHash('sha256').update(readFileSync(file)).digest('hex');
const walk = dir => readdirSync(dir, { withFileTypes: true }).flatMap(entry =>
  entry.isDirectory() ? walk(join(dir, entry.name)) : [join(dir, entry.name)]);
const checks = [];
const check = (name, passed) => checks.push({ name, passed: Boolean(passed) });
const links = text => new Set([...text.matchAll(/\[[^\]\n]+\]\(([^)\n]+)\)/g)].map(match => match[1]));
const slug = text => text.toLowerCase().replace(/[^\p{L}\p{N}_\-\s]/gu, '').replace(/\s/g, '-');
const manifests = walk(before).sort().map(original => {
  const path = relative(before, original);
  const current = join(repo, path);
  const oldText = read(original);
  const newText = read(current);
  check(`changed: ${path}`, oldText !== newText);
  check(`no trailing whitespace: ${path}`, !/^.*[\t ]+$/m.test(newText));
  const oldLinks = links(oldText);
  for (const target of links(newText)) {
    if (oldLinks.has(target) || /^[a-z]+:/i.test(target)) continue;
    const [file, fragment] = decodeURIComponent(target).split('#');
    const dest = file ? resolve(dirname(current), file) : current;
    const present = existsSync(dest);
    check(`new link: ${path} -> ${target}`, present);
    if (present && fragment) {
      const headings = [...read(dest).matchAll(/^#{1,6}\s+(.+)$/gm)].map(match => slug(match[1]));
      check(`new anchor: ${path} -> ${target}`, headings.includes(fragment));
    }
  }
  return { path, before_path: relative(repo, original), before_sha256: sha(original), after_sha256: sha(current) };
});
check('eight edited documents have original snapshots', manifests.length === 8);
const docs = join(repo, 'core_enhanced/docs');
const roadmap = join(docs, 'roadmaps/content_authoring');
check('P2G main and distribution copy identical',
  sha(join(docs, 'policy_to_goal_pipeline_revised_v2.md')) === sha(join(roadmap, 'guides/policy_to_goal_pipeline_revised_v2.md')));
const statusPath = 'core_enhanced/docs/roadmaps/content_authoring/ROADMAP_STATUS.md';
const history = text => text.slice(text.indexOf('## 6. 최근 기록'));
check('existing roadmap history unchanged', history(read(join(repo, statusPath))) === history(read(join(before, statusPath))));
const retired = join(record, '../optional_stage_review_20260913/retired/AUTORUN_REQUEST.ko.md.bak');
check('obsolete active .bak removed', !existsSync(join(roadmap, 'AUTORUN_REQUEST.ko.md.bak')));
check('retired original remains recoverable', existsSync(retired) && read(retired).includes('R00–R24 범위 한정 연속 실행 요청'));
const gex = read(join(docs, 'goal_execution_pipeline(1).md'));
for (const marker of ['2026-09-13-gex-r7', '### 기본 구현 인계와 선택적 단계검토',
  '#### 명시적으로 요청한 단계검토', '#### 구현 전후 고정', '모델 변경·자동 재개·시간 경과·검토 PASS는 진행 승인이 아니다.',
  '부분 구현·검증 종료 / 단계검토 요청 대기', '해시는 내용 복원 수단이 아니다.',
  '새 감사 요청 전에는 GEX-20/30을 시작하지 않는다.']) {
  check(`GEX contract marker: ${marker}`, gex.includes(marker));
}
check('removed mandatory preplanned checkpoint scope', !gex.includes('구현 전에 PLAN에 각 checkpoint의 포함 작업'));
check('removed mandatory implementation checkpoint plan', !gex.includes('실제 의존성에 맞는 checkpoint 계획을 연결한다.'));
check('runtime selection criterion exists', gex.includes('구현 중 다음 조건을 처음 충족하는 의미 있는 부분 변경'));
check('optional candidates do not block implementation', gex.includes('후보가 없다는 이유로 구현을 막거나 새 계획 승인을 요구하지 않는다.'));
check('baseline before segment preserved', gex.includes('미래 중단 지점을 모르더라도 구간 시작 전 기준선은 생략하지 않는다.'));
check('partial review does not require future functionality', gex.includes('미래 단계의 검증까지 요구하지 않는다.'));
const p2g = read(join(docs, 'policy_to_goal_pipeline_revised_v2.md'));
check('P2G optional candidates', p2g.includes('중간 지점 후보와 검증 방법을 제안할 수 있다.'));
check('P2G handoff not blocked by missing candidates', p2g.includes('exit·계획 인계·구현 진입을 막거나 상세 계획을 다시 작성하지 않는다.'));
const result = {
  kind: 'document-change-checks-not-goal-audit', checked_at: new Date().toISOString(),
  passed: checks.every(item => item.passed), checks, documents: manifests,
  retired: { original_path: relative(repo, join(roadmap, 'AUTORUN_REQUEST.ko.md.bak')),
    preserved_path: relative(repo, retired), sha256: sha(retired) },
  limitations: ['Checks changed documents only, not all repository links.',
    'Marker presence does not prove natural-language semantics or runner/model behavior.',
    'No game compilation, runtime test, Goal activation, or GEX audit performed.']
};
process.stdout.write(`${JSON.stringify(result, null, 2)}\n`);
process.exitCode = result.passed ? 0 : 1;
