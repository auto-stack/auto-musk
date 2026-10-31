"""Generate reviewable provenance reports from PLAN-107 frozen manifests.

These Markdown reports document migration decisions, not application indexes.
They never approve old product claims or replay historical test outcomes.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import re


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def cell(value):
    return str(value if value is not None else '—').replace('|', '\\|').replace('\n', ' ').replace('\r', '')


def link(root, relative, parent, title):
    path = root / relative.split('#')[0]
    if not path.is_file():
        raise RuntimeError('Missing reviewed reference: ' + str(path))
    suffix = ('#' + relative.split('#', 1)[1]) if '#' in relative else ''
    target = os.path.relpath(path, parent).replace('\\', '/') + suffix
    return '[' + cell(title) + '](' + target.replace(' ', '%20') + ')'


def archived(root, record):
    candidates = []
    identifier = str(record.get('id', ''))
    match = re.match(r'^(?:PLAN-|P)(\d+)', identifier)
    if match:
        candidates.append(match[1])
    for tag in record.get('tags') or []:
        match = re.fullmatch(r'plan:(\d+)', str(tag))
        if match:
            candidates.append(match[1])
    for seq in dict.fromkeys(candidates):
        paths = [p for d in ['archive', 'archived'] for p in (root / 'docs/plans' / d).glob(seq + '-*.md')]
        if len(paths) == 1:
            return str(paths[0].relative_to(root)).replace('\\', '/')
    return None


def decision(row, entry, projects, parent):
    root = Path(row['worktree'])
    record = entry['original_record']
    section = entry['source'].split(':', 1)[1].split('/', 1)[0]
    raw = str(record.get('file') or '').replace('\\', '/')
    reference = entry.get('reference')
    historical = archived(root, record)
    if row['project'] in ['auto-ui', 'auto-man'] and raw == 'docs/web-integration.md':
        reference = 'docs/specs/modules/web-integration.md'
    if reference and reference.startswith('docs/specs/') and section not in ['reviews', 'reports']:
        return 'current-source-linked', link(root, reference, parent, '当前规范'), '现有规范保持权威；旧摘要及状态完整保全，不从旧验证状态生成新批准。'
    # Explicit known repository prefixes identify the owner; never let a
    # short UI/OS Plan number resolve to a language Plan with the same number.
    for owner, owner_root in projects.items():
        prefixes = [owner + '/', '../' + owner + '/']
        matched = next((prefix for prefix in prefixes if raw.startswith(prefix)), None)
        if not matched or owner == row['project']:
            continue
        foreign = Path(owner_root)
        relative = raw[len(matched):]
        if '..' in Path(relative).parts or ':' in relative or relative.startswith('/'):
            continue
        scope_record = dict(record)
        match = re.match(r'^(?:UI|OS)(\d+)', str(record.get('id', '')))
        if match:
            scope_record['id'] = 'P' + match[1]
        receipt = re.match(r'docs/plans/attachments/(\d+)/', relative)
        if receipt:
            scope_record['id'] = 'P' + receipt[1]
        old_plan = archived(foreign, scope_record)
        if old_plan and (section in ['reviews', 'reports', 'tests'] or '/attachments/' in relative):
            return 'external-historical-plan', link(foreign, old_plan, parent, owner + ' 归档 Plan'), '原引用带明确外仓归属；改用稳定归档导航，原旧路径及状态保全，不重跑历史验证。'
        candidate = foreign / relative
        if candidate.is_file() and candidate.resolve().is_relative_to(foreign.resolve()):
            return 'external-owner-linked', link(foreign, relative, parent, owner + ' 实际来源'), '归属已登记的外仓；保留旧记录，不复制为本项目的新业务契约。'
    receipt = re.match(r'docs/plans/attachments/(\d+)/', raw)
    if not historical and receipt:
        historical = archived(root, {'id': 'P' + receipt[1]})
    if historical:
        return 'historical-plan-linked', link(root, historical, parent, '归档 Plan'), '关联唯一归档计划；旧实现细节和测试数字仅作历史，本次不重跑或重新批准。'
    if str(record.get('id', '')).startswith('AGEX-P002-'):
        return 'external-historical-receipt', 'ag-examples / PLAN-002（原引用见保全文件）', '外部示例仓的历史测试收据；不宣称本批重跑，也不将未登记外仓加入迁移。'
    if record.get('id') == 'G1' and not record.get('content') and record.get('title') == 'test':
        return 'empty-test-marker-preserved', '无当前业务规范', '无正文测试占位；保全原值，不制造现行目标。'
    if reference:
        reason = '原源上下文保留；当前业务要求继续查项目规范，不把过去的状态或结果晋升为现行要求。'
        if row['project'] == 'auto-down' and reference.startswith('jade-garden/'):
            reason = '原架构文档明确标注 plan 022 state，按历史架构保全，不冒称当前引擎全部现状。'
        return 'historical-source-linked', link(root, reference, parent, '原源上下文'), reason
    if section in ['reviews', 'reports', 'tests']:
        return 'historical-unresolved-source', '原路径未认证（原值完整保全）', '旧复审/测试/报告属于历史；缺少可核实来源，不生成当前批准或测试通过声明。'
    raise RuntimeError('Unreviewed record needs an explicit decision: ' + row['key'] + ' ' + entry['source'])


def run(registry_path):
    registry_path = Path(registry_path)
    base = registry_path.parent
    registry = json.loads(registry_path.read_text(encoding='utf-8'))
    projects = {r['project']: r['worktree'] for r in registry['sources']}
    counts = {}
    for row in registry['sources']:
        if row['skip']:
            continue
        root = Path(row['worktree'])
        manifest = json.loads((base / 'journals' / row['key'] / (row['sha256'] + '.json')).read_text())
        parent = root / 'docs/specs/history/legacy-ledger'
        parent.mkdir(parents=True, exist_ok=True)
        groups = {}
        for entry in manifest['entries']:
            if entry['source'] == 'envelope':
                continue
            role, reference, reason = decision(row, entry, projects, parent)
            section_pos = entry['source'].split('/')[0]
            record = entry['original_record']
            preservation = link(root, entry['target'], parent, '完整原记录')
            groups.setdefault(section_pos, []).append('| ' + ' | '.join([
                cell(entry['source']), cell(record.get('title')), cell(record.get('status')),
                role, preservation, reference, reason]) + ' |')
        decisions = []
        for section, lines in groups.items():
            page = parent / f'plan-107-{row["key"]}-section-{section}.md'
            page.write_text(f'# PLAN-107 / {row["project"]} / {row["key"]} / section {section}\n\n'
                f'来源 SHA256：`{row["sha256"]}`；本页 {len(lines)} 条。审阅对象为来源归属、时间类型与去向；旧运行数字不代表本批已执行。\n\n'
                '| 来源位置/旧ID | 旧标题 | 原状态 | 去向类型 | 字段保全 | 当前/历史来源 | 核对结论 |\n'
                '|---|---|---|---|---|---|---|\n' + '\n'.join(lines) + '\n', encoding='utf-8', newline='\n')
            decisions.append(f'- [{row["key"]} section {section}：{len(lines)} 条]({page.name})')
        report = parent / 'README.md'
        marker = '\n<!-- PLAN-107 migration reconciliation -->\n'
        existing = report.read_text(encoding='utf-8') if report.exists() else '# 旧 Spec 账本迁移对账\n'
        if marker in existing:
            existing = existing.split(marker)[0]
        aliases = [a for a in registry['aliases'] if a['project'] == row['project']]
        text = marker + f'\n## PLAN-107：{row["key"]}\n\n'
        text += f'旧源：`{row["project"]}/' + ('backend/' if 'backend' in row['key'] else '') + '.autoos/specs.json`。\n\n'
        text += f'原字节 SHA256：`{row["sha256"]}`；本批 {row["records"]} 条 + 1 envelope，全部原记录可从 AutoDown Markdown JSON 围栏回读。原 JSON 不改写、不删除。\n\n'
        text += f'提交归属：auto-musk PLAN-107；执行基线 `{row["execution_head"]}`。最终迁移提交和默认分支落地映射记录在该计划 §9；本报告与保全文档由同一仓库迁移提交管理。\n\n'
        text += '本批核对每条记录的来源、归属、当前/历史边界及保全内容；现有 canonical 文档优先，历史执行数字不重新认证，也不将旧状态改成 approved。无正文占位和外部历史收据有显式处理结果。\n\n'
        text += '\n'.join(decisions) + '\n\n'
        if aliases:
            text += '### 子模块去重\n\n'
            for alias in aliases:
                text += f'- `{alias["origin"]}`：原字节 SHA256 `{alias["sha256"]}`；解析值与本项目根账本相等，仅登记 alias，不重复导入、不改 gitlink。\n'
            text += '\n子模块组合检出仍使用原 pin；此处独立仓的文档迁移不代表组合发行已经更新。\n\n'
        text += '### 恢复与回滚\n\n'
        text += '未合入时使用同一 manifest 的 rollback，仅删除迁移自有且 after hash 未变的导入文件；修改过的文件明确拒绝。报告、规范提炼和规则变更在完整 Git 写集合中审阅。合入后从专用回滚工作树对本批提交作新的 `git revert`，无关后续修改保留，重叠修改先解决冲突；不 reset 默认分支。批次状态按项目记录，局部失败不伪称跨仓原子成功。\n\n'
        text += '原字节备份/journal/逐文件指纹在本地批次目录，遵守当前证据保留规则；长期回滚依据各仓 Git 提交、本文及原值保全文档，不能仅依赖临时附件。\n'
        report.write_text(existing.rstrip() + '\n' + text, encoding='utf-8', newline='\n')
        counts[row['key']] = sum(map(len, groups.values()))
    print(json.dumps({'reports': counts, 'total': sum(counts.values())}, ensure_ascii=False))


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--registry', required=True)
    run(parser.parse_args().registry)
