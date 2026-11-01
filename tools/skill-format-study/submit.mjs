import { readFileSync, writeFileSync, readdirSync, mkdirSync, copyFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { spawnSync } from 'node:child_process';
import { isDeepStrictEqual } from 'node:util';
import { fileURLToPath } from 'node:url';
import { parseSource } from './adapters.mjs';
import { root, inputs, expectedModule, expectedRepresentation } from './prepare.mjs';

function differences(actual, expected, path = [], result = []) {
  if (isDeepStrictEqual(actual, expected)) return result;
  if (actual && expected && typeof actual === 'object' && typeof expected === 'object' && Array.isArray(actual) === Array.isArray(expected)) {
    for (const part of new Set([...Object.keys(actual), ...Object.keys(expected)])) differences(actual[part], expected[part], [...path, Array.isArray(expected) ? Number(part) : part], result);
  } else result.push({ path, reason: 'Value differs from requested result' });
  return result;
}
export function validate(format, moduleText, representationText) {
  const checks = [];
  let artifact = '';
  for (const [name, source, expected] of [['module', moduleText, expectedModule()], ['representation', representationText, expectedRepresentation()]]) {
    let parsed;
    try { parsed = parseSource(format, source); }
    catch (error) { checks.push({ file: name, check: 'parse', pass: false, reason: error.message, offset: error.offset }); continue; }
    checks.push({ file: name, check: 'parse', pass: true });
    const diff = differences(parsed.value, expected);
    checks.push({ file: name, check: 'requestedMeaningAndUnrelatedValues', pass: !diff.length, differences: diff.slice(0, 20).map(d => ({ ...d, location: parsed.location(d.path) })) });
    for (const marker of ['header', 'interior']) {
      const original = readFileSync(resolve(inputs, `${name}.${format}`), 'utf8').split('\n').find(line => line.includes(`KEEP:${name}:${marker}`)).trim();
      checks.push({ file: name, check: `comment:${marker}`, pass: source.split('\n').some(line => line.trim() === original) });
    }
    if (name === 'module') {
      const compiled = spawnSync(resolve(root, 'target/debug/compile_skill_composition'), { input: JSON.stringify(parsed.value), encoding: 'utf8', maxBuffer: 16 * 1024 * 1024 });
      checks.push({ file: name, check: 'coreCompileAndNativeValidation', pass: compiled.status === 0 && compiled.stdout.length > 0, status: compiled.status, diagnostic: compiled.stderr, environmentError: compiled.error?.message });
      artifact = compiled.stdout;
    }
  }
  return { pass: checks.every(c => c.pass), checks, artifact };
}
export function submit(directory) {
  const meta = JSON.parse(readFileSync(resolve(directory, 'attempt.json'), 'utf8'));
  const submissions = resolve(directory, 'submissions');
  mkdirSync(submissions, { recursive: true });
  const count = readdirSync(submissions).length;
  if (count >= 3) throw new Error('Submission limit reached (initial + two corrections)');
  const save = resolve(submissions, String(count + 1));
  mkdirSync(save);
  for (const name of ['module', 'representation']) copyFileSync(resolve(directory, `${name}.${meta.format}`), resolve(save, `${name}.${meta.format}`));
  const { artifact, ...report } = validate(meta.format, readFileSync(resolve(save, `module.${meta.format}`), 'utf8'), readFileSync(resolve(save, `representation.${meta.format}`), 'utf8'));
  report.submittedAt = new Date().toISOString();
  report.elapsedMs = Date.parse(report.submittedAt) - Date.parse(meta.startedAt);
  report.submission = count + 1;
  writeFileSync(resolve(save, 'report.json'), JSON.stringify(report, null, 2) + '\n');
  if (artifact) writeFileSync(resolve(save, 'compiled.ron'), artifact);
  return report;
}
if (process.argv[1] === fileURLToPath(import.meta.url)) {
  try { const report = submit(resolve(process.argv[2])); console.log(JSON.stringify(report, null, 2)); process.exitCode = report.pass ? 0 : 1; }
  catch (error) { console.error(error.message); process.exitCode = 2; }
}
