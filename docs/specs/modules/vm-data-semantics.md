# VM 数据读侧契约（vm-data-semantics）

> PLAN-066 T-04~T-07 交付的双轨等价契约收口：VM 轨数据读侧与 web 轨同语义
> 的 enduring 规则与已知边界。

## typeof 属性抢占收窄（KD-057① 清偿）

`obj.type` 域读不再无条件抢占为编译期型名：typeof 语义仅原始接收者生效
（`StrFixed(0)` 动态推断哨兵排除），对象接收者落 GET_FIELD 通道与 web 轨
a2ts 纯属性访问同语义。musk ~90 处 `.type` 字段比较（forge_store ev.type
分派/questionnaire q.type 等）随之解锁。

## Regex.match JS web 语义（KD-057② 清偿）

三参契约 `[text, pattern, flags]`（自顶向下）：无 `'g'` 返回
`[全匹配, 组1, …]` 堆列表；含 `'g'` 返回全部匹配子串列表；空匹配空列表；
两参调用 codegen 编译期补 `flags=""`；`'i'` 支持。回归锁：musk_vm_track
p066_2（wl_probe18 全形态+组提取+583 元素存活锁）。

## str.includes 等实例方法（055-4⑥ 现代真身清偿）

合成 fn（handler/computed）内 `str.*` 未注册方法（includes/startsWith/
trim 族）路由引擎 CALL_SPEC str 臂（引擎已实现族随通）；`auto.str.includes`
另注册原生 shim（id2461，实例 CALL_NAT `[pat, receiver]`）。回归锁：语料
`test/ui/plan066_filter_projection/` PA（过滤投影命中 2/清空 3/miss 0）。

**Face A 已知边界（未修，另行立案候选）**：列表字段读改写
（`.messages = .messages + [..]`）在旧值陈旧/Nil 时静默失效；整写正常。
musk 生产消息列表全为后端快照整写，未踩。

## P536-D2 SET_FIELD 裁定

跨模块调用帧内 SET_FIELD 写共享根态——当前树实机语料裁定**可达**
（MarkDone→ClearWindow 自调链写落盘；624 闭包帧协议修复或更早已覆盖）。
musk `done` 臂「本臂自清」绕行不再必要但保留（防御性等价）。

## ThinkBlock chevron 读侧独立（KD-057③/T-13 清偿）

子件 computed 串等值（`isOpen => .expanded == .current_msg.id` 形态）+
子件 handler 直调 store msg 翻转，全链回归锁绿（语料 PC 三断言：翻转
expanded="m1"/展开 true/收起 false）。obj-prop 传递依赖逐帧烘焙（生产
每帧重建），纯静态 harness 不可测——行为面以实机验收。

## use 模块名纪律

`use <module>: <Symbol>` 的 module 名必须与文件名一致（resolve 按名找
文件，`use msg_bubble:` 配 bubble.at 永远解析失败且静默）。
