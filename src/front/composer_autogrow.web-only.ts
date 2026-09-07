// composer_autogrow.web-only.ts — composer 自适应增高（web 专属逃生舱）。
//
// zcode/codex 式 composer：输入面高度随内容增长（空态 ~1 行,封顶 200px 后
// 内滚）。textarea 因 mention backdrop 镜像（codegen 由声明类逐 token 推导
// 背板类,PLAN-493）恒 absolute inset-0 h-full——增高只能打在其 wrapper
//（parent）上：先 0px 复位测 scrollHeight 再定 wrapper 高（h-full 下
// scrollHeight 恒等于 clientHeight,不复位则发送清空后永远缩不回去）。
//
// 调用面：platformComposerAutoGrow() 幂等重入——首调装 document 级委托
// input 监听（打字即时增高）+ resize 重排；所有调用统一 setTimeout(0) 合帧
// （Vue v-model 程序化清空/mention 插入不冒泡 input 事件,委托监听覆盖不到,
// 必须显式调度;rAF 在 IAB/后台页停帧会饿死,沿 focus_composer 口径）。
// VM/iced：platform.vm.at 同名 fn 空实现,text_editor 固定 height:72。

const MAX_EDITOR_HEIGHT = 200

function growOne(ta: HTMLTextAreaElement): void {
  const wrap = ta.parentElement
  if (!wrap) return
  ta.style.height = '0px'
  const h = Math.min(ta.scrollHeight, MAX_EDITOR_HEIGHT)
  ta.style.height = ''
  wrap.style.height = `${h}px`
}

function growAll(): void {
  document.querySelectorAll<HTMLTextAreaElement>('textarea.chats-input').forEach(growOne)
}

let installed = false
let scheduled = false

export function composerAutoGrow(): void {
  if (!installed) {
    installed = true
    document.addEventListener('input', (ev) => {
      const t = ev.target
      if (t instanceof HTMLTextAreaElement && t.classList.contains('chats-input')) {
        growOne(t)
      }
    })
    window.addEventListener('resize', growAll)
  }
  if (scheduled) return
  scheduled = true
  setTimeout(() => {
    scheduled = false
    growAll()
  }, 0)
}
