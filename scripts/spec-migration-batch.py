"""Explicit multi-source wrapper for the Rust spec-migrate engine (PLAN-107).

The registry is a user-selected allowlist, not directory discovery. Origins and
aliases are read-only; snapshots and recovery journals belong outside Git.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import subprocess
import tempfile


def sha(data):
    return hashlib.sha256(data).hexdigest()


def load(path):
    return json.loads(Path(path).read_text(encoding='utf-8-sig'))


def atomic_json(path, value):
    path = Path(path)
    path.parent.mkdir(parents=True, exist_ok=True)
    with tempfile.NamedTemporaryFile(dir=path.parent, delete=False) as f:
        temp = Path(f.name)
        f.write((json.dumps(value, ensure_ascii=False, indent=2) + '\n').encode('utf-8'))
        f.flush()
        os.fsync(f.fileno())
    try:
        os.replace(temp, path)
    finally:
        temp.unlink(missing_ok=True)


def records(document):
    sections = document['sections']
    if isinstance(sections, list):
        return [item for section in sections for item in section['items']]
    return [item for value in sections.values() for item in (value if isinstance(value, list) else value['items'])]


def json_fences(content):
    # Fence lengths depend on actual legacy content, not a hard-coded triple.
    import re
    for match in re.finditer(r'(?m)^(`{3,})json\s*\n', content):
        end = re.search(r'(?m)^' + re.escape(match[1]) + r'\s*$', content[match.end():])
        if end:
            try:
                yield json.loads(content[match.end():match.end() + end.start()])
            except ValueError:
                pass


def check_sources(registry, projects=None):
    for row in registry['sources']:
        if projects and row['project'] not in projects:
            continue
        for field in ['origin', 'snapshot']:
            path = Path(row[field])
            if sha(path.read_bytes()) != row['sha256']:
                raise RuntimeError(f"Source drift: {row['key']} {field}; freeze a new revision before continuing")
    by_key = {row['key']: row for row in registry['sources']}
    for alias in registry['aliases']:
        if projects and alias['project'] not in projects:
            continue
        for field in ['origin', 'snapshot']:
            if sha(Path(alias[field]).read_bytes()) != alias['sha256']:
                raise RuntimeError('Alias source drift: ' + alias['origin'])
        if not alias['equivalent'] or load(alias['snapshot']) != load(by_key[alias['canonical_source']]['snapshot']):
            raise RuntimeError('Alias differs; reconcile unique records before applying: ' + alias['origin'])


def physical(path):
    value = str(Path(path).resolve())
    # Rust canonicalize uses Windows extended-length paths. They identify the
    # same directory as the ordinary absolute path in the explicit registry.
    prefix = chr(92) * 2 + '?' + chr(92)
    if value.startswith(prefix):
        value = value[len(prefix):]
    return os.path.normcase(value)


def checked_manifest(row, journal):
    raw = journal.read_bytes()
    if journal.with_suffix('.manifest-sha256').read_text() != sha(raw):
        raise RuntimeError('Manifest checksum changed: ' + str(journal))
    m = json.loads(raw)
    if physical(m['root']) != physical(row['worktree']) or physical(m['source']) != physical(row['snapshot']):
        raise RuntimeError('Manifest root/source binding mismatch')
    if m['source_hash'] != row['sha256'] or m['source_key'] != row['key']:
        raise RuntimeError('Manifest source identity mismatch')
    items = [e['original_record'] for e in m['entries'] if e['source'] != 'envelope']
    original = load(row['snapshot'])
    if items != records(original) or len(items) != row['records']:
        raise RuntimeError('Record conservation failed: ' + row['key'])
    envelope = json.loads(json.dumps(original))
    sections = envelope['sections']
    if isinstance(sections, list):
        for section in sections:
            section['items'] = []
    else:
        for name, section in sections.items():
            if isinstance(section, list):
                sections[name] = []
            else:
                section['items'] = []
    actual = [e['original_record'] for e in m['entries'] if e['source'] == 'envelope']
    if len(actual) != 1 or json.dumps(actual[0]) != json.dumps(envelope):
        raise RuntimeError('Envelope fields/order conservation failed: ' + row['key'])
    return m


def verify(row, manifest):
    root = Path(row['worktree']).resolve()
    for e in manifest['entries']:
        path = root / e['target']
        if not path.resolve().is_relative_to(root / 'docs/specs/history/legacy-ledger'):
            raise RuntimeError('Escaped output path')
        raw = path.read_bytes()
        if sha(raw) != e['content_hash']:
            raise RuntimeError('Output changed or incomplete: ' + str(path))
        if e['original_record'] not in list(json_fences(raw.decode('utf-8'))):
            raise RuntimeError('Markdown cannot recover original fields: ' + str(path))


def verify_skip(row):
    root = Path(row['worktree'])
    document = load(row['snapshot'])
    for section_pos, section in enumerate(document['sections']):
        for item_pos, item in enumerate(section['items']):
            path = root / 'docs/specs/history/legacy-ledger' / row['sha256'][:16] / f'{section_pos}-{item_pos}.md'
            if item not in list(json_fences(path.read_text(encoding='utf-8'))):
                raise RuntimeError('Previously imported source has no exact preservation: ' + str(path))


def capture_changes(row):
    root = Path(row['worktree'])
    base = row['execution_head']
    tracked = subprocess.check_output(['git', 'diff', '--name-only', '-z', base, '--', 'docs/specs', 'docs/design', 'docs/web-integration.md', 'AGENTS.md'], cwd=root, text=True, encoding='utf-8').split('\0')
    untracked = subprocess.check_output(['git', 'ls-files', '-z', '--others', '--exclude-standard', '--', 'docs/specs'], cwd=root, text=True, encoding='utf-8').split('\0')
    base_paths = set(subprocess.check_output(['git', 'ls-tree', '-r', '-z', '--name-only', base], cwd=root, text=True, encoding='utf-8').split('\0'))
    result = []
    for relative in sorted(set(tracked + untracked) - {''}):
        old = subprocess.run(['git', 'show', f'{base}:{relative}'], cwd=root, capture_output=True) if relative in base_paths else None
        path = root / relative
        result.append({'path': relative, 'before_hash': sha(old.stdout) if old is not None and old.returncode == 0 else None,
                       'after_hash': sha(path.read_bytes()) if path.is_file() else None,
                       'base_commit': base})
    return result


def run(args):
    registry_path = Path(args.registry).resolve()
    registry_bytes = registry_path.read_bytes()
    registry = json.loads(registry_bytes)
    directory = registry_path.parent
    lock_path = directory / 'batch.lock'
    lock_path.parent.mkdir(parents=True, exist_ok=True)
    with lock_path.open('a+b') as lock:
        if os.name == 'nt':
            import msvcrt
            lock.seek(0)
            lock.write(b'0')
            lock.flush()
            lock.seek(0)
            try:
                msvcrt.locking(lock.fileno(), msvcrt.LK_NBLCK, 1)
            except OSError:
                raise RuntimeError('Another migration batch owns this registry')
        else:
            import fcntl
            fcntl.flock(lock.fileno(), fcntl.LOCK_EX | fcntl.LOCK_NB)
        state_path = directory / 'batch-state.json'
        state = load(state_path) if state_path.exists() else {'registry_hash': sha(registry_bytes), 'sources': {}}
        if state['registry_hash'] != sha(registry_bytes):
            raise RuntimeError('Frozen registry changed; reconcile old state before starting a new batch')
        projects = set(args.only or [])
        known = {r['project'] for r in registry['sources']}
        if projects - known:
            raise RuntimeError('Unregistered projects: ' + ','.join(projects - known))
        selected = [r for r in registry['sources'] if not projects or r['project'] in projects]
        check_sources(registry, projects)
        for row in selected:
            key = row['key']
            if row['skip']:
                verify_skip(row)
                state['sources'][key] = {'status': 'skipped-already-migrated', 'records': row['records'], 'source_hash': row['sha256']}
                atomic_json(state_path, state)
                print(key, 'verified existing import; skipped')
                continue
            journal = directory / 'journals' / key / (row['sha256'] + '.json')
            command = [str(Path(args.binary).resolve()), 'spec-migrate', '--root', row['worktree'],
                       '--source', row['snapshot'], '--journal', str(journal), '--source-key', key]
            try:
                if args.action == 'prepare':
                    if not journal.exists():
                        subprocess.run(command + ['--action', 'dry-run'], check=True)
                    manifest = checked_manifest(row, journal)
                    status = 'prepared'
                else:
                    manifest = checked_manifest(row, journal)
                    if args.action in ['apply', 'rollback']:
                        # The selected source is a frozen isolated snapshot; the
                        # live origin must still match before and after the run.
                        subprocess.run(command + ['--action', args.action, '--legacy-writer-stopped'], check=True)
                    if args.action != 'rollback':
                        verify(row, manifest)
                        status = 'verified'
                    else:
                        status = 'import-rolled-back'
                        for e in manifest['entries']:
                            if e['before_hash'] is None and (Path(row['worktree']) / e['target']).exists():
                                raise RuntimeError('Rollback left a newly owned output')
                check_sources(registry, projects)
                state['sources'][key] = {'status': status, 'source_hash': row['sha256'], 'records': row['records'],
                    'journal': str(journal), 'outputs': len(manifest['entries']), 'write_set': capture_changes(row)}
                atomic_json(state_path, state)
                print(key, status, row['records'])
            except Exception as e:
                # A failed command can have completed earlier entries. Never
                # replace that fact with a fabricated zero-write rollback.
                state['sources'][key] = {'status': 'partial-or-conflict', 'journal': str(journal), 'error': str(e),
                    'source_hash': row['sha256'], 'write_set': capture_changes(row)}
                atomic_json(state_path, state)
                raise
        print(json.dumps({'selected_sources': len(selected), 'state': str(state_path), 'action': args.action}))


if __name__ == '__main__':
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('--registry', required=True)
    p.add_argument('--binary', required=True)
    p.add_argument('--action', choices=['prepare', 'apply', 'verify', 'rollback'], required=True)
    p.add_argument('--only', nargs='*')
    try:
        run(p.parse_args())
    except Exception as error:
        p.exit(1, f'migration batch: {error}\n')
