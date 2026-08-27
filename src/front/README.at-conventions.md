# src/front Auto(.at) 编写规约 —— VM 轨可用性约定

> 来源:PLAN-045 VM-clean 现场修复 + PLAN-046 T11 固化。目的:把"vue 轨容忍、
> VM 轨拒收"的写法差异前置到编码期,避免探针回红后的事后清理批次。
> 门禁:`scripts/vm-link-probe.cmd`(改任何 front 源后跑一次)。

## R1 普通 fn 与 store state 共享:必须纯 fn 形态

**规约**:普通 `fn` 不直读 store state(`.messages` 这类 `.field`)——handler
合成重写器只对 handler 体走状态引用改写,普通 fn 体内的 `.x` 会漏成裸
`self`,VM 合成报 `Undefined symbol`,整个 handler 毒化。

- ✅ 正例(store handler 内把数据当参数传出,副作用留在 handler):

```at
// forge_store.at —— PLAN-045 现场定形
var msg = currentAssistantMsgIn(.messages)   // 读 state → 参数传入
msg.content = chunk                          // 写回也经局部中转
```

```at
// 普通 fn:只碰参数
fn currentAssistantMsgIn(msgs Value) Value { /* 纯参数驱动 */ }
```

- ❌ 反例(会链接死):

```at
fn ensureAssistantMsg() {
    .messages.push(...)     // fn 体直读 state → 裸 self → Undefined symbol
}
```

判断速记:**能抄进独立库文件而不缺上下文的 fn 才是安全形态**。

## R2 动态值(obj 接收者)方法面

截至 PLAN-046,obj(json/dynamic 值)接收者的方法族在 VM 无 native:
`.find/.slice/.map` 等在动态接收者上链接死(上游补 native 前规避,
KNOWN-DEBT 045/046 行)。当前可用形态:

| 场景 | 可用写法 | 备注 |
|---|---|---|
| 查找元素 | 手扫 while 循环 | relayFindRun 等,见 046-T4 清册 |
| 字典遍历 | `Object.keys(x)` + 索引 | token_cost/getErrand 形态 |
| join 拼接 | 显式 `[]str` 定型接收者,或 helper fn | specDepsJoinText 形态 |
| slice 截取 | 先定型 `var c str = …` 再 `c.slice()` | chatBranchLabel 形态 |
| sort | `data.sort((a,b)=>…)` | 已验证可链接 |

上游 obj 方法族 native 到位后按 [PLAN-046 T4 清册]回撤(仅回撤为绕链接而
变形的位点)。

## R3 其它已固化约束(摘要)

- **let 重赋值**:声明后被重赋的变量用 `var`(VM 拒收 let 重赋,vue/JS 容忍)。
- **裸 window 全局**:一律走 `ports/platform.at`(`use.web` 引用;
  如 platformViewportHeight)。源码出现裸 `window.` 即违规。
- **store 层不调 ports**:跨层调用禁止,平台副作用经视图/handler 层转发
  (auth 头重注入协议见 `ports/platform.vm.at` 头注与 auth_store.Me 注)。
