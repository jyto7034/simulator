"""Bounded static check for adjacent same-argument shared/native debits.

Scan all Core source files, compare sites to the exact U06 preimage, and keep
the raw candidates. This check complements runtime boundary tests; it does
not claim to prove all possible indirect double charging absent.
"""
import argparse
import json
import re
import tarfile
from pathlib import Path
from run import ROOT, HERE, identity, sha


def charges(source):
    pattern = re.compile(r'(?P<prefix>(?:\w+::)*\w+)::charge\s*\(')
    found = []
    for match in pattern.finditer(source):
        depth, quote, escaped, end = 1, None, False, match.end()
        while end < len(source) and depth:
            char = source[end]
            if quote:
                if escaped:
                    escaped = False
                elif char == '\\':
                    escaped = True
                elif char == quote:
                    quote = None
            elif char == '"':
                quote = char
            elif char == '(':
                depth += 1
            elif char == ')':
                depth -= 1
            end += 1
        assert depth == 0
        arguments = re.sub(r'\s+', '', source[match.end():end - 1]).rstrip(',')
        found.append({'prefix': match['prefix'], 'arguments': arguments,
                      'line': source.count('\n', 0, match.start()) + 1,
                      'start': match.start(), 'end': end})
    return found


def candidates(source):
    found = charges(source)
    result = []
    for first, second in zip(found, found[1:]):
        between = source[first['end']:second['start']]
        if (first['arguments'] == second['arguments']
                and first['prefix'] != second['prefix']
                and re.fullmatch(r'[\s?;]*', between)):
            result.append({'arguments': first['arguments'],
                           'prefixes': [first['prefix'], second['prefix']],
                           'lines': [first['line'], second['line']]})
    return result


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('output', type=Path)
    args = parser.parse_args()
    archive = HERE / 'execution/single_goal/baseline/SOURCE_BEFORE.tar.gz'
    with tarfile.open(archive) as stream:
        originals = {m.name: stream.extractfile(m).read().decode()
                     for m in stream.getmembers() if m.isfile()
                     and m.name.startswith('core_enhanced/src/') and m.name.endswith('.rs')}
    rows, added = [], []
    paths = sorted((ROOT / 'core_enhanced/src').rglob('*.rs'))
    for path in paths:
        name = str(path.relative_to(ROOT))
        source = path.read_text()
        current = candidates(source)
        previous = candidates(originals.get(name, ''))
        previous_keys = {(r['arguments'], tuple(r['prefixes'])) for r in previous}
        new = [r for r in current if (r['arguments'], tuple(r['prefixes'])) not in previous_keys]
        if current or previous:
            rows.append({'path': name, 'current': current, 'predecessor': previous})
        added.extend({'path': name, **r} for r in new)
    value = {'source': identity(), 'tool_sha256': sha(Path(__file__)),
             'predecessor_archive_sha256': sha(archive), 'files_scanned': len(paths),
             'scope': 'All core_enhanced/src Rust files; balanced call arguments, distinct prefixes, adjacent statements; comparison to exact U06 source archive.',
             'candidates': rows, 'u07_added_candidates': added, 'passed': not added}
    with args.output.open('x') as stream:
        json.dump(value, stream, indent=2)
        stream.write('\n')
    print(json.dumps({'files_scanned': len(paths), 'u07_added_candidates': added, 'passed': not added}))
    return 0 if not added else 1


if __name__ == '__main__':
    raise SystemExit(main())
