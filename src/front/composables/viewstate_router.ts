// viewstate_router.ts — URL 路由桥(PLAN-041 T8,web useViewState 对拍面)
//
// T7 勘察裁定(auto-lang master 实证):语言层 `routes` 块生成 vue-router
// + createWebHashHistory(hash 路由 + 页面模块挂载),不匹配 useViewState
// 语义(path URL + 单 App 视图状态机 + detail 子路径);widget 事件面亦无
// popstate/pushState 表达。故 URL 路由落端口层——本文件为 web adapter
// (history API),.at 侧经 composables 端口消费;Rust/VM 后端未来提供同名
// adapter(与五域端口机制一致)。
//
// 形态限制(单 store per widget):App widget 持 AuthStore 后无法再挂
// ViewStateStore,popstate→视图切换经"点击对应 rail tab"桥接(等价触发
// Show* 消息链);detail 子路径的 popstate 实时恢复仅覆盖挂载时(Init),
// 会话中 back/forward 的 detail 级变化不回放——KNOWN-DEBT 登记。

const VALID_VIEWS = ['chats', 'plans', 'specs', 'wiki'] as const
const DEFAULT_VIEW = 'chats'

let _currentView: string = DEFAULT_VIEW
let _currentDetail = ''
let _installed = false

export function vsParseUrl(): { view: string; detail: string } {
  if (typeof window === 'undefined') return { view: DEFAULT_VIEW, detail: '' }
  const path = window.location.pathname
  let rest = path.replace(/^\/+/, '').replace(/\/+$/, '')
  if (!rest) return { view: DEFAULT_VIEW, detail: '' }
  const [rawView, ...detailParts] = rest.split('/')
  if (!(VALID_VIEWS as readonly string[]).includes(rawView)) {
    return { view: DEFAULT_VIEW, detail: '' }
  }
  return { view: rawView, detail: detailParts.join('/') }
}

function buildPath(view: string, detail: string): string {
  return '/' + view + (detail ? '/' + detail : '')
}

/** web useViewState.updateHistory 对拍:同路径守卫 + 保 query(workspace)。 */
function pushHistory(view: string, detail: string, replace = false): void {
  const path = buildPath(view, detail)
  if (typeof window !== 'undefined' && window.location.pathname !== path) {
    const url = path + window.location.search
    if (replace) window.history.replaceState({}, '', url)
    else window.history.pushState({}, '', url)
  }
}

/** .at 侧读取:当前视图(挂载时 Init 消费)。 */
export function vsCurrentView(): string {
  return _currentView
}

/** .at 侧读取:当前 detail 子路径(挂载时 Init 消费)。 */
export function vsCurrentDetail(): string {
  return _currentDetail
}

/** .at 侧写:切视图(rail 按钮消息链调用;pushState)。 */
export function vsSetView(view: string): void {
  if (!(VALID_VIEWS as readonly string[]).includes(view)) return
  _currentView = view
  _currentDetail = ''
  pushHistory(view, '', false)
}

/** .at 侧写:detail 子路径(replaceState,web setDetailPath 对拍)。 */
export function vsSetDetailPath(detail: string): void {
  _currentDetail = detail
  pushHistory(_currentView, detail, true)
}

/**
 * 安装桥(App setup 调用一次,同 gate_router 模式):
 * - URL → 状态(初始解析 + popstate 监听)
 * - popstate 视图级变化 → 点击对应 rail tab(触发 .at Show* 消息链;
 *   同视图 detail 级变化不回放,见文件头登记)
 */
export function useViewRouter(): void {
  if (_installed || typeof window === 'undefined') return
  _installed = true

  const init = vsParseUrl()
  _currentView = init.view
  _currentDetail = init.detail

  window.addEventListener('popstate', () => {
    const parsed = vsParseUrl()
    if (parsed.view !== _currentView) {
      _currentView = parsed.view
      _currentDetail = parsed.detail
      // rail tab 顺序 = VALID_VIEWS 顺序(App view rail)。
      const idx = (VALID_VIEWS as readonly string[]).indexOf(parsed.view)
      const tab = document.querySelector<HTMLButtonElement>(
        `div.gap-1 > button.rail-tab:nth-child(${idx + 1})`,
      )
      if (tab) tab.click()
    } else {
      _currentDetail = parsed.detail
    }
  })
}
