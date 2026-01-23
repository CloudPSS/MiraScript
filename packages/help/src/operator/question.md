---
token: '?'
order: 6
---

`?` 用于条件表达式（三元表达式）：`cond ? thenExpr : elseExpr`。

```mira
fn sign { it > 0 ? 1 : it < 0 ? -1 : 0 }
```
