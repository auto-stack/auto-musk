"""PLAN-107 public batch CLI against real temporary Git worktrees."""
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

SCRIPT = Path(__file__).with_name('spec-migration-batch.py')
BINARY = Path(os.environ['MUSK_MIGRATION_BINARY']).resolve()


def git(root, *args, check=True):
    return subprocess.run(['git', *args], cwd=root, check=check, capture_output=True, text=True)


class MigrationBatchContract(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.base = Path(self.temp.name)
        self.rows = []
        for name in ['alpha', 'beta']:
            main = self.base / 'origins' / name
            main.mkdir(parents=True)
            git(main, 'init', '-b', 'main')
            git(main, 'config', 'user.name', 'Fixture')
            git(main, 'config', 'user.email', 'fixture@example.test')
            (main / 'README.md').write_text('baseline\n')
            (main / '.gitignore').write_text('.autoos/\n')
            git(main, 'add', '.')
            git(main, 'commit', '-m', 'baseline')
            work = self.base / 'worktrees' / name
            git(main, 'worktree', 'add', str(work), '-b', 'dev')
            head = git(work, 'rev-parse', 'HEAD').stdout.strip()
            origin = main / '.autoos/specs.json'
            origin.parent.mkdir()
            raw = json.dumps({'unknown': {'keep': 42}, 'sections': {'designs': [{'id': 'same', 'content': name + '\r\n旧正文'}]}}).encode()
            origin.write_bytes(raw)
            snapshot = self.base / ('snapshot-' + name + '.json')
            snapshot.write_bytes(raw)
            self.rows.append({'key': name + '-root', 'project': name, 'origin': str(origin), 'snapshot': str(snapshot),
                              'sha256': hashlib.sha256(raw).hexdigest(), 'records': 1, 'worktree': str(work),
                              'execution_head': head, 'skip': False})
        alias = self.base / 'alias-alpha.json'
        alias.write_text(json.dumps(json.loads(Path(self.rows[0]['origin']).read_bytes()), indent=4))
        self.registry = self.base / 'registry.json'
        self.registry.write_text(json.dumps({'sources': self.rows, 'aliases': [{'project': 'alpha', 'origin': str(alias),
            'snapshot': str(alias), 'sha256': hashlib.sha256(alias.read_bytes()).hexdigest(),
            'canonical_source': 'alpha-root', 'equivalent': True}]}))

    def tearDown(self):
        for row in self.rows:
            git(Path(row['origin']).parent.parent, 'worktree', 'remove', '--force', row['worktree'])
        self.temp.cleanup()

    def batch(self, action, only=None, success=True):
        args = [sys.executable, str(SCRIPT), '--registry', str(self.registry), '--binary', str(BINARY), '--action', action]
        if only:
            args += ['--only', only]
        out = subprocess.run(args, capture_output=True, text=True)
        self.assertEqual(out.returncode == 0, success, out.stdout + out.stderr)
        return out

    def test_aliases_are_not_imported_twice_and_repeated_apply_and_rollback_are_safe(self):
        self.batch('prepare')
        self.batch('apply')
        self.batch('apply')
        self.batch('verify')
        state = json.loads((self.base / 'batch-state.json').read_text())
        self.assertEqual(len(state['sources']), 2)
        self.assertTrue(all(s['status'] == 'verified' for s in state['sources'].values()))
        for row in self.rows:
            files = list((Path(row['worktree']) / 'docs/specs/history/legacy-ledger').rglob('*.md'))
            self.assertEqual(len(files), 2)  # one record and one envelope
        self.batch('rollback')
        self.batch('rollback')
        for row in self.rows:
            self.assertEqual(list((Path(row['worktree']) / 'docs/specs/history/legacy-ledger').rglob('*.md')), [])

    def test_source_drift_blocks_its_project_while_an_independent_project_can_continue(self):
        self.batch('prepare')
        Path(self.rows[0]['origin']).write_text('changed')
        self.batch('apply', only='alpha', success=False)
        self.batch('apply', only='beta')
        state = json.loads((self.base / 'batch-state.json').read_text())
        self.assertEqual(state['sources']['beta-root']['status'], 'verified')
        self.assertFalse((Path(self.rows[0]['worktree']) / 'docs').exists())

    def test_divergent_alias_refuses_to_discard_its_unique_records(self):
        registry = json.loads(self.registry.read_text())
        alias = registry['aliases'][0]
        source = Path(alias['snapshot'])
        value = json.loads(source.read_bytes())
        value['unique_alias_record'] = {'content': 'must not disappear'}
        source.write_text(json.dumps(value))
        alias['sha256'] = hashlib.sha256(source.read_bytes()).hexdigest()
        self.registry.write_text(json.dumps(registry))
        self.batch('prepare', success=False)
        self.assertFalse((Path(self.rows[0]['worktree']) / 'docs').exists())

    def test_git_checkout_keeps_preview_hash_with_crlf_original_content(self):
        self.batch('prepare')
        self.batch('apply')
        for row in self.rows:
            root = Path(row['worktree'])
            git(root, 'config', 'core.autocrlf', 'input')
            git(root, 'add', 'docs/specs')
            git(root, 'commit', '-m', 'migration')
            target = next((root / 'docs/specs/history/legacy-ledger').rglob('*-0.md'))
            before = target.read_bytes()
            self.assertNotIn(b'\r\n', before)
            relative = str(target.relative_to(root))
            target.unlink()
            git(root, 'restore', '--', relative)
            self.assertEqual(target.read_bytes(), before)
        self.batch('verify')

    def test_two_actual_apply_processes_share_the_project_lock(self):
        self.batch('prepare')
        row = self.rows[0]
        journal = self.base / 'journals' / row['key'] / (row['sha256'] + '.json')
        args = [str(BINARY), 'spec-migrate', '--root', row['worktree'], '--source', row['snapshot'],
                '--source-key', row['key'], '--journal', str(journal), '--action', 'apply', '--legacy-writer-stopped']
        children = [subprocess.Popen(args, stdout=subprocess.PIPE, stderr=subprocess.PIPE) for _ in range(2)]
        for child in children:
            _, error = child.communicate(timeout=30)
            self.assertEqual(child.returncode, 0, error.decode(errors='replace'))
        self.batch('verify', only='alpha')
        self.assertEqual(len(list((Path(row['worktree']) / 'docs/specs/history/legacy-ledger').rglob('*.md'))), 2)

    def test_partial_failure_preserves_user_edit_and_reconstructs_resume_from_files(self):
        self.batch('prepare')
        row = self.rows[0]
        journal = self.base / 'journals' / row['key'] / (row['sha256'] + '.json')
        m = json.loads(journal.read_text())
        e = m['entries'][0]
        path = Path(row['worktree']) / e['target']
        path.parent.mkdir(parents=True)
        path.write_text('foreign user content')
        self.batch('apply', only='alpha', success=False)
        self.assertEqual(path.read_text(), 'foreign user content')
        state = json.loads((self.base / 'batch-state.json').read_text())
        self.assertEqual(state['sources']['alpha-root']['status'], 'partial-or-conflict')
        path.write_bytes(e['content'].encode('utf-8'))
        self.batch('apply', only='alpha')
        path.write_text('later edit')
        self.batch('rollback', only='alpha', success=False)
        self.assertEqual(path.read_text(), 'later edit')

    def test_landed_rollback_is_a_new_commit_and_keeps_later_user_changes(self):
        self.batch('prepare')
        self.batch('apply', only='alpha')
        root = Path(self.rows[0]['worktree'])
        git(root, 'add', 'docs/specs')
        git(root, 'commit', '-m', 'migrate')
        delivery = git(root, 'rev-parse', 'HEAD').stdout.strip()
        main = Path(self.rows[0]['origin']).parent.parent
        git(main, 'merge', '--ff-only', 'dev')
        (root / 'README.md').write_text('later user change\n')
        git(root, 'add', 'README.md')
        git(root, 'commit', '-m', 'user')
        git(root, 'revert', '--no-edit', delivery)
        self.assertEqual((root / 'README.md').read_text(), 'later user change\n')
        self.assertFalse((root / 'docs/specs/history').exists())
        git(main, 'merge', '--ff-only', 'dev')
        self.assertEqual((main / 'README.md').read_text(), 'later user change\n')
        self.assertEqual(git(root, 'merge-base', '--is-ancestor', delivery, 'HEAD').returncode, 0)
        # Recreate the migration, then overlap a later user edit with a file
        # that rollback would delete. Git must report a real conflict.
        undo = git(root, 'rev-parse', 'HEAD').stdout.strip()
        git(root, 'revert', '--no-edit', undo)
        migration = git(root, 'rev-parse', 'HEAD').stdout.strip()
        target = next((root / 'docs/specs/history/legacy-ledger').rglob('*-0.md'))
        target.write_text('later overlapping user edit')
        git(root, 'add', 'docs/specs')
        git(root, 'commit', '-m', 'user edits imported file')
        result = git(root, 'revert', '--no-edit', migration, check=False)
        self.assertNotEqual(result.returncode, 0)
        git(root, 'revert', '--abort')
        self.assertEqual(target.read_text(), 'later overlapping user edit')


if __name__ == '__main__':
    unittest.main()
