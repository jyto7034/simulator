import { parseTree, getNodeValue, printParseErrorCode } from 'jsonc-parser';
import { parseDocument, stringify as yamlStringify, isMap, isSeq, isAlias } from 'yaml';
import { parse as kdlParse, format as kdlFormat, getLocation } from '@bgotink/kdl';
import { fromJson, toJson } from '@bgotink/kdl/json';

export const formats = ['jsonc', 'yaml', 'kdl'];
const key = path => JSON.stringify(path);
const scalar = value => value === null || ['string', 'number', 'boolean'].includes(typeof value);

export class SourceError extends Error {
  constructor(message, offset = 0) { super(message); this.offset = offset; }
}

// Source spans use UTF-16 offsets, matching all three parser APIs and editor text.
export function parseSource(format, text) {
  const spans = new Map();
  let value;
  const remember = (path, start, end) => spans.set(key(path), { start, end });
  if (format === 'jsonc') {
    const errors = [];
    const root = parseTree(text, errors, { allowTrailingComma: true });
    if (errors.length || !root) throw new SourceError(printParseErrorCode(errors[0]?.error), errors[0]?.offset);
    function walk(node, path) {
      remember(path, node.offset, node.offset + node.length);
      if (node.type === 'object') {
        const seen = new Set();
        for (const property of node.children ?? []) {
          const [name, child] = property.children;
          if (seen.has(name.value)) throw new SourceError(`Duplicate key: ${name.value}`, name.offset);
          seen.add(name.value);
          walk(child, [...path, name.value]);
        }
      } else if (node.type === 'array') node.children.forEach((child, i) => walk(child, [...path, i]));
    }
    walk(root, []);
    value = getNodeValue(root);
  } else if (format === 'yaml') {
    const doc = parseDocument(text, { version: '1.2', uniqueKeys: true, keepSourceTokens: true });
    if (doc.errors.length || doc.warnings.length) {
      const error = doc.errors[0] ?? doc.warnings[0];
      throw new SourceError(error.message, error.pos?.[0]);
    }
    function walk(node, path) {
      if (!node) throw new SourceError('Use explicit null for an empty value');
      if (isAlias(node) || node.anchor || node.tag) throw new SourceError('Aliases, anchors and tags are excluded', node.range?.[0]);
      if (node.range) remember(path, node.range[0], node.range[1]);
      if (isMap(node)) for (const pair of node.items) {
        if (typeof pair.key?.value !== 'string' || pair.key.value === '<<' || pair.key.tag || pair.key.anchor) {
          throw new SourceError('Only untagged string keys without merge are supported', pair.key?.range?.[0]);
        }
        walk(pair.value, [...path, pair.key.value]);
      }
      else if (isSeq(node)) node.items.forEach((child, i) => walk(child, [...path, i]));
    }
    walk(doc.contents, []);
    value = doc.toJS({ maxAliasCount: 0 });
  } else if (format === 'kdl') {
    let doc;
    try { doc = kdlParse(text, { storeLocations: true }); }
    catch (error) { throw new SourceError(error.message, error.start?.offset ?? error.errors?.[0]?.start?.offset); }
    function check(node) {
      if (node.tag && !['array', 'object'].includes(node.tag.name)) throw new SourceError('Only JSON array/object node tags are supported', getLocation(node.tag)?.start.offset);
      const seen = new Set();
      for (const entry of node.entries) {
        if (entry.value.tag) throw new SourceError('Tagged scalar values are excluded', getLocation(entry.value)?.start.offset);
        if (entry.name && seen.has(entry.getName())) throw new SourceError(`Duplicate key: ${entry.getName()}`, getLocation(entry)?.start.offset);
        if (entry.name) seen.add(entry.getName());
      }
      const children = new Set(seen);
      for (const child of node.children?.nodes ?? []) {
        const name = child.getName();
        if (name !== '-' && children.has(name)) throw new SourceError(`Duplicate key: ${name}`, getLocation(child.name)?.start.offset);
        children.add(name);
        check(child);
      }
    }
    doc.nodes.forEach(check);
    try { value = toJson(doc); }
    catch (error) { throw new SourceError(error.message, getLocation(doc.nodes[0])?.start.offset); }
    function walk(node, data, path) {
      const location = getLocation(node);
      if (location) remember(path, location.start.offset, location.end.offset);
      const args = node.getArgumentEntries();
      if (scalar(data)) {
        const loc = getLocation(args[0]?.value);
        if (loc) remember(path, loc.start.offset, loc.end.offset);
      } else if (Array.isArray(data)) {
        args.forEach((entry, i) => { const loc = getLocation(entry.value); remember([...path, i], loc.start.offset, loc.end.offset); });
        (node.children?.nodes ?? []).forEach((child, i) => walk(child, data[args.length + i], [...path, args.length + i]));
      } else {
        for (const entry of node.getPropertyEntries()) {
          const loc = getLocation(entry.value); remember([...path, entry.getName()], loc.start.offset, loc.end.offset);
        }
        for (const child of node.children?.nodes ?? []) walk(child, data[child.getName()], [...path, child.getName()]);
      }
    }
    walk(doc.nodes[0], value, []);
  } else throw new Error(`Unknown format: ${format}`);
  function finite(data, path = []) {
    if (typeof data === 'number' && (!Number.isFinite(data) || (Number.isInteger(data) && !Number.isSafeInteger(data)))) {
      throw new SourceError('Number cannot be represented exactly by the comparison adapter', spans.get(key(path))?.start);
    }
    if (data && typeof data === 'object') for (const [name, child] of Object.entries(data)) finite(child, [...path, Array.isArray(data) ? Number(name) : name]);
  }
  finite(value);
  // Compare JSON meaning, not library-specific object prototypes.
  value = JSON.parse(JSON.stringify(value));
  return { value, spans, source: text, location(path) {
    const span = spans.get(key(path));
    if (!span) return null;
    const prefix = text.slice(0, span.start);
    return { ...span, line: prefix.split('\n').length, column: span.start - prefix.lastIndexOf('\n') };
  } };
}

export function serialize(format, value) {
  if (format === 'jsonc') return JSON.stringify(value, null, 2) + '\n';
  if (format === 'yaml') return yamlStringify(value, { lineWidth: 0 });
  if (format === 'kdl') return kdlFormat(fromJson(value, { allowEntries: false, indentation: 2 })) + '\n';
  throw new Error(`Unknown format: ${format}`);
}

export function patchScalar(format, text, path, value) {
  const parsed = parseSource(format, text);
  const previous = path.reduce((node, part) => node?.[part], parsed.value);
  if (!scalar(previous) || !scalar(value)) throw new Error('Scalar patch requires an existing scalar');
  const span = parsed.location(path);
  let literal = JSON.stringify(value);
  if (format === 'kdl' && (value === null || typeof value === 'boolean')) literal = `#${literal}`;
  const result = text.slice(0, span.start) + literal + text.slice(span.end);
  const checked = parseSource(format, result);
  if (JSON.stringify(path.reduce((node, part) => node[part], checked.value)) !== JSON.stringify(value)) throw new Error('Scalar patch changed meaning');
  return { text: result, replaced: span, insertedLength: literal.length };
}
