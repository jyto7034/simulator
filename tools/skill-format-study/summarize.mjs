import { readFileSync, writeFileSync, readdirSync } from 'node:fs';
import { resolve } from 'node:path';
import { createHash } from 'node:crypto';
import { spawnSync } from 'node:child_process';
import { performance } from 'node:perf_hooks';
import { formats, parseSource, patchScalar } from './adapters.mjs';
import { root, inputs } from './prepare.mjs';
import { validate } from './submit.mjs';

const evidence = resolve(root, 'core_enhanced/evidence/skill_authoring_roadmap_20261023');
const baseline = JSON.parse(readFileSync(resolve(evidence, 'format-study-frozen-inputs.json'), 'utf8'));
const unchanged = Object.entries(baseline).every(([name, hash]) => createHash('sha256').update(readFileSync(resolve(root, name))).digest('hex') === hash);
if (!unchanged) throw new Error('Frozen study inputs or grader changed after trials started');
const report = { frozenInputsUnchanged: unchanged, usage: { tokens: null, billedCost: null, reason: 'Agent API does not expose measured token/cost usage here' }, timingDefinition: 'Elapsed wall clock from attempt directory preparation before dispatch to submission; includes dispatch and tools. No time limit.', tools: [], authors: [] };
for (const format of formats) {
  const text = readFileSync(resolve(inputs, `module.${format}`), 'utf8');
  const aux = readFileSync(resolve(inputs, `representation.${format}`), 'utf8');
  const path = ['values', 1, 'value', 'Amount', 'root', 'Ratio', 'numerator', 'Mul', 'scalar', 'Const', 'value'];
  const timings = [];
  for (let i = 0; i < 20; i++) { const start = performance.now(); parseSource(format, text); patchScalar(format, text, path, 5); timings.push(performance.now() - start); }
  timings.sort((a, b) => a - b);
  const patch = patchScalar(format, text, path, 5);
  report.tools.push({ format, moduleBytes: Buffer.byteLength(text), representationBytes: Buffer.byteLength(aux), guideBytes: Buffer.byteLength(readFileSync(resolve(inputs, `task.${format}.txt`))), parsedAndPatchedMedianMs: timings[10], parsedAndPatchedP95Ms: timings[18], scalarUnrelatedBytesChanged: text.slice(0, patch.replaced.start) === patch.text.slice(0, patch.replaced.start) && text.slice(patch.replaced.end) === patch.text.slice(patch.replaced.start + patch.insertedLength) ? 0 : null });
}
for (const name of readdirSync(resolve(evidence, 'authors')).sort()) {
  const directory = resolve(evidence, 'authors', name);
  const meta = JSON.parse(readFileSync(resolve(directory, 'attempt.json'), 'utf8'));
  const attempts = readdirSync(resolve(directory, 'submissions')).sort().map(index => {
    const path = resolve(directory, 'submissions', index);
    const recorded = JSON.parse(readFileSync(resolve(path, 'report.json'), 'utf8'));
    const rechecked = validate(meta.format, readFileSync(resolve(path, `module.${meta.format}`), 'utf8'), readFileSync(resolve(path, `representation.${meta.format}`), 'utf8'));
    if (rechecked.pass !== recorded.pass) throw new Error(`Non-reproducible trial: ${name}/${index}`);
    return { submission: recorded.submission, pass: recorded.pass, elapsedMs: recorded.elapsedMs, failures: recorded.checks.filter(check => !check.pass) };
  });
  for (const stem of ['module', 'representation']) {
    const diff = spawnSync('diff', ['-u', resolve(inputs, `${stem}.${meta.format}`), resolve(directory, `${stem}.${meta.format}`)], { encoding: 'utf8' });
    writeFileSync(resolve(directory, `${stem}.diff`), diff.stdout);
  }
  report.authors.push({ ...meta, attempts });
}
writeFileSync(resolve(evidence, 'format-study-results.json'), JSON.stringify(report, null, 2) + '\n');
console.log(JSON.stringify({ tools: report.tools, authors: report.authors.map(a => ({ model: a.model, format: a.format, repetition: a.repetition, submissions: a.attempts.length, pass: a.attempts.at(-1)?.pass, elapsedMs: a.attempts.at(-1)?.elapsedMs })) }, null, 2));
