#!/usr/bin/env python3
"""PLAN-037 Phase 3: convert in-widget `use { ... }` blocks to top-level
`use.web` statements.

Rules:
- entries WITH `from "path"` -> `use.web [kind] names from "path"` collected
  at file top-level (before the first component fn/widget decl)
- entries WITHOUT from (cross-file component refs) stay in a rewritten
  in-widget use block at the original position
- comments/blank lines inside blocks are dropped (they describe the old form)

Usage: python migrate_use_web.py file1.at file2.at ...
"""
import re
import sys


def first_decl_index(lines):
    for i, l in enumerate(lines):
        if re.match(r'^(component fn |widget |store )', l):
            return i
    return 0


def convert(path):
    src = open(path, encoding='utf-8').read()
    lines = src.split('\n')
    web_stmts = []
    keep_blocks = []  # (out_index, [entries]) rewritten blocks
    issues = []

    out = []
    in_block = False
    block_out_idx = None
    keep_entries = []

    for line in lines:
        stripped = line.strip()
        if not in_block:
            if re.match(r'^\s{4,8}use \{$', line):
                in_block = True
                block_out_idx = len(out)
                keep_entries = []
                continue
            out.append(line)
            continue
        # inside a block
        if stripped == '}':
            in_block = False
            if keep_entries:
                block = ['    use {']
                block += ['        ' + e for e in keep_entries]
                block.append('    }')
                keep_blocks.append((block_out_idx, block))
            continue
        if not stripped or stripped.startswith('//'):
            continue
        m = re.match(r'^(fn|component|composable):\s*(.+)$', stripped)
        if not m:
            issues.append((path, stripped))
            keep_entries.append(stripped)
            continue
        kind, rest = m.group(1), m.group(2).strip()
        fm = re.search(r'\s+from\s+"([^"]+)"\s*$', rest)
        if fm:
            names = rest[: fm.start()].strip().rstrip(',')
            path_str = fm.group(1)
            mod = {'fn': '', 'component': 'component ', 'composable': 'composable '}[kind]
            web_stmts.append('use.web {}{} from "{}"'.format(mod, names, path_str))
        else:
            keep_entries.append('{}: {}'.format(kind, rest))

    if not web_stmts and not keep_blocks:
        return False

    # insert kept blocks at their recorded positions (indices refer to `out`)
    for idx, block in reversed(keep_blocks):
        out[idx:idx] = block

    # insert use.web statements before the first top-level decl
    if web_stmts:
        idx = first_decl_index(out)
        stmts = ['// PLAN-037 Phase 3: web ecosystem imports (use.web).'] + web_stmts + ['']
        out[idx:idx] = stmts

    open(path, 'w', encoding='utf-8').write('\n'.join(out))
    for p, txt in issues:
        print('ISSUE {} :: {}'.format(p, txt[:80]), file=sys.stderr)
    return True


if __name__ == '__main__':
    for f in sys.argv[1:]:
        changed = convert(f)
        print(('CONVERTED ' if changed else 'unchanged ') + f)
