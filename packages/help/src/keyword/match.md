---
token: 'match'
order: 16
---

`match` 表达式对一个值进行分支匹配，由多个 `case` 组成。

```mira
fn classify {
  match it {
    case nil { "nil" }
    case 0 { "zero" }
    case _ { "other" }
  }
}
```
