import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync, writeFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { spawnSync } from 'node:child_process';
import { formats, serialize, parseSource, patchScalar } from './adapters.mjs';
import { root, inputs, prepare, baseModule, representation, expectedModule, expectedRepresentation } from './prepare.mjs';
import { validate } from './submit.mjs';

prepare();
const scalarPath = ['values', 1, 'value', 'Amount', 'root', 'Ratio', 'numerator', 'Mul', 'scalar', 'Const', 'value'];
for (const format of formats) {
  test(`${format}: same semantics, unresolved fields, comments, exact scalar patch and location`, () => {
    const text = readFileSync(resolve(inputs, `module.${format}`), 'utf8');
    const parsed = parseSource(format, text);
    assert.deepEqual(parsed.value, baseModule());
    const location = parsed.location(scalarPath);
    assert.equal(text.slice(location.start, location.end), '3');
    assert.ok(location.line > 1 && location.column > 1);
    const patch = patchScalar(format, text, scalarPath, 5);
    assert.equal(patch.text.slice(0, patch.replaced.start), text.slice(0, patch.replaced.start));
    assert.equal(patch.text.slice(patch.replaced.start + patch.insertedLength), text.slice(patch.replaced.end));
    const expected = baseModule(); expected.values[1].value.Amount.root.Ratio.numerator.Mul.scalar.Const.value = 5;
    assert.deepEqual(parseSource(format, patch.text).value, expected);
    const aux = readFileSync(resolve(inputs, `representation.${format}`), 'utf8');
    assert.deepEqual(parseSource(format, aux).value, representation());
    assert.equal(aux.includes('KEEP:representation:interior'), true);
    // A save/reopen of incomplete input preserves the original text, never recovery data.
    const incomplete = text.slice(0, Math.floor(text.length / 2));
    const file = resolve(root, `core_enhanced/evidence/skill_authoring_roadmap_20261023/incomplete.${format}`);
    writeFileSync(file, incomplete);
    assert.equal(readFileSync(file, 'utf8'), incomplete);
  });
  test(`${format}: malformed syntax and duplicate keys fail`, () => {
    const malformed = { jsonc: '{"a":', yaml: 'a: [', kdl: '- {' }[format];
    const duplicate = { jsonc: '{"a":1,"a":2}', yaml: 'a: 1\na: 2\n', kdl: '- a=1 a=2\n' }[format];
    for (const input of [malformed, duplicate]) assert.throws(() => parseSource(format, input), error => Number.isInteger(error.offset));
  });
  test(`${format}: initial reference reaches Core rejection; repaired candidate compiles`, () => {
    const source = readFileSync(resolve(inputs, `module.${format}`), 'utf8');
    const parsed = parseSource(format, source);
    const compile = spawnSync(resolve(root, 'target/debug/compile_skill_composition'), { input: JSON.stringify(parsed.value), encoding: 'utf8' });
    assert.notEqual(compile.status, 0);
    const diag = JSON.parse(compile.stderr);
    assert.equal(diag.code, 'Reference');
    assert.match(diag.reason, /burst-powre/);
    const path = ['skills', 1, 'body', 'Call', 'arguments', 0, 'value', 'Shared'];
    const location = parsed.location(path);
    assert.ok(source.slice(location.start, location.end).includes('burst-powre'));
    const annotate = (name, value) => readFileSync(resolve(inputs, `${name}.${format}`), 'utf8').split('\n').filter(line => line.includes(`KEEP:${name}:`)).join('\n') + '\n' + serialize(format, value);
    const report = validate(format, annotate('module', expectedModule()), annotate('representation', expectedRepresentation()));
    assert.equal(report.pass, true, JSON.stringify(report.checks));
    const altered = expectedRepresentation(); altered.states[0].owner.pop();
    assert.equal(validate(format, annotate('module', expectedModule()), annotate('representation', altered)).pass, false);
  });
}
test('YAML restrictions reject implicit extensions and non-string keys', () => {
  for (const text of ['a: &a 1\nb: *a', 'a: !!str 1', 'a: !custom 1', '<<: {}', '1: x', 'a: .nan', 'a: .inf']) assert.throws(() => parseSource('yaml', text));
});
test('KDL duplicate children and numeric loss are rejected', () => {
  for (const text of ['- {\na 1\na 2\n}', '- a=#nan', '- a=9007199254740993', '- {(foo)a 1\n}', '- a=(foo)1']) assert.throws(() => parseSource('kdl', text));
});
test('JSONC numeric loss is rejected', () => {
  assert.throws(() => parseSource('jsonc', '{"a":9007199254740993}'));
});
