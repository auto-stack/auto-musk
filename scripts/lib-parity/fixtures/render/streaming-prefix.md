# 流式前缀构造:截断的标题/段落/未闭合代码块(模拟 chat 流式 draft 中间态)

流式渲染中的 assistant 消息前缀——句子尚未

## 已完成的段落

这段是完整段落,含 `行内代码` 与 **加粗**。

```rust
// 代码块流式中——fence 未闭合
fn streaming_example(x: i32) -> i32 {
    x * 2
