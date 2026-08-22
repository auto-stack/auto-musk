#!/usr/bin/env python3
"""PLAN-037 Phase 4: convert `component fn X(params) { blocks... body }` to
`widget X(params) { blocks... view { body } }`.

Grammar invariant (component fn): params -> use/computed/msg/model/on/watch/
style blocks first, then the view body as bare elements. The transformer wraps
the trailing element region in `view { }` and renames the decl.

Usage: python migrate_widget.py file1.at ...
"""
import re
import sys

BLOCK_KW = re.compile(r'^(use|computed|msg|model|on|watch|style|bind|routes)\b')


def convert(path):
    lines = open(path, encoding='utf-8').read().split('\n')
    opener = re.compile(r'^component fn (\w+)\((.*)\) \{$')
    out = []
    i = 0
    converted = False
    while i < len(lines):
        m = opener.match(lines[i])
        if not m:
            out.append(lines[i])
            i += 1
            continue
        out.append('widget {}({}) {{'.format(m.group(1), m.group(2)))
        i += 1
        converted = True

        # walk blocks; find where the element body starts
        body_start = None
        depth = 0  # brace depth inside the decl (opener = 1)
        j = i
        block_depth = 0
        in_block = False
        while j < len(lines):
            line = lines[j]
            if in_block:
                block_depth += line.count('{') - line.count('}')
                if block_depth == 0:
                    in_block = False
                out.append(line)
                j += 1
                continue
            if line == '}' or line == '    }' and False:
                pass
            if line == '}':  # decl close at top level
                break
            if line.startswith('    ') and not line.startswith('     ') and BLOCK_KW.match(line.strip()):
                in_block = True
                block_depth = line.count('{') - line.count('}')
                if block_depth == 0 and line.rstrip().endswith('{'):
                    block_depth = 1
                out.append(line)
                j += 1
                continue
            # indent-4 non-block content or comment/blank: potential body start
            if body_start is None and line.strip() and not line.strip().startswith('//'):
                body_start = j
                break
            if body_start is None and line.strip().startswith('//'):
                # comment(s) directly before elements — skip consecutive
                # comment/blank lines, then decide on the first content line
                k = j + 1
                while k < len(lines) and (not lines[k].strip() or lines[k].strip().startswith('//')):
                    k += 1
                nxt = lines[k] if k < len(lines) else '}'
                if nxt.startswith('    ') and nxt.strip() and not BLOCK_KW.match(nxt.strip()) and nxt != '}':
                    body_start = j
                    break
            out.append(line)
            j += 1
        if body_start is None:
            # no element body (empty view?) — close as-is
            while j < len(lines):
                out.append(lines[j])
                j += 1
            i = j
            continue
        # body region: body_start .. decl close (the top-level '}' line)
        close = j
        while close < len(lines) and lines[close] != '}':
            close += 1
        body = lines[body_start:close]
        out.append('    view {')
        for b in body:
            out.append(('    ' + b) if b.strip() else b)
        out.append('    }')
        i = close  # continue from decl close; outer loop appends it

    if converted:
        open(path, 'w', encoding='utf-8').write('\n'.join(out))
    return converted


if __name__ == '__main__':
    for f in sys.argv[1:]:
        print(('CONVERTED ' if convert(f) else 'unchanged ') + f)
