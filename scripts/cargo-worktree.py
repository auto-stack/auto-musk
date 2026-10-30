"""Run Cargo with explicit dependency roots, restoring the manifest afterwards.

No filesystem links. Use only in a worktree with one build owner. Cargo has no
environment interpolation for path dependencies; the temporary paths are never
delivery changes. Environment overrides > group siblings > main checkouts.
"""
import os
from pathlib import Path
import re
import subprocess
import sys

root = Path(__file__).resolve().parents[1]
if not (root / '.git').is_file():
    raise SystemExit('This build helper temporarily resolves manifests: use a dedicated Git worktree')
manifest = root / 'backend/crates/musk/Cargo.toml'
original = manifest.read_bytes()
text = original.decode('utf-8')
roots = {}
for repo, variable in [('auto-lang', 'AUTO_LANG_ROOT'), ('auto-ai', 'AUTO_AI_ROOT'), ('auto-ui', 'AUTO_UI_ROOT'), ('auto-down', 'AUTO_DOWN_ROOT')]:
    candidates = [Path(os.environ[variable])] if variable in os.environ else [root.parent / repo, Path('D:/autostack') / repo]
    dependency = next((p.resolve() for p in candidates if (p / '.git').exists()), None)
    if dependency is None:
        raise SystemExit(f'Missing {repo}; set {variable}')
    roots[repo] = dependency
    text = text.replace(f'../../../../{repo}/', dependency.as_posix() + '/')
restores = {manifest: original}
try:
    # A dependency worktree's transitive references need the same root override.
    for consumer in [roots['auto-ai'], roots['auto-ui']]:
      if (consumer / '.git').is_file():
        for child in (consumer / 'crates').glob('*/Cargo.toml'):
            old = child.read_bytes()
            resolved = old.decode('utf-8')
            for repo, dependency in roots.items():
                resolved = resolved.replace(f'../../../{repo}/', dependency.as_posix() + '/')
            if resolved != old.decode('utf-8'):
                restores[child] = old
                child.write_text(resolved, encoding='utf-8', newline='')
    manifest.write_text(text, encoding='utf-8', newline='')
    result = subprocess.run(['cargo', *sys.argv[1:]], cwd=root)
finally:
    for path, content in restores.items():
        path.write_bytes(content)
sys.exit(result.returncode)
