# 构造边界:多语言代码块 + 行内代码 + 表格 + 引用 + 列表

行内 `code` 与 **粗体**、*斜体*、[链接](https://example.com)。

## Rust

```rust
fn main() {
    let s = String::from("hello");
    println!("{s}");
}
```

## TypeScript

```typescript
export function i18nT(locale: string, key: string, params: obj): string {
    return key.split('.').reduce((o, p) => o?.[p], catalog[locale]) ?? key
}
```

## Bash

```bash
for f in *.md; do
  echo "processing $f"
done
```

## SQL

```sql
SELECT id, name FROM plans WHERE status = 'executing' ORDER BY created_at DESC;
```

> 引用块:渲染真源切换对拍。
>
> 第二段引用。

| 列 A | 列 B |
|---|---|
| 1 | 2 |
| 3 | 4 |

- 无序列表项一
- 无序列表项二
  1. 嵌套有序项
  2. 嵌套有序项二

普通段落结尾。
