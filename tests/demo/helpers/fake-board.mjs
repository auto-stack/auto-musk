// tests/demo/helpers/fake-board.mjs — V02 共享桩:合规/缺陷 FocusBoard 假应用。
// VM 快照协议(与宿主解析器同形)+ autoui_action + 磁盘持久化(board.json)。
// defect: null | 'no-persist' | 'lose-on-restart' | 'shrink-drops'
import fs from 'node:fs';
import path from 'node:path';

// ── 假 FocusBoard 应用：VM 快照协议（解析器同形）+ autoui_action + 磁盘持久化
// mode: 'p001' 三列无上限 | 'p002' 固定上限 3 | 'p003' 可配置上限（初始 3）
function fakeBoard({ mode, defect = null }) {
  const st = { tasks: [], buffer: '', editing: null, editBuf: '', pendingDelete: null,
    limit: mode === 'p001' ? Infinity : 3, limitBuf: '', error: '', gen: 5 };
  let repoDir = null;
  const dataFile = () => path.join(repoDir, 'src', 'front', 'data', 'board.json');
  const persist = () => {
    if (defect === 'no-persist') return;
    fs.mkdirSync(path.dirname(dataFile()), { recursive: true });
    fs.writeFileSync(dataFile(), JSON.stringify({ tasks: st.tasks, limit: st.limit }));
  };
  const restore = () => {
    if (defect === 'lose-on-restart') { st.tasks = []; return; }
    if (!fs.existsSync(dataFile())) return;
    const j = JSON.parse(fs.readFileSync(dataFile(), 'utf8'));
    st.tasks = j.tasks;
    st.limit = mode === 'p001' ? Infinity : (j.limit ?? 3);
  };
  const doingCount = () => st.tasks.filter(t => t.col === 'doing').length;
  const esc = s => String(s).replace(/["\\]/g, '');

  let controls = {};
  const snapshot = () => {
    controls = {};
    const L = [];
    let vid = 100;
    const add = (indent, kind, label, handler = null, ph = null) => {
      const id = `v${vid++}`;
      L.push(`${'  '.repeat(indent)}${kind} #${id} "${esc(label)}"`);
      if (ph) L.push(`${'  '.repeat(indent + 1)}placeholder: "${esc(ph)}"`);
      if (handler) controls[id] = handler;
      return id;
    };
    add(0, 'window', 'FocusBoard');
    if (mode === 'p003') {
      add(1, 'text', `上限设置 ${st.limit}`);
      add(1, 'input', st.limitBuf, args => { st.limitBuf = String(args?.value ?? ''); }, '上限(1~5)');
      add(1, 'button', '保存设置', () => {
        const n = Number(st.limitBuf);
        if (!Number.isInteger(n) || n < 1 || n > 5) { st.error = `错误:上限必须是 1~5 的整数,当前 ${st.limit}`; st.limitBuf = ''; return; }
        st.limit = n; st.limitBuf = ''; persist();
        if (defect === 'shrink-drops' && Number.isFinite(st.limit)) {
          const doing = st.tasks.filter(t => t.col === 'doing');
          doing.slice(st.limit).forEach(t => { t.col = 'todo'; });
        }
      });
    } else if (mode === 'p002') {
      add(1, 'text', `并行上限 ${st.limit}`);
    } else {
      add(1, 'text', '任务看板');
    }
    add(1, 'input', st.buffer, args => { st.buffer = String(args?.value ?? ''); }, '任务标题');
    add(1, 'button', '添加', () => {
      if (!st.buffer.trim()) return;
      st.tasks.push({ title: st.buffer, col: 'todo' });
      st.buffer = ''; persist();
    });
    for (const [key, name] of [['todo', 'todo'], ['doing', 'doing'], ['done', 'done']]) {
      const items = st.tasks.filter(t => t.col === key);
      add(1, 'column', `${name} ${items.length}${key === 'doing' && Number.isFinite(st.limit) ? `/${st.limit}` : ''}`);
      for (const t of items) {
        add(2, 'card', t.title);
        if (st.editing === t.title) {
          add(3, 'input', st.editBuf, args => { st.editBuf = String(args?.value ?? ''); }, '新标题');
          add(3, 'button', '保存', () => {
            const task = st.tasks.find(x => x.title === st.editing);
            if (task && st.editBuf.trim()) { task.title = st.editBuf; persist(); }
            st.editing = null; st.editBuf = '';
          });
        }
        add(3, 'button', '重命名', () => { st.editing = t.title; st.editBuf = t.title; });
        add(3, 'button', '删除', () => { st.pendingDelete = t.title; });
        if (st.pendingDelete === t.title) add(3, 'button', '确认删除', () => {
          st.tasks = st.tasks.filter(x => x.title !== t.title);
          st.pendingDelete = null; persist();
        });
        if (key === 'todo') add(3, 'button', '→ doing', () => {
          if (doingCount() >= st.limit) { st.error = `已到上限:当前 doing ${doingCount()}/${st.limit},不能添加`; return; }
          t.col = 'doing'; persist();
        });
        if (key === 'doing') {
          add(3, 'button', '→ done', () => { t.col = 'done'; persist(); });
          add(3, 'button', '← todo', () => { t.col = 'todo'; persist(); });
        }
        if (key === 'done') add(3, 'button', '← todo', () => { t.col = 'todo'; persist(); });
      }
    }
    if (st.error) { add(1, 'text', st.error); st.error = ''; }
    return L.join('\n');
  };

  return {
    set repoDir(d) { repoDir = d; },
    get state() { return st; },
    snapshot: async () => snapshot(),
    stableSnapshot: async function () { return await snapshot(); },
    call: async (name, args) => {
      if (name !== 'autoui_action') throw new Error(`fake: unknown tool ${name}`);
      const h = controls[args?.element_id];
      if (!h) throw new Error('MCP autoui_action isError: VNode not found');
      h(args);
      return 'ok';
    },
    __restart: async () => { restore(); st.gen += 1; return { oldGen: st.gen - 1, newGen: st.gen }; },
  };
}

export { fakeBoard };
