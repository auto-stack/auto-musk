#!/usr/bin/env python3
"""PLAN-037 P1-1: convert in-widget `use { component: A, B }` blocks (no-from
cross-file refs) to top-level `use snake_alias: Name` statements.

After Plan 425 every child is a widget, so the native top-level alias form
applies; this zeroes the deprecated in-widget use-block syntax in musk.

Usage: python migrate_use_alias.py file1.at ...
"""
import re
import sys


def snake(name: str) -> str:
    out = []
    for i, c in enumerate(name):
        if c.isupper() and i > 0:
            out.append('_')
        out.append(c.lower())
    return ''.join(out)


def first_decl_index(lines):
    for i, l in enumerate(lines):
        if re.match(r'^(widget |store |fn |use\.web |use \w+: )', l):
            # insert BEFORE the first widget/store decl but AFTER existing
            # top-level use statements — find first widget/store/fn instead
            if not l.startswith('use'):
                return i
    return 0


def convert(path):
    lines = open(path, encoding='utf-8').read().split('\n')
    out = []
    aliases = []
    in_block = False
    block_indent = 0
    block_start_idx = None
    changed = False
    for line in lines:
        m = re.match(r'^(\s+)use \{$', line)
        if not in_block and m:
            in_block = True
            block_indent = len(m.group(1))
            block_start_idx = len(out)
            continue
        if in_block:
            if line.strip() == '}':
                in_block = False
                continue
            if not line.strip() or line.strip().startswith('//'):
                continue
            em = re.match(r'^component:\s*(.+)$', line.strip())
            assert em, 'unexpected block entry in {}: {}'.format(path, line.strip())
            names = [n.strip() for n in em.group(1).split(',') if n.strip()]
            for n in names:
                aliases.append('use {}: {}'.format(snake(n), n))
            changed = True
            continue
        out.append(line)
    if not changed:
        return False
    # collision check with existing top-level use aliases
    existing = {l.split(':')[0].strip() for l in out if re.match(r'^use \w+: ', l)}
    for a in aliases:
        assert a.split(':')[0].strip() not in existing, 'alias collision: ' + a
    idx = first_decl_index(out)
    out[idx:idx] = ['// PLAN-037 P1: cross-file component refs (native alias form).'] + aliases + ['']
    open(path, 'w', encoding='utf-8').write('\n'.join(out))
    return True


if __name__ == '__main__':
    for f in sys.argv[1:]:
        print(('CONVERTED ' if convert(f) else 'unchanged ') + f)
