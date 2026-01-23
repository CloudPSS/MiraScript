---
token: 'case'
order: 17
---

`case` 是 `match` 表达式中的分支。

```mira
match 3 {
  case 1..3 { "small" }
  case _ { "other" }
}
```
